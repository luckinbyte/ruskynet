use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use mio::unix::pipe::{Receiver};
use std::collections::HashMap;
use std::io::{Read, Write};
use libc;

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};


use crate::rsknet_handle::{RskynetHandle};
use crate::rsknet_monitor::{RskynetMonitor};
use crate::rsknet_mq::{RuskynetMsg, GlobalQueue};
use crate::rsknet_global::{HANDLES, GLOBALMQ, SENDFD, GLOBALREQ};
use crate::rsknet_server::{RskynetContext};

struct ReqRecord{
    source:u32,
    req_type:u32,
}

pub struct GlobalReqRecord{
    id:u32,
    id_rec:HashMap<u32, ReqRecord>
}

impl GlobalReqRecord{
    pub fn new() -> Self{
        return GlobalReqRecord{
            id:0,
            id_rec:HashMap::new(),
        }
    }

    fn add_req(&mut self, source: u32, req_type: u32)->u32{
        self.id = self.id+1;
        self.id_rec.insert(self.id, ReqRecord{source, req_type});
        return self.id
    }
}

enum PolEnety {
    TcpStream(TcpStream),
    TcpListener(TcpListener),
}

pub struct SocketServer {
	// volatile uint64_t time;
	// int reserve_fd;	// for EMFILE
	// int recvctrl_fd;
	// int sendctrl_fd;
	// int checkctrl;
	// poll_fd event_fd;
	// ATOM_INT alloc_id;
    checkctrl:bool,
	event_n:u32,
	event_index:u32,
    events:Events,
    poll:Poll,
    token_id:u32,
    token_map:HashMap<u32, PolEnety>,
    recv_fd:Receiver,
	// struct socket_object_interface soi;
	// struct event ev[MAX_EVENT];
	// struct socket slot[MAX_SOCKET];
	// char buffer[MAX_INFO];
	// uint8_t udpbuffer[MAX_UDP_PACKAGE];
	// fd_set rfds;
}
impl SocketServer{
    pub fn new(recv_fd: Receiver) -> Self{
        let poll = Poll::new().unwrap();
        let token_id:u32 = 1;
        let token_map = HashMap::new();
        let events = Events::with_capacity(128);
        
        let mut ret = SocketServer{
            checkctrl:true,
            event_index:0,
            event_n:0,
            events,
            poll,
            token_id,
            token_map,
            recv_fd
        };

        ret.poll.registry()
            .register(&mut ret.recv_fd, Token(1 as usize), Interest::READABLE).unwrap();
        return ret
    }

    //fn add_fd()

}

pub fn rsknet_socket_start(monitor:Arc<RskynetMonitor>, recv_fd:Receiver) {
    let mut ss = SocketServer::new(recv_fd);
    loop{
        let r = rsknet_socket_epoll(&mut ss);
        if r == 0 {
            break
        }
        monitor.wake_up();
    }
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

fn rsknet_socket_epoll(ss:&mut SocketServer) -> u32{
    let ret = socket_server_epoll(ss);
    match ret {
        _ => {}
    } 
    return 1
}

fn deal_cmd(ss:&mut SocketServer) -> u32{
    let mut buf = [0; 512];
    let n = ss.recv_fd.try_io(|| {
        let buf_ptr = &mut buf as *mut _ as *mut _;
        let res = unsafe { libc::read(ss.recv_fd.as_raw_fd(), buf_ptr, buf.len()) };
        Ok(res)
    }).unwrap();
    if n > 0 {
        let mut recv_vec = buf.to_vec();
        let _ = recv_vec.split_off(n as usize);
        let recv_str = String::from_utf8(recv_vec).unwrap();
        let arg:Vec<&str> = recv_str.split_whitespace().collect();
        match arg[1]{
            "1" =>{ //listen  2:host 3:port
                let ip = arg[2].to_string();
                let port = arg[3].to_string();
                let ip_port = ip+&":"+&port;
                let addr: std::net::SocketAddr = ip_port.parse().unwrap();

                let server = TcpListener::bind(addr).unwrap();
                ss.token_id = ss.token_id+1;
                ss.token_map.insert(ss.token_id, PolEnety::TcpListener(server));
                if let PolEnety::TcpListener(pol_server) = ss.token_map.get_mut(&ss.token_id).unwrap(){
                    ss.poll.registry()
                    .register(pol_server, Token(ss.token_id as usize), Interest::READABLE).unwrap();
                };
            }
            _ =>{
                //println!("deal_cmd error req_type {}", &arg[1])
            }
        }
        return 1
    }else{
        return 0
    }
}

fn socket_server_epoll(ss:&mut SocketServer) -> u32{
    loop{
        if (ss.checkctrl){
            let cmd_ret = deal_cmd(ss);
            if cmd_ret != 0 {return cmd_ret} else {ss.checkctrl=false};
        }
        if (ss.event_index == ss.event_n){
            ss.checkctrl=true;
            ss.poll.poll(&mut ss.events, None).unwrap();
        }

        //if ()
    }
    return 1
}


pub fn rsknet_socket_listen(handle_id:u32, host:String, port:u32) -> u32 {
    let req_id = GLOBALREQ.lock().unwrap().add_req(handle_id, 1);
    let send_str = req_id.to_string()+&" 1 "+&host+&" "+&port.to_string();
    let mut send_fd = SENDFD.get().unwrap();
    send_fd.write_all(send_str.as_bytes()).unwrap();
    return req_id;
}