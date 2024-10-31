use std::sync::{Arc, Mutex, Condvar};
pub struct RskynetMonitor{
    pub mutex: Mutex<bool>,
    pub condvar: Condvar,
}

impl RskynetMonitor{
    pub fn new() -> Self{
        return RskynetMonitor{
            mutex: Mutex::new(false),
            condvar: Condvar::new(),
        }
    }

    pub fn wake_up(&self) {
        *self.mutex.lock().unwrap() = true;
        self.condvar.notify_all();
    }

    pub fn wait_data(&self){
        let mut guard = self.mutex.lock().unwrap();
        while !*guard {
            guard = self.condvar.wait(guard).unwrap();
        }
        *guard = false;
    }
}
