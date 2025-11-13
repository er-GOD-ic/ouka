use evdev::*;
use mlua::RegistryKey;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

pub const KEY_UP: i32 = 0;
pub const KEY_DOWN: i32 = 1;
pub const KEY_HELD: i32 = 2;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct KeyEvent {
    code: KeyCode,
    value: i32,
}

impl KeyEvent {
    pub fn new(key: &InputEvent) -> Self {
        Self {
            code: KeyCode(key.code()),
            value: key.value(),
        }
    }

    fn code(&self) -> u16 {
        self.code.code()
    }
}

impl Hash for KeyEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.code.hash(state);
        self.value.hash(state);
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct KeyCombo {
    keys: HashSet<KeyEvent>,
}

impl KeyCombo {
    pub fn new(keys: HashSet<KeyEvent>) -> Self {
        Self{ keys }
    }
}

impl Hash for KeyCombo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // HashSet の内容を順不同で hash
        let mut codes: Vec<_> = self.keys.iter().collect();
        codes.sort_by_key(|k| k.code()); // 順序を固定
        for k in codes {
            k.hash(state);
        }
    }
}

pub type HotMap = HashMap<KeyCombo, RegistryKey>;
