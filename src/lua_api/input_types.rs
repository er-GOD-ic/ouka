use bitflags::bitflags;
use std::collections::BTreeSet;

pub type KeyCode = u16;
pub type ModMask = u8;

bitflags! {
    struct ModKeys: ModMask {
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hotkey {
    pub state: State,
    pub key_code: KeyCode,
    pub mods: ModMask,
}

pub struct Chord {
    pub keys: BTreeSet<Hotkey>,
}

/*
impl fmt::Display for Hotkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    }
}
*/
