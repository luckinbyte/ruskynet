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
use mlua::{ffi, ffi::*,  Chunk, FromLua, Function, Lua, MetaMethod, Result, 
            Table, UserData, UserDataMethods, Value, Variadic};

use crate::rsknet_mq::RuskynetMsg;
use crate::rsknet_server::{RskynetContext, rsknet_send};
use crate::rsknet_global::{to_cstr, LUACBFUNSTR, RSKNETCTXSTR};

pub struct RsnLua{
    lua_main:Option<Lua>,
    rsknet_ctx:Option<Arc<Mutex<RskynetContext>>>,
}

pub fn _cb(ctx:&mut RskynetContext, proto_type:u32, data:Vec<u8>, session:u32, source:u32) -> Result<()>{
    let rsn_lua = ctx.instance.clone();
    let lua = (*rsn_lua.lock().unwrap()).lua_main.take().unwrap();

    let data = lua.pack(data)?;
    println!("in _cb data:{:?}", data);
    // let lua_cb_fun:Value = lua.named_registry_value(LUACBFUNSTR)?;
    // println!("in cb cb_fun:{:?}", lua_cb_fun);
    unsafe{
        lua.exec_raw((1, proto_type, data, session, source), |state|{
            ffi::lua_getfield(state, ffi::LUA_REGISTRYINDEX, to_cstr(LUACBFUNSTR));
            ffi::lua_replace(state, 1);

            let n = ffi::lua_gettop(state);
            // println!("in cb {:?}", n);
            ffi::lua_call(state, 4, 0);
        })
    }?;
    (*rsn_lua.lock().unwrap()).lua_main = Some(lua);

    Ok(())
}


pub fn launch_cb(ctx:&mut RskynetContext, proto_type:u32, data:Vec<u8>, session:u32, source:u32) -> Result<()>{
    let thread_id = thread::current().id();
    println!("launch_cb in thread {thread_id:?} {:?} {data:?} begin", ctx.handle);
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

                ffi::lua_setfield(state, ffi::LUA_REGISTRYINDEX, to_cstr(LUACBFUNSTR));
                //ffi::lua_settop(state,1);

                ffi::lua_getfield(state, ffi::LUA_REGISTRYINDEX, to_cstr(RSKNETCTXSTR));
                //let ctx = ffi::lua_touserdata(state, ffi::lua_upvalueindex(1)) as *mut RskynetContext;
                let ctx = ffi::lua_touserdata(state, -1) as *mut RskynetContext;
                //let cb_ud = ffi::lua_newthread(state);
                //ffi::lua_xmove(state, cb_ud, 1);

                // println!("get ptr:{:?}, {:?}, {:?}", ctx, to_cstr(RSKNETCTXSTR), to_cstr(LUACBFUNSTR));
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

    let command_fun = lua.create_function(|lua: &Lua, (a, b):(Value, Value) | {
        let res:Value = unsafe{
            lua.exec_raw((a, b),|state|{
                ffi::lua_getfield(state, ffi::LUA_REGISTRYINDEX, to_cstr(RSKNETCTXSTR));
                let ctx = ffi::lua_touserdata(state, -1) as *mut RskynetContext;

                let cmd = ffi::lua_tostring(state, 1);
                let cmd = CStr::from_ptr(cmd).to_string_lossy().to_string();
                let parm = ffi::lua_tostring(state, 2);
                let parm = CStr::from_ptr(parm).to_string_lossy().to_string();
                let result = (*ctx).rsknet_command(cmd, parm);
                ffi::lua_pop(state, 3);
                match result{
                    None => {
                        ffi::lua_pushnil(state);
                    },
                    Some(res) =>{
                        ffi::lua_pushstring(state, CString::new(res).unwrap().as_ptr());
                    }
                }
            })
        }?;
        return Ok(res);
    })?;
    globals.set("rsknet_core_command", command_fun)?;

    let send_fun = lua.create_function(|lua: &Lua, (des, ptype, session, msg):(Value, Value, Value, Value) | {
        let res:Value = unsafe{
            lua.exec_raw((des, ptype, session, msg),|state|{
                ffi::lua_getfield(state, ffi::LUA_REGISTRYINDEX, to_cstr(RSKNETCTXSTR));
                let ctx = ffi::lua_touserdata(state, -1) as *mut RskynetContext;
                //let n = ffi::lua_gettop(state);

                let des = lua_tointeger(state, 1) as u32;
                let ptype:u32 = lua_tointeger(state, 2) as u32;
                let session = lua_tointeger(state, 3) as u32;
                let data = lua_tostring(state, 4);     
                let data = CStr::from_ptr(data).to_string_lossy().to_string();  
                println!("send fun {data}");      

                let new_session = (*ctx).rsknet_send(des, ptype, session, data);
                lua_pushinteger(state, new_session as i64);
            })
        }?;
        return Ok(res);
    })?;
    globals.set("rsknet_core_send", send_fun)?;

    let lua_pack_fun = lua.create_function(|lua: &Lua, (tt):(Value) | {
        match tt{
            Value::Table(tt) =>{
                if let Ok(ser) = serde_json::to_string(&tt){
                    return Ok(ser);
                }else{
                    return Ok("".to_string())
                }
            },
            _ => {
                let tt_str = tt.as_string_lossy().unwrap();
                return Ok(tt_str)
            }
        }
    })?;
    globals.set("rsknet_core_luapack", lua_pack_fun)?;

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

    pub fn init(&mut self, rsknet_ctx:Arc<Mutex<RskynetContext>>, arg:&str) {
        self.rsknet_ctx = Some(rsknet_ctx.clone());
        (*rsknet_ctx.lock().unwrap()).cb = Some(launch_cb);
        let handle_id =(*rsknet_ctx.lock().unwrap()).handle; 
        rsknet_send(rsknet_ctx.clone(), handle_id, handle_id, 0, 0, arg.as_bytes().to_vec());
    }

}