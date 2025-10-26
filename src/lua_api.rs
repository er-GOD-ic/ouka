use dotenv::dotenv;
use mlua::Lua;
use std::fs;
use std::path::Path;

// load init.lua from path
pub fn load_lua(path: &str) -> Lua {
    let lua: Lua = Lua::new();
    dotenv().ok();
    let config_path = Path::new(path).join("init.lua");
    let code = fs::read_to_string(&config_path).expect("init.luaが読み込めません");
    lua.load(&code)
        .exec()
        .expect("init.luaのロード時にエラーが発生しました");
    lua
}

// get global var that define device to grab
pub fn get_device_name(lua: &Lua, key: &str) -> String {
    match lua.globals().get::<_, String>(key) {
        Ok(name) => name,
        Err(e) => panic!("Luaのグローバル変数'{}'が存在しません: {}", key, e),
    }
}

