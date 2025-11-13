use crate::device::*;
use crate::hotmap::*;
use evdev::*;
use std::collections::{HashMap, HashSet};

pub struct Binding {
    device: DeviceHandler,
    keycode_table: HashMap<String, KeyCode>,
    hotmap: HotMap,
}

impl Binding {
    pub fn new(device: DeviceHandler) -> Self {
        Self {
            device,
            keycode_table: HashMap::new(),
            hotmap: HotMap::new(),
        }
    }
}

impl mlua::UserData for Binding {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "setKeycodes",
            |_, this, tables: mlua::Variadic<mlua::Table>| {
                this.keycode_table = lua_table_to_hashmap(tables);
                Ok(())
            },
        );
        methods.add_method_mut(
            "map",
            |lua, this, (pattern, func): (mlua::String, mlua::Function)| {
                let key_combo: KeyCombo = parse_keycombo(pattern.to_str()?, &this.keycode_table);
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

fn lua_table_to_hashmap(tables: mlua::Variadic<mlua::Table>) -> HashMap<String, KeyCode> {
    let mut out = HashMap::new();
    for table in tables {
        for pair in table.pairs::<String, u16>() {
            if let Ok((key, value)) = pair {
                out.insert(key, KeyCode(value));
            }
        }
    }
    out
}

/// 入力文字列を HashMap で解釈して KeyCombo に変換
pub fn parse_keycombo(input: &str, table: &HashMap<String, KeyCode>) -> KeyCombo {
    let mut keys = HashSet::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        let mut found = false;
        let chars: Vec<char> = remaining.chars().collect();
        for len in (1..=chars.len()).rev() {
            let substr: String = chars[..len].iter().collect();
            if let Some(&keycode) = table.get(&substr) {
                // KeyCode から KeyEvent を生成して HashSet に追加
                let event = crate::hotmap::KeyEvent::new(&InputEvent::new(
                    EventType::KEY.0,
                    keycode.code(),
                    crate::hotmap::KEY_HELD,
                ));
                keys.insert(event);

                remaining = &remaining[substr.len()..];
                found = true;
                break;
            }
        }
        if !found {
            // 一致しない場合は 1文字消費（またはエラー扱いにする）
            remaining = &remaining[remaining.chars().next().unwrap().len_utf8()..];
        }
    }

    KeyCombo::new(keys)
}
