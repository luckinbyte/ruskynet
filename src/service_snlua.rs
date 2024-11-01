use std::sync::{Arc, Mutex};

use mlua::{FromLua, Function, Lua, MetaMethod, Result, UserData, UserDataMethods, Value, Variadic};

use crate::rsknet_server::{RskynetContext, rsknet_send};

pub struct RsnLua{
    lua_main:Lua,
    rsknet_ctx:Option<Arc<Mutex<RskynetContext>>>,
}

pub fn launch_cb(ctx:Arc<Mutex<RskynetContext>>, lua:Arc<Mutex<Lua>>, session:u32, source:u32, data:Vec<u32>) -> u32{

    return 0
}

impl RsnLua{
    pub fn new() -> Self{
        let lua_main = Lua::new();
        return RsnLua{
            lua_main,
            rsknet_ctx:None
        }
    }

    pub fn init(&mut self, rsknet_ctx:Arc<Mutex<RskynetContext>>) {
        self.rsknet_ctx = Some(rsknet_ctx.clone());
        (*rsknet_ctx.lock().unwrap()).cb = launch_cb;
        let handle_id =(*rsknet_ctx.lock().unwrap()).handle; 
        rsknet_send(rsknet_ctx.clone(), handle_id, handle_id, 0, vec![777]);
    }
}