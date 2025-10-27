use std::collections::BTreeSet;

pub type KeyCode = u16;
pub type KeyValue = u32;
pub type ModMask = u8;

pub const MOD_LCTRL: ModMask = 0b1000_0000;
pub const MOD_LSHIFT: ModMask = 0b0100_0000;
pub const MOD_LALT: ModMask = 0b0010_0000;
pub const MOD_LMETA: ModMask = 0b0001_0000;
pub const MOD_RCTRL: ModMask = 0b0000_1000;
pub const MOD_RSHIFT: ModMask = 0b0000_0100;
pub const MOD_RALT: ModMask = 0b0000_0010;
pub const MOD_RMETA: ModMask = 0b0000_0001;


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

pub struct Hotkey {
    pub state: State,
    pub key_code: KeyCode,
    pub ev_value: KeyValue, // DOWN, HOLD, UP
    pub mods: ModMask,
}

pub struct Chord {
    pub keys: BTreeSet<Hotkey>,
}
