use std::fs;
use std::env;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use dotenv::dotenv;
use mlua::{Lua, Result};

pub fn load_lua() -> Lua {
    let lua: Lua = Lua::new();
    dotenv().ok();
    let config_path = env::var("OUKA_CONFIG").unwrap() + "config.lua";
    let code = fs::read_to_string(config_path).expect("ファイル読込失敗");
    lua.load(&code).exec();
    lua
}

pub struct RemapTable {
    pub simple: HashMap<String, String>,
}

/// LuaインタープリタとAPIの初期化
pub fn init_fn_bind(lua: &Lua, remap_table: Arc<Mutex<RemapTable>>) -> Result<()> {
    let remap_table_clone = Arc::clone(&remap_table);
    let bind = lua.create_function(move |_, (from, to): (String, String)| {
        let mut table = remap_table_clone.lock().unwrap();
        table.simple.insert(from, to);
        Ok(())
    })?;
    lua.globals().set("bind", bind)?;
    Ok(())
}

pub fn device_name(lua: &Lua, key: &str) -> String {
    match lua.globals().get::<_, String>(key) {
        Ok(name) => name,
        Err(e) => panic!("Luaのグローバル変数 'Device' が存在しません: {}", e),
    }
}
