use crate::process;
use mlua::{Function, Lua, RegistryKey, Result, Table, Value, Variadic, UserDataMethods, UserData};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::device;
use crate::lua_api::input_types;

// load init.lua from path
pub fn load_lua(lua: &Lua, path: &Path) {
    let config_path = path.join("config.lua");
    let code = fs::read_to_string(&config_path).expect("config.luaが読み込めません");
    lua.load(&code)
        .exec()
        .expect("config.luaのロード時にエラーが発生しました");
}

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

struct DeviceData {
    name: String,
    keymap: HashMap<input_types::Token, RegistryKey>,
}

impl UserData for DeviceData {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("map", |lua, this, (key, func): (mlua::String, Function)| {
            // FunctionをRegistryに保存
            let key_ref = lua.create_registry_value(func)?;
            Ok(())
        });
    }
}

// functions
pub fn register_api(lua: &Lua) -> Result<()> {
    let ouka = lua.create_table()?;

    // define map
    {
        let marge_tables = lua.create_function(|lua, tables: Variadic<Table>| {
            let out = lua.create_table()?;
            for table in tables {
                merge_into(lua, &out, &table)?;
            }
            Ok(out)
        })?;
        ouka.set("margeTables", marge_tables)?;
    }

    // get device by name
    {
        let get_device_by_name = lua.create_function(|lua, str: mlua::String| {
            let out = lua.create_table()?;
            let devices = device::find_device_by_name(str.to_str()?);
            if devices.is_none() {
                eprintln!("The target device cannot be resolved.");
                process::exit(1);
            }
            println!("=== Target device ===");
            device::print_device_list(devices.as_deref().unwrap_or(&[]));
            device::listen_device_list(devices.as_deref().unwrap_or(&[]));
            Ok(out)
        })?;
        ouka.set("getDeviceByName", get_device_by_name)?;
    }

    lua.globals().set("ouka", ouka)?;
    Ok(())
}
