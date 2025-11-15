use dotenv::dotenv;
use mlua::Lua;
use std::env;
use std::fs;
use std::path::Path;

mod binding;
mod device;
mod virtual_device;
mod hotmap;
mod lua_function;

// load init.lua from path
pub fn load_lua(lua: &Lua, path: &Path) {
    let code = fs::read_to_string(&path.join("config.lua")).expect("config.luaが読み込めません");
    lua.load(&code)
        .exec()
        .expect("config.luaのロード時にエラーが発生しました");
}

fn main() {
    dotenv().ok();

    // init lua api
    let lua = Lua::new().into_static();

    // for Debugging. Resolve the require path
    lua.load(r#"package.path = package.path .. ';./lua/?.lua;./lua/?/init.lua'"#)
        .exec()
        .unwrap();

    lua_function::register_api(&lua).unwrap();
    load_lua(
        &lua,
        Path::new(&env::var("OUKA_CONF").expect("Faild to load env var: OUKA_CONF")),
    );
}
