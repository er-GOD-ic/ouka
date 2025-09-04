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

fn group_by_existing_devices(list: Vec<Device>) -> Vec<Vec<Device>> {
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

fn print_device_list(list: &[Device]) {
    list.iter().for_each(|dev| {
        println!(
            "  name: {:?}, vendor: {:04x}, product: {:04x}",
            dev.name(),
            dev.input_id().vendor(),
            dev.input_id().product(),
        );
    });
}

fn main() {
    let device_list = list_input_devices().unwrap();
    let grouped_device_list = group_by_existing_devices(device_list);

    let filtered_list = find_first_group_by_name(grouped_device_list, "Keychron");
    println!("=== device list matching ===");
    print_device_list(filtered_list.as_deref().unwrap_or(&[]));
}
