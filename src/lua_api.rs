use mlua::{Lua, Result};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct RemapTable {
    pub simple: HashMap<String, String>,
}

/// LuaインタープリタとAPIの初期化
pub fn init_bind(lua: &Lua, remap_table: Arc<Mutex<RemapTable>>) -> Result<()> {
    let remap_table_clone = Arc::clone(&remap_table);
    let bind = lua.create_function(move |_, (from, to): (String, String)| {
        let mut table = remap_table_clone.lock().unwrap();
        table.simple.insert(from, to);
        Ok(())
    })?;
    lua.globals().set("bind", bind)?;
    Ok(())
}
