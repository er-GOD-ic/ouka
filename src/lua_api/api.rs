use crate::process;
use mlua::{Function, Lua, RegistryKey, Result, Table, UserData, UserDataMethods, Value, Variadic};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::device;
use crate::lua_api::input_types;

fn merge_into(lua: &Lua, dest: &Table, src: &Table) -> Result<()> {
    for pair in src.clone().pairs::<Value, Value>() {
        let (k, v) = pair?;
        match v {
            Value::Table(v_table) => {
                // dest に既にテーブルがあるか確認
                match dest.get::<_, Value>(k.clone())? {
                    Value::Table(dest_sub) => {
                        // 両方テーブル -> 再帰マージ
                        merge_into(lua, &dest_sub, &v_table)?;
                    }
                    _ => {
                        // dest にテーブルが無い -> 新しいテーブルを作ってコピー（元の src を変更しない）
                        let new_table = lua.create_table()?;
                        merge_into(lua, &new_table, &v_table)?;
                        dest.set(k, new_table)?;
                    }
                }
            }
            other => {
                // テーブルでない値は上書き
                dest.set(k, other)?;
            }
        }
    }
    Ok(())
}

fn merge_tables<'lua>(lua: &'lua Lua, a: Table<'lua>, b: Table<'lua>) -> Result<Table<'lua>> {
    let result = lua.create_table()?;

    // copy a into result
    a.clone().pairs::<Value, Value>().try_for_each(|pair| {
        let (k, v) = pair?;
        result.set(k, v)
    })?;

    // fold b over result, recursively merging sub-tables
    b.clone()
        .pairs::<Value, Value>()
        .try_fold(result, |acc, pair| {
            let (k, v) = pair?;
            match v {
                Value::Table(v_table) => {
                    let merged = match acc.get::<_, Value>(k.clone())? {
                        Value::Table(acc_sub) => merge_tables(lua, acc_sub, v_table)?,
                        _ => merge_tables(lua, lua.create_table()?, v_table)?,
                    };
                    acc.set(k, merged)?;
                    Ok(acc)
                }
                other => {
                    acc.set(k, other)?;
                    Ok(acc)
                }
            }
        })
}

struct DeviceData {
    device: Option<Vec<device::DeviceHandler>>,
    keycodes: Option<RegistryKey>,
    keymap: HashMap<input_types::Hotkey, RegistryKey>,
}

impl UserData for DeviceData {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("setKeycodes", |lua, this, tables: Variadic<Table>| {
            let keycode_table = lua.create_table()?;
            for table in tables {
                merge_into(lua, &keycode_table, &table)?;
            }

            this.keycodes = Some(lua.create_registry_value(keycode_table)?);
            Ok(())
        });
        methods.add_method("listen", |_, this, ()| {
            device::listen_device_list(this.device.as_deref().unwrap_or(&[]));
            Ok(())
        });
        methods.add_method_mut(
            "map",
            |lua, this, (pattern, func): (mlua::String, Function)| {
                if let Some(ref keycodes) = this.keycodes {
                    this.keymap.insert(
                        input_types::parse_hotkey(lua, keycodes, pattern.to_str().unwrap()),
                        lua.create_registry_value(func)
                            .expect("cannot create registry value"),
                    );
                }
                Ok(())
            },
        );
    }
}

// functions
pub fn register_api(lua: &Lua) -> Result<()> {
    let ouka = lua.create_table()?;
    // get device by name
    {
        let get_device_by_name = lua.create_function(|lua, str: mlua::String| {
            let device = device::find_device_by_name(str.to_str()?);
            if device.is_none() {
                eprintln!("The target device cannot be resolved.");
                process::exit(1);
            }
            println!("=== Target device ===");
            device::print_device_list(device.as_deref().unwrap_or(&[]));
            let out = DeviceData {
                device: device,
                keycodes: None,
                keymap: HashMap::new(),
            };
            Ok(out)
        })?;
        ouka.set("getDeviceByName", get_device_by_name)?;
    }

    lua.globals().set("ouka", ouka)?;
    Ok(())
}
