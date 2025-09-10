mod lua_api;
mod device;

fn main() {
    let lua = lua_api::load_lua();

    // This is the name of global variable. This have to set in your lua file.
    // You can change this to whatever you want to call it. For example; Mouse, Numpad or Keyboard.
    let key = "Device";

    let devices = device::find_device_by_name(&lua_api::device_name(&lua, key));
    println!("=== device list matching ===");
    device::print_device_list(devices.as_deref().unwrap_or(&[]));
}
