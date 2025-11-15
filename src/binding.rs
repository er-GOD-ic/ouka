use crate::device::*;
use crate::hotmap::*;
use evdev::{EventType, InputEvent, KeyCode};
use mlua::Lua;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io;
use std::io::{Error, ErrorKind};

pub struct Binding {
    device: DeviceHandler,
    code_table: HashMap<String, KeyCode>,
    value_table: HashMap<String, i32>,
    hotmap: HotMap,
}

impl Binding {
    pub fn new(device: DeviceHandler) -> Self {
        Self {
            device: device,
            code_table: HashMap::new(),
            value_table: HashMap::new(),
            hotmap: HotMap::new(),
        }
    }
}

impl mlua::UserData for Binding {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "setKeycodes",
            |_, this, tables: mlua::Variadic<mlua::Table>| {
                this.code_table = lua_table_to_hashmap(tables)
                    .into_iter()
                    .map(|(k, v)| (k, KeyCode(v as u16)))
                    .collect();
                Ok(())
            },
        );
        methods.add_method_mut(
            "setKeyValues",
            |_, this, tables: mlua::Variadic<mlua::Table>| {
                this.value_table = lua_table_to_hashmap(tables)
                    .into_iter()
                    .map(|(k, v)| (k, v as i32))
                    .collect();
                Ok(())
            },
        );
        methods.add_method_mut(
            "map",
            |lua, this, (pattern, func): (mlua::String, mlua::Function)| {
                let key_combo: KeyCombo = parse_keycombo(
                    &pattern.to_str()?.to_string(),
                    &this.code_table,
                    &this.value_table,
                )?;
                let reg_func = lua.create_registry_value(func)?;

                // HotMap に挿入
                this.hotmap.insert(key_combo, reg_func);

                println!("{:?}", this.hotmap);

                Ok(())
            },
        );
        methods.add_method_mut("listen", |lua, this, ()| {
            this.device.listen(lua, &this.hotmap);
            Ok(())
        });
        methods.add_method_mut("grab", |_, this, ()| {
            this.device.grab()?;
            Ok(())
        });
        methods.add_method_mut("ungrab", |_, this, ()| {
            this.device.ungrab()?;
            Ok(())
        });
        methods.add_method_mut("send", |_, this, pattern: mlua::String| {
            this.device.sender().send(&parse_keycombo(
                &pattern.to_str()?.to_string(),
                &this.code_table,
                &this.value_table,
            )?)?;

            Ok(())
        });
    }
}

fn lua_table_to_hashmap(tables: mlua::Variadic<mlua::Table>) -> HashMap<String, mlua::Number> {
    let mut out = HashMap::new();
    for table in tables {
        for pair in table.pairs::<String, mlua::Number>() {
            if let Ok((key, value)) = pair {
                out.insert(key, value);
            }
        }
    }
    out
}

/// 入力文字列を HashMap で解釈して KeyCombo に変換
pub fn parse_keycombo(
    pattern: &String,
    codes: &HashMap<String, KeyCode>,
    values: &HashMap<String, i32>,
) -> io::Result<KeyCombo> {
    let mut keys: HashSet<KeyEvent> = HashSet::new();
    let key_vec = split_pattern(pattern);

    for key in key_vec {
        let value = get_value(&key, values)?;
        let code = codes
            .get(&remove_value_identifier(&key, values)?)
            .expect("not found")
            .code();

        let input_ev = InputEvent::new(EventType::KEY.0, code, value);
        keys.insert(KeyEvent::new(&input_ev));
    }

    Ok(KeyCombo::new(keys))
}

fn get_value(key: &String, values: &HashMap<String, i32>) -> io::Result<i32> {
    let mut matching_list: Vec<i32> = Vec::new();
    for (k, v) in values {
        let reg = Regex::new(k).expect("faild to creating regex");
        if reg.is_match(key) {
            matching_list.insert(matching_list.len(), v.clone());
        }
    }
    match matching_list.len() {
        0 => Ok(KEY_DOWN),
        1 => Ok(matching_list.get(0).unwrap().clone()),
        _ => Err(Error::new(ErrorKind::Other, "multiple regex matching.")),
    }
}

fn remove_value_identifier(key: &String, values: &HashMap<String, i32>) -> io::Result<String> {
    let mut matching_list: Vec<String> = Vec::new();
    for (k, _) in values {
        let reg = Regex::new(&k).expect("faild to creating regex");
        if reg.is_match(key) {
            if let Some(captures) = reg.captures(key) {
                if let Some(cap) = captures.get(1) {
                    matching_list.insert(matching_list.len(), cap.as_str().to_string());
                }
            }
        }
    }
    match matching_list.len() {
        0 => Ok(key.clone()),
        1 => Ok(matching_list.get(0).unwrap().clone()),
        _ => Err(Error::new(ErrorKind::Other, "multiple regex matching.")),
    }
}

fn split_pattern(pattern: &String) -> Vec<String> {
    let out: Vec<String> = pattern
        .split('+')
        .flat_map(|chunk| {
            chunk
                .split_inclusive('-')
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .collect();
    out
}

pub fn call_lua_function(lua: &Lua, reg_key: &mlua::RegistryKey) {
    // get and exec Lua Function from RegistryKey
    let lua_func: mlua::Function = lua.registry_value(reg_key).unwrap();
    lua_func.call::<_, ()>(()).unwrap();
}
