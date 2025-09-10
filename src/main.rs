mod lua_api;
mod device;

fn main() {
    let devices = device::find_device_by_name("Keychron");
    println!("=== device list matching ===");
    device::print_device_list(devices.as_deref().unwrap_or(&[]));
}
