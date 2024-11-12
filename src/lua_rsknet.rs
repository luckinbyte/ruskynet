use std::sync::{Arc, Mutex};
use std::thread;
use std::path::{Path, PathBuf};
use std::ffi::{CStr, CString};
use std::str;

//use serde_json;
//use serde::{Deserialize, Serialize};
use mlua::{ffi::{self, *}, Lua, Result, Value, LuaSerdeExt};

use crate::rsknet_server::{RskynetContext};
use crate::rsknet_global::{to_cstr, LUACBFUNSTR, RSKNETCTXSTR};

pub fn _cb(ctx:&mut RskynetContext, proto_type:u32, data:Vec<u8>, session:u32, source:u32) -> Result<()>{
    let rsn_lua = ctx.instance.clone();
    let lua = (*rsn_lua.lock().unwrap()).lua_main.take().unwrap();

    println!("handle:{:?} _cb ptype:{proto_type:?} session:{session:?} source:{source:?} data:{:?}", ctx.handle, str::from_utf8(&data).unwrap());
    //let ud = lua.create_ser_any_userdata(data).unwrap();
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

pub fn luaopen_rsknet_core(lua:&Lua) -> Result<()>{
    let globals = lua.globals();
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

                let new_session = (*ctx).rsknet_send(des, ptype, session, data);
                ffi::lua_pop(state, 5);
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

    let lua_unpack_fun = lua
        .create_function(|lua: &Lua, s:Value| {
            //lua.to_value(&serde_json::from_str::<serde_json::Value>(&s).unwrap())
            let json_str = serde_json::to_string_pretty(&s).unwrap();
            Ok(json_str)
        })?;
    globals.set("rsknet_core_luaunpack", lua_unpack_fun)?;

    Ok(())
}