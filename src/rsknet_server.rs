use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::vec;
use mlua::{lua_State, FromLua, Function, Lua, MetaMethod, Result, UserData, UserDataMethods, Value, Variadic};
use std::str;

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
    println!("rsknet_init_send source:{} des:{} session:{} data:{:?}", source, destination, session, str::from_utf8(&data).unwrap());
    let msg = RuskynetMsg::new(ptype, data, session, source);
    context_push(destination, msg); 
    return 1;
}

impl RskynetContext{
    pub fn new(hanlde:Arc<Mutex<RskynetHandle>>, arg:&str) -> u32{
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
        let handle = (*hanlde.lock().unwrap()).handle_register(ctx.clone());
        (*instance.lock().unwrap()).init(ctx.clone(), arg);
        println!("!!!!! new handle {handle} !!!!!");
        return handle;
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
        println!("handle:{:?} rsknet_command: {:?}", self.handle, &cmd as &str);
        match &cmd as &str{
            "LAUNCH" => {
                let handle = RskynetContext::new(HANDLES.clone(), pram.as_str());
                Some(handle.to_string())
            },
            "TIMEOUT" => {
                self.session_id = self.session_id+1;
                let new_sid = self.session_id;
                let ptype = 1; //PTYPE_RESPONSE
                let new_msg = RuskynetMsg::new(ptype, "TIMEOUT".to_string().into_bytes(), new_sid, self.handle);
                self.push_msg(new_msg);
                Some(new_sid.to_string())
            },
            _ => None
        }
    }

    pub fn rsknet_send(&mut self, des:u32, ptype:u32, session:u32, data:String) -> u32{
        let new_sid = if session == 0 {
            self.session_id = self.session_id+1;
            self.session_id
        }else{
            session
        };
        let handle_id:u32 = des;
        let des_ctx = (*(HANDLES.lock().unwrap())).get_context(handle_id);

        println!("rsknet_core_send hand:{} des:{} session:{} ptype:{ptype} data:{:?}", self.handle, handle_id, new_sid, &data);

        let new_msg = RuskynetMsg::new(ptype, data.into_bytes(), new_sid, self.handle);
        des_ctx.lock().unwrap().push_msg(new_msg);
        new_sid
    }
}