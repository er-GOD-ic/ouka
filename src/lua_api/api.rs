use mlua::{Error, Function, Lua, RegistryKey, Result as LuaResult, Table, Value, Variadic};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
use std::time::Instant;

#[path = "../device.rs"]
mod device;

// load init.lua from path
pub fn load_lua(lua: &Lua, path: &Path) {
    let config_path = path.join("config.lua");
    let code = fs::read_to_string(&config_path).expect("config.luaが読み込めません");
    lua.load(&code)
        .exec()
        .expect("config.luaのロード時にエラーが発生しました");
}

// get global variable that define device to grab
pub fn get_device_name(lua: &Lua, key: &str) -> String {
    match lua.globals().get::<_, String>(key) {
        Ok(name) => name,
        Err(e) => panic!("Luaのグローバル変数'{}'が存在しません: {}", key, e),
    }
}

fn table_to_map_checked<'lua>(table: Table<'lua>) -> mlua::Result<HashMap<String, u16>> {
    // より安全な方法：一度 i64 等で受け取ってから u16 の範囲チェックを行う
    let mut map = HashMap::new();
    for pair in table.pairs::<String, i64>() {
        let (k, v) = pair?;
        if !(0..=u16::MAX as i64).contains(&v) {
            return Err(Error::FromLuaConversionError {
                from: "integer",
                to: "u16",
                message: Some(format!("value {} out of range for u16 at key '{}'", v, k)),
            });
        }
        map.insert(k, v as u16);
    }
    Ok(map)
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
    let ouka = lua.create_table()?;

    // define map
    {
        let marge_table = lua.create_function(|lua, tables: Variadic<Table>| {
            let out = lua.create_table()?;
            for table in tables {
                for pair in table.pairs::<Value, Value>() {
                    let (k, v) = pair?;
                    out.set(k, v)?;
                }
            }
            Ok(out)
        })?;
        ouka.set("margeTable", marge_table)?;
    }

    // get device by name
    {
        let get_device_by_name = lua.create_function(|lua, str: mlua::String| {
            let out = lua.create_table()?;
            device::find_device_by_name(str.to_str()?);
        })?;
        ouka.set("getDeviceByName", get_device_by_name)?;
    }

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
        ouka.set("map", map_fn)?;
    }

    lua.globals().set("ouka", ouka)?;
    Ok(())
}
