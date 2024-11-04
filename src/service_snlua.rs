use std::sync::{Arc, Mutex};
use std::thread;
use std::{fs, io, path};
use std::path::{Path, PathBuf};
use std::env;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use mlua::{ffi, Chunk, FromLua, Function, Lua, MetaMethod, Result, UserData, UserDataMethods, Value, Variadic};

use crate::rsknet_mq::RuskynetMsg;
use crate::rsknet_server::{RskynetContext, rsknet_send};
use crate::rsknet_global::{to_cstr};

pub struct RsnLua{
    lua_main:Option<Lua>,
    rsknet_ctx:Option<Arc<Mutex<RskynetContext>>>,
}

pub fn _cb(ctx:&mut RskynetContext, session:u32, source:u32, data:Vec<u32>) -> Result<()>{
    println!("from _cb _cb _cb");



    Ok(())
}


pub fn launch_cb(ctx:&mut RskynetContext, session:u32, source:u32, data:Vec<u32>) -> Result<()>{
    let thread_id = thread::current().id();
    println!("launch_cb in thread {thread_id:?} {data:?} begin");
    let rsn_lua = ctx.instance.clone();
    let lua = (*rsn_lua.lock().unwrap()).lua_main.take().unwrap();
    // load bootstrap.lua
    let globals = lua.globals();
    globals.set("LUA_SERVICE", "../service/?.lua")?;

    // require rsknet_core lib
    let callback = lua.create_function(|lua: &Lua, a:Value| {
        unsafe{
            lua.exec_raw((a),|state|{
                let n = ffi::lua_gettop(state);
                println!("num of top {}", n);
                println!("check type, {:?}", ffi::luaL_checktype(state, 1, ffi::LUA_TFUNCTION));

                ffi::lua_getfield(state, ffi::LUA_REGISTRYINDEX, to_cstr("skynet_context"));
                // let ctx = ffi::lua_touserdata(state, ffi::lua_upvalueindex(1)) as *mut RskynetContext;
                let ctx = ffi::lua_touserdata(state, -1) as *mut RskynetContext;
                //let cb_ud = ffi::lua_newthread(state);
                //ffi::lua_xmove(state, cb_ud, 1);

                (*ctx).cb = Some(_cb);
                //(*ctx).cb_userdata = Some(Arc::new(Mutex::new(*cb_ud)));
            })
        }?;
        Ok(1)
        // let state = a.state();
        // println!("in calback");
        // let ctx = ffi::lua_touserdata((lua, ffi::lua_upvalueindex(1)) as *mut RskynetContext;
        //  //ffi::lua_newthread(lua);
        // let a =  lua.create_thread({}).unwrap();
    })?;
    globals.set("rsknet_core_callback", callback)?;
    
    unsafe {
        let load_file = "lualib/loader.lua";
        let service_name = ("bootstrap.lua", load_file);
        let ctx_ptr = ptr::from_mut(ctx);
        lua.exec_raw(service_name, |state| {
            let n = ffi::lua_gettop(state);

            ffi::lua_pushlightuserdata(state, ctx_ptr as *mut c_void);
            ffi::lua_setfield(state, ffi::LUA_REGISTRYINDEX, to_cstr("skynet_context"));

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
    println!("launch_cb in thread {thread_id:?} {data:?} end");
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

    pub fn init(&mut self, rsknet_ctx:Arc<Mutex<RskynetContext>>) {
        self.rsknet_ctx = Some(rsknet_ctx.clone());
        (*rsknet_ctx.lock().unwrap()).cb = Some(launch_cb);
        let handle_id =(*rsknet_ctx.lock().unwrap()).handle; 
        rsknet_send(rsknet_ctx.clone(), handle_id, handle_id, 0, vec![777]);
    }

}