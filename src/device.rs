use evdev::Device;
use std::collections::HashMap;
use std::fs;
use std::io;

// list all available devices
fn list_input_devices() -> io::Result<Vec<Device>> {
    let vec_devices = fs::read_dir("/dev/input/")?
        .filter_map(|entry| entry.ok())
        .filter(|e| e.path().to_string_lossy().starts_with("/dev/input/event"))
        .filter_map(|e| Device::open(&e.path()).ok())
        .collect::<Vec<_>>();
    Ok(vec_devices)
}

// group device by vendor, product id
fn group_devices(list: Vec<Device>) -> Vec<Vec<Device>> {
    list.into_iter()
        .fold(HashMap::new(), |mut map, device| {
            let id = device.input_id();
            let key = (id.vendor(), id.product());
            map.entry(key).or_insert_with(Vec::new).push(device);
            map
        })
        .into_values()
        .collect()
}

// search device group from target string
fn find_group_by_name(
    groups: Vec<Vec<Device>>,
    target: &str,
) -> Result<Vec<Device>, Vec<Vec<Device>>> {
    let matches: Vec<Vec<Device>> = groups
        .into_iter()
        .filter(|group| {
            group
                .iter()
                .any(|dev| dev.name().map_or(false, |n| n.contains(target)))
        })
        .collect();

    match matches.len() {
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => Err(matches),
    }
}

pub fn find_device_by_name(target: &str) -> Option<Vec<Device>> {
    let device_list = list_input_devices().expect("Faild to get device list");
    if device_list.len() == 0 {
        println!("No device found.");
        println!("Check that you belong to the input group.");
    }
    let grouped_devices = group_devices(device_list);

    match find_group_by_name(grouped_devices, target) {
        Ok(group) => Some(group),
        Err(matches) => {
            if matches.is_empty() {
                println!("No matching group found for '{}'", target);
            } else {
                println!("Multiple matching groups found for '{}'", target);
                for group in &matches {
                    print_device_list(group);
                }
            }
            None
        }
    }
}

pub fn print_device_list(list: &[Device]) {
    list.iter().for_each(|dev| {
        println!(
            "  name: {}, vendor: {:04x}, product: {:04x}",
            dev.name().unwrap_or("<unknown>"),
            dev.input_id().vendor(),
            dev.input_id().product(),
        );
    });
}
