use crate::binding::call_lua_function;
use crate::hotmap::*;
use crate::virtual_device::*;
use evdev::{AttributeSetRef, Device, EventType, InputEvent, KeyCode};
use mlua::prelude::*;
use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct DeviceHandler {
    device: Device,
    sender: VirtualDevice,
    grabbing: bool,
}

impl DeviceHandler {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let device = Device::open(&path)?;
        let sender = VirtualDevice::new(&device)?;
        let grabbing = false;
        Ok(Self {
            device,
            sender,
            grabbing,
        })
    }

    pub fn listen(&mut self, lua: &Lua, map: &HotMap) {
        loop {
            let events: Vec<InputEvent> = self.device.fetch_events().unwrap().collect();
            let pressed: &AttributeSetRef<KeyCode> = self.device.cached_state().key_vals().unwrap();
            let key_combo = events_to_keycombo(&events, pressed);
            if let Some(reg_key) = map.get(&key_combo) {
                call_lua_function(lua, reg_key);
            } else {
                // when grabbing device, pass-through input
                if self.grabbing && !is_related_event_registered(&key_combo, map) {
                    self.sender.emit(&events).unwrap();
                }
            }
        }
    }
    pub fn grab(&mut self) -> io::Result<()> {
        self.device.grab()?;
        self.grabbing = true;
        Ok(())
    }
    pub fn ungrab(&mut self) -> io::Result<()> {
        self.device.ungrab()?;
        self.grabbing = true;
        Ok(())
    }

    pub fn sender(&mut self) -> &mut VirtualDevice {
        &mut self.sender
    }
}

fn events_to_keycombo(events: &Vec<InputEvent>, pressed: &AttributeSetRef<KeyCode>) -> KeyCombo {
    let mut key_events: HashSet<KeyEvent> = HashSet::new();
    let filterd_events: Vec<&InputEvent> = events
        .iter()
        .filter(|ev| ev.event_type() == EventType::KEY)
        .collect();
    // 押下中キーを取得
    for keycode in pressed.iter() {
        // 現在発生したイベントを除く
        if filterd_events.iter().any(|ev| ev.code() == keycode.code()) {
            continue;
        }
        key_events.insert(KeyEvent::new(&InputEvent::new(
            EventType::KEY.0,
            keycode.code(),
            crate::hotmap::KEY_HELD,
        )));
    }

    // イベントを取得
    for ev in filterd_events {
        key_events.insert(KeyEvent::new(&InputEvent::new(
            ev.event_type().0,
            ev.code(),
            ev.value(),
        )));
    }

    KeyCombo::new(key_events)
}

fn is_related_event_registered(key_combo: &KeyCombo, map: &HotMap) -> bool {
    let mut codes: Vec<KeyCode> = key_combo
        .keys()
        .iter()
        .map(|key| key.code().clone())
        .collect();
    codes.sort();
    codes.dedup();

    for (k, _) in map {
        // HotMap 内の code 集合を作る
        let mut reg_codes: Vec<KeyCode> = k
            .keys()
            .iter()
            .map(|key_ev| key_ev.code().clone())
            .collect();
        reg_codes.sort();
        reg_codes.dedup();

        // ④ code 集合が一致すれば「value が違うだけ」と判断
        if codes == reg_codes {
            return true;
        }
    }

    false
}

pub fn listen_device(arc_self: Arc<Mutex<DeviceHandler>>, lua: &Lua, map: &HotMap) {
    loop {
        let mut device_handler = arc_self.lock().unwrap();
        let events: Vec<InputEvent> = device_handler.device.fetch_events().unwrap().collect();
        let pressed: &AttributeSetRef<KeyCode> =
            device_handler.device.cached_state().key_vals().unwrap();
        let key_combo = events_to_keycombo(&events, pressed);
        if let Some(reg_key) = map.get(&key_combo) {
            drop(device_handler);
            call_lua_function(lua, reg_key);
        } else {
            // when grabbing device, pass-through input
            if device_handler.grabbing && !is_related_event_registered(&key_combo, map) {
                device_handler.sender.emit(&events).unwrap();
            }
        }
    }
}
