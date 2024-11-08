use std::sync::{Arc, Mutex};
use std::thread;
use std::{fs, io, path};
use std::path::{Path, PathBuf};
use std::env;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::ffi::{CStr, CString};
use std::str;

//use serde_json;
//use serde::{Deserialize, Serialize};
use mlua::{ffi::{self, *}, Chunk, FromLua, Function, Lua, LuaSerdeExt, MetaMethod, Result, Table, UserData, UserDataMethods, Value, Variadic};

use crate::rsknet_mq::RuskynetMsg;
use crate::rsknet_server::{RskynetContext, rsknet_send};
use crate::rsknet_global::{to_cstr, LUACBFUNSTR, RSKNETCTXSTR};
use crate::lua_rsknet;
use crate::lua_socket;

pub struct RsnLua{
    pub lua_main:Option<Lua>,
    rsknet_ctx:Option<Arc<Mutex<RskynetContext>>>,
}

pub fn launch_cb(ctx:&mut RskynetContext, proto_type:u32, data:Vec<u8>, session:u32, source:u32) -> Result<()>{
    let thread_id = thread::current().id();
    // println!("launch_cb in thread {thread_id:?} handle:{:?} {:?} begin", ctx.handle, str::from_utf8(&data).unwrap());
    let rsn_lua = ctx.instance.clone();
    let lua = (*rsn_lua.lock().unwrap()).lua_main.take().unwrap();

    // load bootstrap.lua
    let globals = lua.globals();
    globals.set("LUA_SERVICE", "../service/?.lua")?;
    globals.set("HANDLE_ID", ctx.handle)?;

    lua_rsknet::luaopen_rsknet_core(&lua);
    lua_socket::luaopen_rsknet_socket(&lua);
    
    let arg:Vec<&str> = str::from_utf8(&data).unwrap().split_whitespace().collect();
    unsafe {
        let load_file = "lualib/loader.lua";
        let service_name = (arg[1].to_owned()+".lua", load_file);
        let ctx_ptr = ptr::from_mut(ctx);
        lua.exec_raw(service_name, |state| {
            let n = ffi::lua_gettop(state);

            ffi::lua_pushlightuserdata(state, ctx_ptr as *mut c_void);
            // println!("set ptr:{:?}, {:?}", ctx_ptr, to_cstr(RSKNETCTXSTR));
            ffi::lua_setfield(state, ffi::LUA_REGISTRYINDEX, to_cstr(RSKNETCTXSTR));

            ffi::luaL_loadfile(state, ffi::lua_tostring(state, 2));
            ffi::lua_pushlstring(state, ffi::lua_tostring(state, 1), 13); 
            ffi::lua_call(state, 1, 0);
            ffi::lua_pop(state, n);
        })
    }?;
    //lua.load(r#"return 1"#).eval()?;
    // let path11 = env::current_dir()?;
    // println!("load successful {}", path11.display());
    (*rsn_lua.lock().unwrap()).lua_main = Some(lua);
    // println!("launch_cb in thread {thread_id:?} {data:?} end");
    Ok(())
}

impl RsnLua{
    pub fn new() -> Self{
        let lua_main = Some(Lua::new());
        return RsnLua{
            lua_main,
            rsknet_ctx:None,
        }
    }

    pub fn init(&mut self, rsknet_ctx:Arc<Mutex<RskynetContext>>, arg:&str) {
        self.rsknet_ctx = Some(rsknet_ctx.clone());
        (*rsknet_ctx.lock().unwrap()).cb = Some(launch_cb);
        let handle_id =(*rsknet_ctx.lock().unwrap()).handle; 
        rsknet_send(rsknet_ctx.clone(), handle_id, handle_id, 0, 0, arg.as_bytes().to_vec());
    }

}