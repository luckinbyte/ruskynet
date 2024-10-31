use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::rsknet_server::RskynetContext;

pub struct RskynetHandle{
    handle_index:u32,
    pub slot:HashMap<u32, Arc<Mutex<RskynetContext>>>,
}

impl RskynetHandle{
    pub fn new() -> Self{
        let slot = HashMap::new();
        return RskynetHandle{
            handle_index:0,
            slot
        }
    }
    pub fn handle_register(&mut self, context:Arc<Mutex<RskynetContext>>) {
        self.handle_index = self.handle_index+1;
        context.lock().unwrap().set_handle(self.handle_index);
        self.slot.insert(self.handle_index, context);
    }

    pub fn get_context(&self, handle_id:u32) -> Arc<Mutex<RskynetContext>>{
        return self.slot.get(&handle_id).unwrap().clone();
    }
}