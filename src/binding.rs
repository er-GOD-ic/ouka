use crate::device::*;
use crate::hotmap::*;
use evdev::InputEvent;
use evdev::{EventType, KeyCode};
use std::collections::{HashMap, HashSet};

pub struct Binding {
    device: DeviceHandler,
    code_table: HashMap<String, KeyCode>,
    value_table: HashMap<char, i32>,
    hotmap: HotMap,
}

impl Binding {
    pub fn new(device: DeviceHandler) -> Self {
        Self {
            device,
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
                    .map(|(k, v)| {
                        print!("{:?}", k);
                        if k.len() > 1 {
                            panic!("value key has to be 1 char:{}", k);
                        }
                        let c = k.chars().next();
                        println!("{:?}", c);
                        (c.unwrap(), v as i32)
                    })
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
                );
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
    values: &HashMap<char, i32>,
) -> KeyCombo {
    let mut keys: HashSet<KeyEvent> = HashSet::new();
    let key_vec = split_pattern(pattern);

    for key in key_vec {
        let value = get_value(&key, values);
        let code = codes.get(&remove_value_identifier(&key, values)).expect("not found").code();

        let input_ev = InputEvent::new(EventType::KEY.0, code, value);
        keys.insert(KeyEvent::new(&input_ev));
    }

    KeyCombo::new(keys)
}

fn get_value(key: &String, values: &HashMap<char, i32>) -> i32 {
    match values.get(&key.chars().next().unwrap()) {
        Some(t) => t.clone(),
        _ => {
            let out;
            if key.chars().last().unwrap() == '-' {
                out = KEY_HELD;
            } else {
                out = KEY_DOWN;
            }
            out
        }
    }
}

fn remove_value_identifier(key: &String, values: &HashMap<char, i32>) -> String {
    if values.get(&key.chars().next().unwrap()).is_some() {
        key.chars().skip(1).collect()
    } else {
        key.clone()
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
