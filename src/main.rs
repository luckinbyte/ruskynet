use std::thread;
use std::sync::{Arc, Mutex, Condvar};
//use std::sync::mpsc::channel;
use std::os::fd::AsRawFd;
use mio::unix::pipe;
use std::sync::RwLock;
use std::io::{Read, Write};

mod rsknet_mq;
mod rsknet_handle;
mod rsknet_socket;
mod rsknet_server;
mod rsknet_monitor;
mod service_snlua;
mod rsknet_global;
mod rsknet_timer;
mod lua_rsknet;
mod lua_socket;

use rsknet_monitor::RskynetMonitor;
use rsknet_server::RskynetContext;
use rsknet_global::{HANDLES, GLOBALMQ, SENDFD};

fn thread_worker(dispatch_type:u32, monitor:Arc<RskynetMonitor>){
    loop{
        let message_que = (*(GLOBALMQ.lock().unwrap())).pop_queue();
        if let Some(message_que) = message_que{
            let handle_id = (*message_que.lock().unwrap()).handle;
            let ctx = (*(HANDLES.lock().unwrap())).get_context(handle_id);
            let msgs = (*message_que.lock().unwrap()).get_msg().unwrap();
            println!("handld:{handle_id} has {} msg", msgs.len());
            for msg in msgs.into_iter() {//todo choose consume length
                let mut ctx = ctx.lock().unwrap();
                // consume msg
                // let raw_ptr: *const RskynetContext = &(*ctx) as *const RskynetContext;
                // println!("worker ptr:{:?}", raw_ptr);
                (*ctx).call_cb(msg);
            }
        }else{
            // let thread_id = thread::current().id();
            monitor.wait_data();
        }
    }
}

fn boot_strap(){
    RskynetContext::new(HANDLES.clone(), "snlua bootstrap");
}

fn main() {
    println!("hello ruskynet!");
    boot_strap();

    let thread_capacity:u32 = 4;
    let mut threads = Vec::with_capacity(thread_capacity.try_into().unwrap());
    let monitor = Arc::new(RskynetMonitor::new());
    let monitor_clone = monitor.clone();

    let (send_fd, recv_fd) = pipe::new().unwrap();
    SENDFD.get_or_init(||{send_fd});

    threads.push(thread::spawn(move || rsknet_socket::rsknet_socket_main_start(monitor_clone, recv_fd))); 
    let monitor_clone = monitor.clone();
    threads.push(thread::spawn(move || rsknet_timer::rsknet_timer_start(monitor_clone))); 

    for i in 1..=thread_capacity-1 {
        let monitor_clone = monitor.clone();
        threads.push(thread::spawn(move || thread_worker(i, monitor_clone))); 
    }
    for thread in threads.into_iter() {
        thread.join().unwrap();
    }
    println!("byebye ruskynet!");
}