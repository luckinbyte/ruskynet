use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

use crate::rsknet_handle::{RskynetHandle};
use crate::rsknet_monitor::{RskynetMonitor};
use crate::rsknet_mq::{RuskynetMsg, GlobalQueue};
use crate::rsknet_global::{HANDLES, GLOBALMQ};

pub fn rsnet_socket_start(monitor:Arc<RskynetMonitor>) {
    for i in 1..=5 {
        for j in 1..=1 {
            thread::sleep(Duration::from_secs(1));
            let handle_id:u32 = i;
            let ctx = (*(HANDLES.lock().unwrap())).get_context(handle_id);
            let mut data = Vec::new();
            data.push(i);
            let new_msg = RuskynetMsg::new(i, i, data);
            println!("from socket push msg {i}");
            ctx.lock().unwrap().push_msg(new_msg);
        }
        monitor.wake_up();
    }
}