use std::sync::{Arc, Mutex};
use std::thread;
use std::{fs, io, path};
use std::path::{Path, PathBuf};
use std::env;

use mlua::{ffi, Chunk, FromLua, Function, Lua, MetaMethod, Result, UserData, UserDataMethods, Value, Variadic};

use crate::rsknet_mq::RuskynetMsg;
use crate::rsknet_server::{RskynetContext, rsknet_send};

pub struct RsnLua{
    lua_main:Option<Lua>,
    rsknet_ctx:Option<Arc<Mutex<RskynetContext>>>,
}

pub fn launch_cb(ctx:&mut RskynetContext, session:u32, source:u32, data:Vec<u32>) -> Result<()>{
    let thread_id = thread::current().id();
    println!("launch_cb in thread {thread_id:?} {data:?}");
    let rsn_lua = ctx.instance.clone();
    let lua = (*rsn_lua.lock().unwrap()).lua_main.take().unwrap();
    // load bootstrap.lua
    let globals = lua.globals();
    globals.set("LUA_SERVICE", "../service/?.lua")?;

    let mut path_buf = PathBuf::new();
    path_buf.push("lualib");
    path_buf.push("loader.lua");
    let path: &Path = path_buf.as_path();
    //lua.load(path).exec()?;
    
    let n: i32 = unsafe {
        let nums = ("alalala");
        lua.exec_raw(nums, |state| {
            let n = ffi::lua_gettop(state);
            let mut sum = 0;
            for i in 1..=n {
                sum += ffi::lua_tointeger(state, i);
            }
            ffi::lua_pop(state, n);
            ffi::lua_pushinteger(state, sum);
        })
    }?;
    println!("aaaa {n}");
    //lua.load(r#"return 1"#).eval()?;
    // let path11 = env::current_dir()?;
    // println!("load successful {}", path11.display());
    (*rsn_lua.lock().unwrap()).lua_main = Some(lua);
    Ok(())
}

impl RsnLua{
    pub fn new() -> Self{
        let lua_main = Some(Lua::new());
        return RsnLua{
            lua_main,
            rsknet_ctx:None
        }
    }

    pub fn init(&mut self, rsknet_ctx:Arc<Mutex<RskynetContext>>) {
        self.rsknet_ctx = Some(rsknet_ctx.clone());
        (*rsknet_ctx.lock().unwrap()).cb = Some(launch_cb);
        let handle_id =(*rsknet_ctx.lock().unwrap()).handle; 
        rsknet_send(rsknet_ctx.clone(), handle_id, handle_id, 0, vec![777]);
    }

}