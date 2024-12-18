use lazy_static::lazy_static;
use std::sync::{Arc, Mutex, Condvar, RwLock, OnceLock};
use std::os::raw::c_char;
use mio::unix::pipe::{Sender, Receiver};

use crate::rsknet_handle::RskynetHandle;
use crate::rsknet_server::RskynetContext;
use crate::rsknet_mq::GlobalQueue;
use crate::rsknet_socket::GlobalReqRecord;

lazy_static! {
    pub static ref HANDLES:Arc<Mutex<RskynetHandle>> = Arc::new(Mutex::new(RskynetHandle::new()));
    pub static ref GLOBALMQ:Arc<Mutex<GlobalQueue>> = Arc::new(Mutex::new(GlobalQueue::new()));
    pub static ref SENDFD:OnceLock<Sender> = OnceLock::new();
    pub static ref GLOBALREQ:Arc<Mutex<GlobalReqRecord>> = Arc::new(Mutex::new(GlobalReqRecord::new()));
}

pub fn get_ctx_by_handle(handle:u32) -> Arc<Mutex<RskynetContext>>{
    return (*HANDLES.lock().unwrap()).get_context(handle);
}

pub fn to_cstr(a:&str) -> *const c_char {
    return a.as_bytes().as_ptr() as *const c_char;
    //return a as *const str as *const [c_char] as *const c_char;
}

pub static LUACBFUNSTR:&str = "lua_cb_fun";
pub static RSKNETCTXSTR:&str = "rsknet_context";

