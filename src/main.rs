use dotenv::dotenv;
use mlua::Lua;
use std::env;
use std::path::Path;
use std::process;

mod device;
mod lua_api;

fn main() {
    dotenv().ok();

    // init lua api
    let lua = Lua::new().into_static();

    // for Debugging. Resolve the require path
    lua.load(r#"package.path = package.path .. ';./lua/?.lua;./lua/?/init.lua'"#).exec().unwrap();

    lua_api::api::register_api(&lua).unwrap();
    lua_api::api::load_lua(&lua, Path::new(&env::var("OUKA_CONF").expect("Faild to load env var: OUKA_CONF")));

    let key = "Device"; // This is the name of global variable. This have to set in your lua file.

    print!("Device:{}\n", &lua_api::api::get_device_name(&lua, key));
}
