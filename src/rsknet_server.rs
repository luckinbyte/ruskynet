use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use mlua::{lua_State, FromLua, Function, Lua, MetaMethod, Result, UserData, UserDataMethods, Value, Variadic};

use crate::rsknet_handle::RskynetHandle;
use crate::rsknet_mq::{MessageQueue, RuskynetMsg, GlobalQueue};
use crate::service_snlua::{RsnLua};
use crate::rsknet_global::{get_ctx_by_handle, GLOBALMQ, HANDLES};

pub struct RskynetContext{
    pub instance:Arc<Mutex<RsnLua>>, 
    pub cb:Option<fn(&mut RskynetContext, u32, Vec<u8>, u32, u32)-> Result<()>>,
    pub cb_userdata:Option<()>,
    session_id:u32,
	pub handle:u32,// uint32_t handle;
    queue:Arc<Mutex<MessageQueue>>,
}

fn context_push(destination:u32, msg:RuskynetMsg) -> u32{
    let ctx = get_ctx_by_handle(destination);
    ctx.lock().unwrap().push_msg(msg);
    return 0;
}

pub fn rsknet_send(ctx:Arc<Mutex<RskynetContext>>, source:u32, destination:u32, ptype:u32, session:u32, data:Vec<u8>) -> u32{
    let msg = RuskynetMsg::new(ptype, data, session, source);
    context_push(destination, msg); 
    println!("push success {}", session);
    return 1;
}

impl RskynetContext{
    pub fn new(hanlde:Arc<Mutex<RskynetHandle>>, arg:&str) -> Arc<Mutex<RskynetContext>>{
        let queue = Arc::new(Mutex::new(MessageQueue::new()));
        let instance = Arc::new(Mutex::new(RsnLua::new()));

        let ctx = Arc::new(Mutex::new(
            RskynetContext{
                instance:instance.clone(),
                cb:None,
                cb_userdata:None,
                session_id:1,
                handle:1,
                queue,
            }
        ));
        (*hanlde.lock().unwrap()).handle_register(ctx.clone());
        (*instance.lock().unwrap()).init(ctx.clone(), arg);

        return ctx;
    }

    pub fn set_handle(&mut self, handle:u32){
        self.handle = handle;
        self.queue.lock().unwrap().set_handle(handle);
    }

    pub fn push_msg(&mut self, msg:RuskynetMsg) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_msg(msg);
        if !queue.in_global {
            queue.in_global = true;
            (*GLOBALMQ.lock().unwrap()).push_queue(self.queue.clone())
        }
    }
    
    pub fn call_cb(&mut self, msg:RuskynetMsg) -> u32{ 
        let cb_fun = self.cb.take().unwrap();
        cb_fun(self, msg.proto_type, msg.data, msg.session, msg.source);
        if self.cb == None{
            self.cb = Some(cb_fun);
        }
        return 0
    }

    pub fn rsknet_command(&mut self, cmd:String, pram:String) -> Option<String>{
        match &cmd as &str{
            "LAUNCH" => {
                println!("command launch success, {:?}", pram);

                RskynetContext::new(HANDLES.clone(), pram.as_str());

                // size_t sz = strlen(param);
                // char tmp[sz+1];
                // strcpy(tmp,param);
                // char * args = tmp;
                // char * mod = strsep(&args, " \t\r\n");
                // args = strsep(&args, "\r\n");
                // struct skynet_context * inst = skynet_context_new(mod,args);
                // if (inst == NULL) {
                //     return NULL;
                // } else {
                //     id_to_hex(context->result, inst->handle);
                //     return context->result;
                // }

                None
            },
            _ => None
        }
    }
}