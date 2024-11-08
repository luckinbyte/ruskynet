use std::sync::{Arc, Mutex};
use std::thread;
use std::path::{Path, PathBuf};
use std::ffi::{CStr, CString};
use std::str;

use mlua::{ffi::{self, *}, Lua, Result, Value};

use crate::rsknet_server::{RskynetContext};
use crate::rsknet_global::{to_cstr, LUACBFUNSTR, RSKNETCTXSTR};
use crate::rsknet_socket::{rsknet_socket_listen};


pub fn luaopen_rsknet_socket(lua:&Lua) -> Result<()>{
    let globals = lua.globals();

    let lua_listen_fun = lua.create_function(|lua: &Lua, (host, port):(Value, Value)| {
        let res:Value = unsafe{
            lua.exec_raw((host, port),|state|{
                let host = ffi::lua_tostring(state, 1);
                let host = CStr::from_ptr(host).to_string_lossy().to_string();
                let port = ffi::lua_tointeger(state, 2) as u32;

                ffi::lua_getfield(state, ffi::LUA_REGISTRYINDEX, to_cstr(RSKNETCTXSTR));
                let ctx = ffi::lua_touserdata(state, -1) as *mut RskynetContext;
               
                let id = rsknet_socket_listen((*ctx).handle, host, port);
                ffi::lua_pop(state, 3);
                lua_pushinteger(state, id as i64);
            })
        }?;
        Ok(res)
    })?;
    globals.set("rsknet_socket_listen", lua_listen_fun)?;

    Ok(())
}