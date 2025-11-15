use crate::hotmap::*;
use evdev::*;
use std::io;

pub struct VirtualDevice {
    vdev: uinput::VirtualDevice,
}

impl VirtualDevice {
    pub fn new(base_device: &Device) -> io::Result<Self> {
        let vdev = uinput::VirtualDevice::builder()?
            .name(&("ouka-".to_string() + base_device.name().unwrap()))
            .with_keys(base_device.supported_keys().unwrap())
            .unwrap()
            .build()
            .unwrap();
        Ok(Self { vdev })
    }

    pub fn send(&mut self, key_combo: &KeyCombo) -> io::Result<()> {
        // key down
        let down_events: Vec<InputEvent> = key_combo.keys().iter().map(|key_ev| InputEvent::new(EventType::KEY.0, key_ev.code().code(), KEY_DOWN)).collect();
        self.vdev.emit(&down_events)?;
        // key up
        let up_events: Vec<InputEvent> = down_events.iter().map(|ev| InputEvent::new(ev.event_type().0, ev.code(), KEY_UP)).collect();
        self.vdev.emit(&up_events)?;
        Ok(())
    }

    pub fn emit(&mut self, events: &[InputEvent]) -> io::Result<()> {
        self.vdev.emit(events)?;
        Ok(())
    }
}

