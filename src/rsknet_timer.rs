use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::rsknet_monitor::{RskynetMonitor};

fn rsknet_socket_updatetime(){
}

pub fn rsnet_timer_start(monitor:Arc<RskynetMonitor>){
    loop{
        rsknet_socket_updatetime();
        thread::sleep(Duration::from_millis(100));
        monitor.wake_up();
    }
}