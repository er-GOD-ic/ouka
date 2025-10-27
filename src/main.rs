use dotenv::dotenv;
use mlua::Lua;
use std::collections::HashMap;
use std::env;
use std::process;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

mod device;
mod lua_api;

fn main() {
    dotenv().ok();

    // init lua api
    let lua = Lua::new();
    let store: lua_api::api::MapStore = Arc::new(Mutex::new(HashMap::new()));
    let id_gen = Arc::new(AtomicU64::new(1));
    lua_api::api::register_api(&lua, store.clone(), id_gen).unwrap();
    lua_api::api::load_lua(&lua, &env::var("OUKA_CONF").expect("Faild to load env var: OUKA_CONF"));

    let key = "Device"; // This is the name of global variable. This have to set in your lua file.

    print!("Device:{}\n", &lua_api::api::get_device_name(&lua, key));

    // user needs to
    let devices = device::find_device_by_name(&lua_api::api::get_device_name(&lua, key));
    if devices.is_none() {
        eprintln!("The target device cannot be resolved.");
        process::exit(1);
    }
    println!("=== Target device ===");
    device::print_device_list(devices.as_deref().unwrap_or(&[]));

    device::listen_device_list(devices.as_deref().unwrap_or(&[]));
}
