use crate::hotmap::*;
use evdev::{AttributeSetRef, Device, EventType, InputEvent, KeyCode};
use mlua::prelude::*;
use std::collections::HashSet;
use std::io;
use std::path::PathBuf;

pub struct DeviceHandler {
    device: Device,
}

impl DeviceHandler {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let device = Device::open(&path)?;
        Ok(Self { device })
    }

    pub fn listen(&mut self, lua: &Lua, map: &HotMap) {
        loop {
            let events: Vec<InputEvent> = self.device.fetch_events().unwrap().collect();
            let pressed: &AttributeSetRef<KeyCode> = self.device.cached_state().key_vals().unwrap();
            let key_combo = events_to_keycombo(events, pressed);
            println!("combo: {:?}", key_combo);
            if let Some(reg_key) = map.get(&key_combo) {
                // RegistryKey から Lua の関数を取得
                let lua_func: LuaFunction = lua.registry_value(reg_key).unwrap();
                lua_func.call::<_, ()>(()).unwrap();
            }
        }
    }
}

fn events_to_keycombo(events: Vec<InputEvent>, pressed: &AttributeSetRef<KeyCode>) -> KeyCombo {
    let mut key_events: HashSet<KeyEvent> = HashSet::new();
    let filterd_events = events.iter().filter(|ev| ev.event_type() == EventType::KEY);
    // 押下中キーを取得
    for keycode in pressed.iter() {
        key_events.insert(KeyEvent::new(&InputEvent::new(
            EventType::KEY.0,
            keycode.code(),
            crate::hotmap::KEY_HELD,
        )));
    }

    // 現在のイベントで上書き
    for ev in filterd_events {
        key_events.insert(KeyEvent::new(&InputEvent::new(
            ev.event_type().0,
            ev.code(),
            ev.value(),
        )));
    }

    KeyCombo::new(key_events)
}
