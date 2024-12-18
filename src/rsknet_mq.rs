use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

pub struct RuskynetMsg{
	pub proto_type:u32,
	pub data:Vec<u8>,
	pub session: u32,
	pub source: u32,
}

impl RuskynetMsg{
	pub fn new(proto_type:u32, data:Vec<u8>, session:u32, source:u32) -> Self{
		return RuskynetMsg{
			proto_type, data, session, source
		}
	}
}

pub struct MessageQueue {
	// struct spinlock lock;
	pub handle:u32,// uint32_t handle;
	// int cap;
	// int head;
	// int tail;
	// int release;
	pub in_global:bool,
	// int overload;
	// int overload_threshold;
	pub queue:Option<Vec<RuskynetMsg>>,// struct skynet_message *queue;>
}

impl MessageQueue{
	pub fn new() -> Self{
		return MessageQueue{
			handle:0,
			in_global:false,
			queue:Option::None, 
		}
	}
	pub fn set_handle(&mut self, handle_id:u32){
		self.handle = handle_id;
	}
	pub fn push_msg(&mut self, msg:RuskynetMsg){
		match &mut self.queue {
			None=>{
				let mut new_vec = Vec::new();
				new_vec.push(msg);
				self.queue = Some(new_vec);
			},
			Some(temp)=>temp.push(msg),
		}
		return;
	}
	pub fn get_msg(&mut self) -> Option<Vec<RuskynetMsg>>{
		self.in_global = false;
		return self.queue.take();
	}
}
pub struct GlobalQueue {
	global_que:Option<Arc<Mutex<VecDeque<Arc<Mutex<MessageQueue>>>>>>,
}

impl GlobalQueue{
	pub fn new() -> Self{
		return GlobalQueue{
			global_que:None,
		}
	}
	pub fn push_queue(&mut self, queue:Arc<Mutex<MessageQueue>>){
		match &mut self.global_que{
			None=>{
				let mut temp = VecDeque::new();
				temp.push_back(queue);
				self.global_que = Some(Arc::new(Mutex::new(temp)));
			},
			Some(temp)=>{
				(*temp.lock().unwrap()).push_back(queue);
			}
		}
	}
	pub fn pop_queue(&mut self) -> Option<Arc<Mutex<MessageQueue>>>{
		match &mut self.global_que {
			None => None,
			Some(temp)=>{
				return (*temp.lock().unwrap()).pop_front()
			}
		}
	}
}