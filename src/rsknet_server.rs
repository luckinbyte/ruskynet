use std::sync::{Arc, Mutex};

use crate::rsknet_mq::{MessageQueue, RuskynetMsg, GlobalQueue};

pub struct RskynetContext{
	handle:u32,// uint32_t handle;
    queue:Arc<Mutex<MessageQueue>>,
    //session:u32,
}

impl RskynetContext{
    pub fn new() -> Self{
        let queue = Arc::new(Mutex::new(MessageQueue::new()));
        return RskynetContext{
            handle:1,
            queue,
        }
    }

    pub fn set_handle(&mut self, handle:u32){
        self.handle = handle;
        self.queue.lock().unwrap().set_handle(handle);
    }

    pub fn push_msg(&mut self, global_que:Arc<Mutex<GlobalQueue>>, msg:RuskynetMsg) {
        (*self.queue.lock().unwrap()).push_msg(msg);
        if !self.queue.lock().unwrap().in_global {
            (*global_que.lock().unwrap()).push_queue(self.queue.clone())
        }
    }
}