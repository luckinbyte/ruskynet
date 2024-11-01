use std::sync::{Arc, Mutex};
use mlua::{FromLua, Function, Lua, MetaMethod, Result, UserData, UserDataMethods, Value, Variadic};

use crate::rsknet_handle::RskynetHandle;
use crate::rsknet_mq::{MessageQueue, RuskynetMsg, GlobalQueue};
use crate::service_snlua::{RsnLua};
use crate::rsknet_global::{get_ctx_by_handle, GLOBALMQ};

pub struct RskynetContext{
    pub instance:Arc<Mutex<RsnLua>>, 
    pub cb:Option<fn(&mut RskynetContext, u32, u32, Vec<u32>)-> u32>,
    session_id:u32,
	pub handle:u32,// uint32_t handle;
    queue:Arc<Mutex<MessageQueue>>,
}

fn context_push(destination:u32, msg:RuskynetMsg) -> u32{
    let ctx = get_ctx_by_handle(destination);
    ctx.lock().unwrap().push_msg(msg);
    return 0;
}

pub fn rsknet_send(ctx:Arc<Mutex<RskynetContext>>, source:u32, destination:u32, session:u32, data:Vec<u32>) -> u32{
    let msg = RuskynetMsg::new(source, session, data);
    context_push(destination, msg); 
    return 1;
}

impl RskynetContext{
    pub fn new(hanlde:Arc<Mutex<RskynetHandle>>) -> Arc<Mutex<RskynetContext>>{
        let queue = Arc::new(Mutex::new(MessageQueue::new()));
        let instance = Arc::new(Mutex::new(RsnLua::new()));

        let ctx = Arc::new(Mutex::new(
            RskynetContext{
                instance:instance.clone(),
                cb:None,
                session_id:1,
                handle:1,
                queue,
            }
        ));
        (*hanlde.lock().unwrap()).handle_register(ctx.clone());
        (*instance.lock().unwrap()).init(ctx.clone());

        return ctx;
    }

    pub fn set_handle(&mut self, handle:u32){
        self.handle = handle;
        self.queue.lock().unwrap().set_handle(handle);
    }

    pub fn push_msg(&mut self, msg:RuskynetMsg) {
        (*self.queue.lock().unwrap()).push_msg(msg);
        if !self.queue.lock().unwrap().in_global {
            (*GLOBALMQ.lock().unwrap()).push_queue(self.queue.clone())
        }
    }
    
    pub fn call_cb(&mut self, msg:RuskynetMsg) -> u32{ 
        let cb_fun = self.cb.take().unwrap();
        cb_fun(self, msg.session, msg.source, msg.data);
        self.cb = Some(cb_fun);
        //self.cb;
        return 0
    }
}