use std::thread;
use std::sync::{Arc, Mutex, Condvar};

mod rsknet_mq;
mod rsknet_handle;
mod rsknet_socket;
mod rsknet_server;
mod rsknet_monitor;
mod service_snlua;
mod rsknet_global;

use rsknet_monitor::RskynetMonitor;
use rsknet_server::RskynetContext;
use rsknet_global::{HANDLES, GLOBALMQ};


fn thread_worker(dispatch_type:u32, monitor:Arc<RskynetMonitor>){
    loop{
        let message_que = (*(GLOBALMQ.lock().unwrap())).pop_queue();
        if let Some(message_que) = message_que{
            let handle_id = (*message_que.lock().unwrap()).handle;
            let ctx = (*(HANDLES.lock().unwrap())).get_context(handle_id);
            let msgs = (*message_que.lock().unwrap()).get_msg().unwrap();
            for msg in msgs.into_iter() {//todo choose consume length
                // consume msg
               // ((*ctx.lock().unwrap()).cb)(ctx.clone(), msg.session, msg.source, msg.data);
               (*ctx.lock().unwrap()).call_cb(msg);
            }
        }else{
            // let thread_id = thread::current().id();
            monitor.wait_data();
        }
    }
}

fn boot_strap(){
    RskynetContext::new(HANDLES.clone());
    RskynetContext::new(HANDLES.clone());
    RskynetContext::new(HANDLES.clone());
    RskynetContext::new(HANDLES.clone());
    RskynetContext::new(HANDLES.clone());
}

fn main() {
    println!("hello ruskynet!");
    boot_strap();

    let thread_capacity:u32 = 5;
    let mut threads = Vec::with_capacity(thread_capacity.try_into().unwrap());
    let monitor = Arc::new(RskynetMonitor::new());
    let monitor_clone = monitor.clone();

    threads.push(thread::spawn(move || rsknet_socket::rsnet_socket_start(monitor_clone))); 

    for i in 1..=thread_capacity-1 {
        let monitor_clone = monitor.clone();
        threads.push(thread::spawn(move || thread_worker(i, monitor_clone))); 
    }
    for thread in threads.into_iter() {
        thread.join().unwrap();
    }
    println!("byebye ruskynet!");
}