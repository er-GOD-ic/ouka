use dotenv::dotenv;
use std::env;
use std::process;

mod device;
mod lua_api;

fn main() {
    dotenv().ok();

    let ouka_conf = env::var("OUKA_CONF").expect("Faild to load env var");
    print!("OUKA_CONF: {}\n", &ouka_conf);
    let lua = lua_api::load_lua(&ouka_conf);

    // This is the name of global variable. This have to set in your lua file.
    let key = "Device";

    print!("Device:{}\n",&lua_api::get_device_name(&lua, key));

    // user needs to 
    let devices = device::find_device_by_name(&lua_api::get_device_name(&lua, key));
    if devices.is_none() {
        eprintln!("The target device cannot be resolved.");
        process::exit(1);
    }
    println!("=== Target device ===");
    device::print_device_list(devices.as_deref().unwrap_or(&[]));
}
