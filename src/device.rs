use std::fs;
use std::io;
use std::collections::HashMap;
use evdev::Device;

fn list_input_devices() -> io::Result<Vec<Device>> {
    let vec_devices = fs::read_dir("/dev/input/")?
        .filter_map(|entry| entry.ok())
        .filter(|e| e.path().to_string_lossy().starts_with("/dev/input/event"))
        .filter_map(|e| Device::open(&e.path()).ok())
        .collect::<Vec<_>>();
    Ok(vec_devices)
}

fn group_devices(list: Vec<Device>) -> Vec<Vec<Device>> {
    list.into_iter()
        .fold(HashMap::new(), |mut map, device| {
            let id = device.input_id();
            let key = (
                id.vendor(),
                id.product(),
            );
            map.entry(key).or_insert_with(Vec::new).push(device);
            map
        })
        .into_values()
        .collect()
}

fn find_first_group_by_name(groups: Vec<Vec<Device>>, target: &str) -> Option<Vec<Device>> {
    groups
        .into_iter()
        .find(|group|
            group.iter().any(|dev|
                dev.name().map_or(false, |n| n.contains(target))
            )
        )
}

pub fn find_device_by_name(target: &str) -> Option<Vec<Device>> {
    let device_list = list_input_devices().unwrap();
    let grouped_devices = group_devices(device_list);

    find_first_group_by_name(grouped_devices, target)
}

pub fn print_device_list(list: &[Device]) {
    list.iter().for_each(|dev| {
        println!(
            "  name: {:?}, vendor: {:04x}, product: {:04x}",
            dev.name(),
            dev.input_id().vendor(),
            dev.input_id().product(),
        );
    });
}

