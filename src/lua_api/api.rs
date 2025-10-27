use mlua::{Function, Lua, RegistryKey, Result as LuaResult, Table};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
use std::time::Instant;

// load init.lua from path
pub fn load_lua(lua: &Lua, path: &str) {
    let config_path = Path::new(path).join("init.lua");
    let code = fs::read_to_string(&config_path).expect("init.luaが読み込めません");
    lua.load(&code)
        .exec()
        .expect("init.luaのロード時にエラーが発生しました");
}

// get global var that define device to grab
pub fn get_device_name(lua: &Lua, key: &str) -> String {
    match lua.globals().get::<_, String>(key) {
        Ok(name) => name,
        Err(e) => panic!("Luaのグローバル変数'{}'が存在しません: {}", key, e),
    }
}

pub struct Mapping {
    pub id: u64,
    pub pattern: String,
    pub handler: RegistryKey,
    pub metadata: Option<String>,
    pub created: Instant,
}

pub type MapStore = Arc<Mutex<HashMap<u64, Mapping>>>;

// functions
pub fn register_api(lua: &Lua, store: MapStore, id_gen: Arc<AtomicU64>) -> LuaResult<()> {
    // map(pattern, func, opts)
    {
        let store = store.clone();
        let id_gen = id_gen.clone();
        let map_fn = lua.create_function(
            move |lua_ctx, (pat, func, opts): (String, Function, Option<Table>)| {
                // pattern をパースして内部表現を作る（ここでは stub で文字列をそのまま保持）
                let metadata = if let Some(tbl) = opts {
                    // 例: opts.app
                    match tbl.get::<_, Option<String>>("app") {
                        Ok(v) => v,
                        Err(_) => None,
                    }
                } else {
                    None
                };

                let id = id_gen.fetch_add(1, Ordering::SeqCst);
                let reg = lua_ctx.create_registry_value(func)?;
                let m = Mapping {
                    id,
                    pattern: pat.clone(),
                    handler: reg,
                    metadata,
                    created: Instant::now(),
                };
                store.lock().unwrap().insert(id, m);
                Ok(id)
            },
        )?;
        lua.globals().set("map", map_fn)?;
    }
    Ok(())
}
