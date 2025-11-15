use crate::{binding::Binding, device::*};
use mlua::*;
use std::path::Path;

pub fn register_api(lua: &Lua) -> Result<()> {
    let ouka = lua.create_table()?;
    // get device by id
    {
        let get_device_by_id = lua.create_function(|_, str: mlua::String| {
            let path = Path::new(str.to_str()?).to_path_buf();
            let device_handler = DeviceHandler::new(path).map_err(|e| mlua::Error::external(e))?;
            let out = Binding::new(device_handler);
            Ok(out)
        })?;
        ouka.set("getDeviceById", get_device_by_id)?;
    }
    // kill process
    {
        let kill = lua.create_function(|_, ()| -> Result<()> {
            std::process::exit(0);
        })?;
        ouka.set("kill", kill)?;
    }
    lua.globals().set("ouka", ouka)?;
    Ok(())
}
