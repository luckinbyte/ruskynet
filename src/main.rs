use std::thread;
use std::sync::{Arc, Mutex, Condvar};
use lazy_static::lazy_static;

mod rsknet_mq;
mod rsknet_handle;
mod rsknet_socket;
mod rsknet_server;
mod rsknet_monitor;

use rsknet_handle::RskynetHandle;
use rsknet_monitor::RskynetMonitor;
use rsknet_server::RskynetContext;

lazy_static! {
    static ref HANDLES:Arc<Mutex<RskynetHandle>> = Arc::new(Mutex::new(RskynetHandle::new()));
    static ref GLOBALMQ:Arc<Mutex<rsknet_mq::GlobalQueue>> = Arc::new(Mutex::new(rsknet_mq::GlobalQueue::new()));
}

fn thread_worker(dispatch_type:u32, monitor:Arc<RskynetMonitor>){
    loop{
        let mut message_que = (*(GLOBALMQ.lock().unwrap())).pop_queue();
        if let Some(mut message_que) = message_que{
            let handle_id = (*message_que.lock().unwrap()).handle;
            let ctx = (*(HANDLES.lock().unwrap())).get_context(handle_id);
            let msgs = (*message_que.lock().unwrap()).get_msg().unwrap();
            for msg in msgs.iter() {//todo choose consume length
                // consume msg
                let data = &msg.data;
                println!("from worked {data:?}")
            }
        }else{
            let thread_id = thread::current().id();
            monitor.wait_data();
        }
    }
}

fn boot_strap(){
    (*(HANDLES.lock().unwrap())).handle_register(Arc::new(Mutex::new(RskynetContext::new())));
    (*(HANDLES.lock().unwrap())).handle_register(Arc::new(Mutex::new(RskynetContext::new())));
    (*(HANDLES.lock().unwrap())).handle_register(Arc::new(Mutex::new(RskynetContext::new())));
    (*(HANDLES.lock().unwrap())).handle_register(Arc::new(Mutex::new(RskynetContext::new())));
    (*(HANDLES.lock().unwrap())).handle_register(Arc::new(Mutex::new(RskynetContext::new())));
}

fn main() {
    println!("hello ruskynet!");
    boot_strap();

    let thread_capacity:u32 = 5;
    let mut threads = Vec::with_capacity(thread_capacity.try_into().unwrap());
    let handles_clone = HANDLES.clone();
    let monitor = Arc::new(RskynetMonitor::new());
    let monitor_clone = monitor.clone();

    let global_clone = GLOBALMQ.clone();
    threads.push(thread::spawn(move || rsknet_socket::rsnet_socket_start(monitor_clone, handles_clone, global_clone))); 

    for i in 1..=thread_capacity-1 {
        let monitor_clone = monitor.clone();
        threads.push(thread::spawn(move || thread_worker(i, monitor_clone))); 
    }
    for thread in threads.into_iter() {
        thread.join().unwrap();
    }
    println!("byebye ruskynet!");
}