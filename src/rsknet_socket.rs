use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

use crate::rsknet_handle::{RskynetHandle};
use crate::rsknet_monitor::{RskynetMonitor};
use crate::rsknet_mq::{RuskynetMsg, GlobalQueue};
use crate::rsknet_global::{HANDLES, GLOBALMQ};

pub fn rsnet_socket_start(monitor:Arc<RskynetMonitor>) {
    return ();
    for i in 1..=1 {
        for j in 1..=2 {
            thread::sleep(Duration::from_secs(1));
            let handle_id:u32 = i;
            let ctx = (*(HANDLES.lock().unwrap())).get_context(handle_id);
            let mut data: Vec<u8> = Vec::new();
            data.push(i as u8);
            println!("from socket push msg begin {handle_id}, {}", i*10+j);
            let new_msg = RuskynetMsg::new(i, data, i*10+j, i);
            println!("from socket push msg end {handle_id} {}", i*10+j);
            ctx.lock().unwrap().push_msg(new_msg);
            monitor.wake_up();
        }
        
    }
}