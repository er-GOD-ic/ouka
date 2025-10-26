use evdev::Device;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::thread;

pub struct DeviceHandler {
    device: Device,
    path: PathBuf,
}

impl DeviceHandler {
    fn open(path: PathBuf) -> std::io::Result<Self> {
        let device = Device::open(&path)?;
        Ok(Self { device, path })
    }
}

// list all available devices
fn list_input_devices() -> io::Result<Vec<DeviceHandler>> {
    let vec_devices = fs::read_dir("/dev/input/")?
        .filter_map(|entry| entry.ok())
        .filter(|e| e.path().to_string_lossy().starts_with("/dev/input/event"))
        .filter_map(|e| DeviceHandler::open(e.path()).ok())
        .collect::<Vec<_>>();
    Ok(vec_devices)
}

// group device by vendor, product id
fn group_devices(list: Vec<DeviceHandler>) -> Vec<Vec<DeviceHandler>> {
    list.into_iter()
        .fold(HashMap::new(), |mut map, device| {
            let id = device.device.input_id();
            let key = (id.vendor(), id.product());
            map.entry(key).or_insert_with(Vec::new).push(device);
            map
        })
        .into_values()
        .collect()
}

// search device group from target string
fn find_group_by_name(
    groups: Vec<Vec<DeviceHandler>>,
    target: &str,
) -> Result<Vec<DeviceHandler>, Vec<Vec<DeviceHandler>>> {
    let matches: Vec<Vec<DeviceHandler>> = groups
        .into_iter()
        .filter(|group| {
            group
                .iter()
                .any(|device| device.device.name().map_or(false, |n| n.contains(target)))
        })
        .collect();

    match matches.len() {
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => Err(matches),
    }
}

pub fn find_device_by_name(target: &str) -> Option<Vec<DeviceHandler>> {
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

pub fn print_device_list(list: &[DeviceHandler]) {
    list.iter().for_each(|device| {
        println!(
            "  name: {}, vendor: {:04x}, product: {:04x}, path: {}",
            device.device.name().unwrap_or("<unknown>"),
            device.device.input_id().vendor(),
            device.device.input_id().product(),
            device.device.physical_path().unwrap_or("<unknown>"),
        );
    });
}

// listen device list
pub fn listen_device_list(list: &[DeviceHandler]) {
    let handles: Vec<_> = list.iter().map(|device| listen(device)).collect();

    // すべてのスレッドをjoin
    for handle in handles {
        handle.join().unwrap();
    }
}

// listen device input
fn listen(device: &DeviceHandler) -> thread::JoinHandle<()> {
    let path = device.path.clone();
    thread::spawn(move || {
        let mut device = match Device::open(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("failed to open device {}: {}", path.display(), e);
                return;
            }
        };

        loop {
            match device.fetch_events() {
                Ok(events) => events
                    .into_iter()
                    .for_each(|ev| println!("event: {:?}", ev)),
                Err(e) => {
                    eprintln!("error reading {}: {}", path.display(), e);
                }
            }
        }
    })
}
