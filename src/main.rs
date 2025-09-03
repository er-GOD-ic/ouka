use std::fs;
use std::io;
use evdev::Device;

fn list_input_devices() -> io::Result<Vec<Device>> {
    let vec_devices = fs::read_dir("/dev/input/")?
        .filter_map(|entry| entry.ok())
        .filter(|e| e.path().to_string_lossy().starts_with("/dev/input/event"))
        .filter_map(|e| Device::open(&e.path()).ok())
        .collect::<Vec<_>>();
    Ok(vec_devices)
}

fn main() {
    let devices = list_input_devices();
}
