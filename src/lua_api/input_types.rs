use bitflags::bitflags;
use evdev::*;
use itertools::Itertools;
use mlua::{Lua, RegistryKey, Table, Value};
use regex::Regex;
use std::collections::HashMap;

use crate::lua_api::input_types;

pub type KeyCode = u16;

bitflags! {
    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    struct ModMask: u8 {
        const LCTRL  = 0b1000_0000;
        const LSHIFT = 0b0100_0000;
        const LALT   = 0b0010_0000;
        const LMETA  = 0b0001_0000;
        const RCTRL  = 0b0000_1000;
        const RSHIFT = 0b0000_0100;
        const RALT   = 0b0000_0010;
        const RMETA  = 0b0000_0001;
    }
}

fn keycode_to_modbits(keycode: &KeyCode) -> Result<ModMask, &KeyCode> {
    match keycode {
        29 => Ok(ModMask::LCTRL),
        97 => Ok(ModMask::RCTRL),
        42 => Ok(ModMask::LSHIFT),
        54 => Ok(ModMask::RSHIFT),
        56 => Ok(ModMask::LALT),
        100 => Ok(ModMask::RALT),
        125 => Ok(ModMask::LMETA),
        126 => Ok(ModMask::RMETA),
        _ => Err(keycode),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum State {
    /// 押された瞬間（デフォルト、接頭子なし）
    Down,
    /// 押している間（'_'）
    Held,
    /// 離れた瞬間（'^'）
    Up,
    /// タイムアウト指定 (ms)
    Timeout(u64),
}

pub fn ev_value_to_state(value: i32) -> Option<State> {
    match value {
        0 => Some(State::Up),
        1 => Some(State::Down),
        2 => Some(State::Held),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hotkey {
    pub state: State,
    pub key_codes: Vec<KeyCode>,
    pub mods: ModMask,
}

fn lua_table_to_hashmap(table: Table) -> HashMap<String, input_types::KeyCode> {
    table
        .pairs::<Value, Value>()
        .filter_map(|pair| {
            // 成功していなければ無視
            let (key, value) = pair.ok()?;
            // キーと値を文字列に変換できる場合のみ Some((k,v))
            if let (Value::String(k), Value::Integer(v)) = (key, value) {
                Some((k.to_str().ok()?.to_string(), v as input_types::KeyCode))
            } else {
                None
            }
        })
        .collect()
}

fn parse_state(state: &str) -> State {
    match state.chars().next() {
        Some('^') => State::Up,
        Some('_') => State::Held,
        Some('(') => {
            if let Some(end) = state.find(')') {
                let num_str = &state[1..end];
                State::Timeout(num_str.parse::<u64>().unwrap())
            } else {
                panic!("Error occerd while parcing Timeout! Are you missing the \')\'?")
            }
        }
        _ => State::Up,
    }
}

fn parse_mods(mods: &str, modcodes: &HashMap<String, u16>) -> ModMask {
    mods.split_inclusive('-')
        .map(str::trim)
        .filter_map(|m| modcodes.get(&m.to_lowercase()))
        .map(|keycode| keycode_to_modbits(keycode).ok())
        .fold(ModMask::empty(), |acc, flag| {
            acc | flag.unwrap_or(ModMask::empty())
        })
}

fn parse_keys(keys: &str, keycodes: &HashMap<String, u16>) -> Vec<KeyCode> {
    keys.split('+')
        .map(str::trim)
        .filter_map(|k| keycodes.get(&k.to_lowercase()).copied())
        .collect()
}

pub fn parse_hotkey(lua: &Lua, reg: &RegistryKey, pattern: &str) -> Hotkey {
    // Lua レジストリからテーブルを取り出す
    let table: Table = lua
        .registry_value(reg)
        .map_err(|e| format!("failed to get registry table: {}", e))
        .unwrap();

    // mods テーブルを取得（なければ空テーブル扱い）
    let mods_table: Option<Table> = match table.get("mods") {
        Ok(t) => Some(t),
        Err(_) => None,
    };

    // Lua テーブルをHashmapにパース
    let keycodes = lua_table_to_hashmap(table);
    let modcodes = lua_table_to_hashmap(mods_table.unwrap_or_else(|| lua.create_table().unwrap()));

    // state/ mods/ keys にパターンを分解
    // それぞれの正規表現を作成・取得
    let state_re = Regex::new(r"^(?:\^|_|(\(\d+\)))").unwrap();
    let mods_re = Regex::new(r"^(?:\^|_|(\(\d+\)))?(.*-).*").unwrap();
    let keys_re = Regex::new(r"^(?:\^|_|\(\d+\))?.*-(.*)$").unwrap();

    let state = if let Some(caps) = state_re.captures(pattern) {
        if let Some(m) = caps.get(1) {
            m.as_str()
        } else {
            &pattern[0..1] // ^ または _ の場合
        }
    } else {
        ""
    };
    println!("state:{}", state);

    let mods = if let Some(caps) = mods_re.captures(pattern) {
        caps.get(2).map(|m| m.as_str()).unwrap_or("")
    } else {
        ""
    };
    println!("mods:{}", mods);

    let keys = if let Some(caps) = keys_re.captures(pattern) {
        caps.get(1).map(|m| m.as_str()).unwrap_or("")
    } else {
        ""
    };
    println!("keys:{}", keys);

    // それぞれの値をstructのフィールドに翻訳
    let field_state = parse_state(state);
    let field_mods = parse_mods(mods, &modcodes);
    let field_keys = parse_keys(keys, &keycodes);
    println!("parsed state:{:?}", field_state);
    println!("parsed mods:{:?}", field_mods);
    println!("parsed keys:{:?}", field_keys);

    if field_keys.is_empty() {
        panic!("primary key is not set! can not assign map!: {}", pattern);
    }
    Hotkey {
        state: field_state,
        mods: field_mods,
        key_codes: field_keys,
    }
}

/*
fn get_hotkey(DeviceDa) {
    
}
*/
