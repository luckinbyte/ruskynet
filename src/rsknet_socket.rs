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

pub struct GlobalReqRecord{
    id:u32,
}

impl GlobalReqRecord{
    pub fn new() -> Self{
        return GlobalReqRecord{
            id:0,
        }
    }

    fn add_req(&mut self)->u32{
        self.id = self.id+1;
        return self.id
    }
}

static RSKNET_SOCKET_TYPE_DATA:u32 = 1;
static RSKNET_SOCKET_TYPE_CONNECT:u32 = 2;
static RSKNET_SOCKET_TYPE_ACCEPT:u32 = 4;

enum EpolResult{
    ListenRet(Vec<(u32, TcpStream, SocketAddr, u32)>),
    DataRet((u32, String, u32)),
}


enum PolEnety {
    TcpStream(TcpStream),
    TcpListener(TcpListener),
}

struct SocketEnety{
    pol_enety:PolEnety,
    socket_type:u32,
    req_id:u32,
    source:u32,
}

impl SocketEnety{
    fn new(pol_enety:PolEnety, source:u32, req_id:u32, socket_type:u32) -> Self{
        return SocketEnety{
            pol_enety, source, req_id, socket_type
        }
    }
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
	//event_n:u32,
	//event_index:u32,
    events:Events,
    poll:Poll,
    token_id:u32,
    token_map:HashMap<u32, SocketEnety>,
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
            //event_index:0,
            //event_n:0,
            events,
            poll,
            token_id,
            token_map,
            recv_fd
        };

        ret.poll.registry()
            .register(&mut ret.recv_fd, Token(0 as usize), Interest::READABLE).unwrap();
        return ret
    }

    //fn add_fd()

}

pub fn rsknet_socket_main_start(monitor:Arc<RskynetMonitor>, recv_fd:Receiver) {
    let mut ss = SocketServer::new(recv_fd);
    loop{
        let r = rsknet_socket_epoll(&mut ss);
        if r == 0 {
            break
        }
        monitor.wake_up();
    }
    return ();
}

fn rsknet_socket_epoll(ss:&mut SocketServer) -> u32{
    let ret = socket_server_epoll(ss);
    match ret {
        _ => {}
    } 
    return 1
}

fn deal_cmd(ss:&mut SocketServer) -> u32{
    let mut buf_len = [0; 1];
    let n = ss.recv_fd.try_io(|| {
        let buf_ptr = &mut buf_len as *mut _ as *mut _;
        let res = unsafe { libc::read(ss.recv_fd.as_raw_fd(), buf_ptr, buf_len.len())};
        Ok(res)
    }).unwrap();
    if n > 0 {
        let mut buf = [0; 512];
        println!("deal_cmd len of buf_len n:{n}");
        let n = ss.recv_fd.try_io(|| {
            let buf_ptr = &mut buf as *mut _ as *mut _;
            let res = unsafe {libc::read(ss.recv_fd.as_raw_fd(), buf_ptr, buf_len[0])};
            Ok(res)
        }).unwrap();
        println!("deal_cmd len of buf_data n:{n}");
        let recv_vec = buf[..n as usize].to_vec();
        //let _ = recv_vec.split_off(n as usize);
        let recv_str = String::from_utf8(recv_vec).unwrap();
        println!("deal_cmd recv_str:{}", &recv_str);
        let arg:Vec<&str> = recv_str.split_whitespace().collect();
        match arg[1]{
            "1" =>{ //listen    0:req_id 1:type 2:source 3:host 4:port
                let req_id:u32 = arg[0].to_string().parse().unwrap();
                let source:u32 = arg[2].to_string().parse().unwrap();
                let ip = arg[3].to_string();
                let port = arg[4].to_string();
                let ip_port = ip+&":"+&port;
                let addr: std::net::SocketAddr = ip_port.parse().unwrap();

                let server = TcpListener::bind(addr).unwrap();
                //ss.token_id = ss.token_id+1;
                ss.token_map.insert(req_id, 
                    SocketEnety::new(PolEnety::TcpListener(server), source, req_id, 1)
                );
                if let PolEnety::TcpListener(pol_server) = &mut ss.token_map.get_mut(&req_id).unwrap().pol_enety{
                    ss.poll.registry()
                    .register(pol_server, Token(req_id as usize), Interest::READABLE).unwrap();
                };
                
                let ctx = (*(HANDLES.lock().unwrap())).get_context(source);
                // proto_type:6 socket
                let new_msg = RuskynetMsg::new(6, format!("[{},{},{},\"{}\"]",RSKNET_SOCKET_TYPE_CONNECT,req_id,port,arg[3].to_string()).into_bytes(), 0, 0);
                ctx.lock().unwrap().push_msg(new_msg);
            },
            "2" =>{ //start    0:id 1:type 2:source
                let id:u32 = arg[0].to_string().parse().unwrap();
                let source:u32 = arg[2].to_string().parse().unwrap();

                let s_enety = ss.token_map.get_mut(&id).unwrap();
                s_enety.source = source;

                let ctx = (*(HANDLES.lock().unwrap())).get_context(source);
                // proto_type:6 socket
                let new_msg = RuskynetMsg::new(6, format!("[{},{}]",RSKNET_SOCKET_TYPE_CONNECT,id).into_bytes(), 0, 0);
                ctx.lock().unwrap().push_msg(new_msg);
            },
            "3" =>{//send  0:id 1:type 2:data
                let id:u32 = arg[0].to_string().parse().unwrap();
                let data = arg[2];

                let s_enety = ss.token_map.get_mut(&id).unwrap();
                if let PolEnety::TcpStream(ref mut client_fd) = s_enety.pol_enety{
                    let _ = client_fd.write_all(data.as_bytes());
                }
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
        if ss.checkctrl {
            let cmd_ret = deal_cmd(ss);
            if cmd_ret != 0 {return cmd_ret} else {ss.checkctrl=false};
        }
        if ss.events.is_empty() {
            ss.checkctrl=true;
            println!("!!!!!epol begin");
            ss.poll.poll(&mut ss.events, None).unwrap();
            println!("!!!!!epol end");
        }
        if !ss.events.is_empty() {
            for event in ss.events.into_iter() {
                let Token(token_id) = event.token();
                println!("token_id token_id:{token_id:?}");
                let socket_enety = ss.token_map.get_mut(&(token_id as u32));
                let ret: Option<EpolResult> = match socket_enety{
                    Some(socket_enety) =>{
                        match socket_enety.socket_type{
                            1 =>{//listen
                                let mut listen_vec: Vec<(u32, TcpStream, SocketAddr, u32)> = Vec::new();
                                if let PolEnety::TcpListener(tcp_listen) = &socket_enety.pol_enety{
                                    loop {
                                        match tcp_listen.accept() {
                                            Ok((client_fd, socket_addr)) => {
                                                // if next_socket_index == MAX_SOCKETS {
                                                //     return Ok(());
                                                // }
                                                listen_vec.push((socket_enety.req_id, client_fd, socket_addr, socket_enety.source));
                                            }
                                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                                break;
                                            }
                                            e => panic!("err={:?}", e), // Unexpected error
                                        }
                                    }
                                }else{
                                    println!("TcpListener none")
                                };
                                Some(EpolResult::ListenRet(listen_vec))
                            },
                            2 =>{//socket
                                if event.is_readable() {
                                    if let PolEnety::TcpStream(ref mut connection) = socket_enety.pol_enety{
                                        let mut received_data = vec![0; 4096];
                                        let mut bytes_read = 0;
                                        // We can (maybe) read from the connection.
                                        loop {
                                            match connection.read(&mut received_data[bytes_read..]) {
                                                Ok(0) => {
                                                    // todo Socket is closed, remove it from the map
                                                    println!("connection read len equal zero");
                                                    break;
                                                }
                                                Ok(n) => {
                                                    bytes_read += n;
                                                    if bytes_read == received_data.len() {
                                                        received_data.resize(received_data.len() + 1024, 0);
                                                    }
                                                }
                                                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                                    // Socket is not ready anymore, stop reading
                                                    break;
                                                }
                                                Err(err) => {
                                                    println!("connection read err {err:?}");
                                                    break;
                                                }
                                            }
                                        }
                                        if bytes_read != 0 {
                                            let received_data = &received_data[..bytes_read];
                                            if let Ok(str_buf) = std::str::from_utf8(received_data) {
                                                println!("Received data: {}", str_buf.trim_end());
                                                Some(EpolResult::DataRet((socket_enety.req_id, str_buf.to_string(), socket_enety.source)))
                                            } else {
                                                println!("Received (none UTF-8) data: {:?}", received_data);
                                                None
                                            }
                                        }else{
                                            None
                                        }
                                    }else{
                                        None
                                    }
                                }else{
                                    None
                                }
                            }
                            _ => None
                        }
                    }
                    _ =>{
                        ss.checkctrl=true;
                        None
                    }
                };
                match ret{
                    Some(EpolResult::ListenRet(listen_vec)) =>{
                        for (old_req_id, client_fd, socket_addt, source) in listen_vec.into_iter(){
                            let req_id = GLOBALREQ.lock().unwrap().add_req();
                            ss.token_map.insert(req_id, 
                                SocketEnety::new(PolEnety::TcpStream(client_fd), source, req_id, 2)
                            );
                            if let PolEnety::TcpStream(pol_stream) = &mut ss.token_map.get_mut(&req_id).unwrap().pol_enety{
                                ss.poll.registry()
                                .register(pol_stream, Token(req_id as usize), Interest::READABLE).unwrap();
                            };
            
                            let ctx = (*(HANDLES.lock().unwrap())).get_context(source);
                            let new_msg = RuskynetMsg::new(6, format!("[{},{},{},\"{}\"]",RSKNET_SOCKET_TYPE_ACCEPT,old_req_id,req_id,socket_addt.to_string()).into_bytes(), 0, 0);
                            ctx.lock().unwrap().push_msg(new_msg);
                        }
                    },
                    Some(EpolResult::DataRet((req_id, data, source))) =>{
                        let ctx = (*(HANDLES.lock().unwrap())).get_context(source);
                        let new_msg = RuskynetMsg::new(6, format!("[{},{},\"{}\"]",RSKNET_SOCKET_TYPE_DATA,req_id,data).into_bytes(), 0, 0);
                        ctx.lock().unwrap().push_msg(new_msg);
                    }
                    _ =>{

                    }
                }

            }
            ss.events.clear();
        }
        //if ()
    }
    return 1
}


pub fn rsknet_socket_listen(handle_id:u32, host:String, port:u32) -> u32 {
    let req_id = GLOBALREQ.lock().unwrap().add_req();
    let send_str = req_id.to_string()+&" 1 "+&handle_id.to_string()+&" "+&host+&" "+&port.to_string();
    let mut send_fd = SENDFD.get().unwrap();
    let send_bytes = send_str.as_bytes();
    let send_len = send_bytes.len() as u8;
    send_fd.write_all(&[send_len]).unwrap();
    send_fd.write_all(send_bytes).unwrap();
    return req_id;
}

pub fn rsknet_socket_start(handle_id:u32, id:u32) {
    let send_str = id.to_string()+&" 2 "+&handle_id.to_string();
    let mut send_fd = SENDFD.get().unwrap();
    let send_bytes = send_str.as_bytes();
    let send_len = send_bytes.len() as u8;
    send_fd.write_all(&[send_len]).unwrap();
    send_fd.write_all(send_bytes).unwrap();
    return ()
}

pub fn rsknet_socket_send(_handle_id:u32, id:u32, data:String){
    let send_str = id.to_string()+&" 3 "+&data;
    let mut send_fd = SENDFD.get().unwrap();
    let send_bytes = send_str.as_bytes();
    let send_len = send_bytes.len() as u8;
    //todo send_len > u8
    send_fd.write_all(&[send_len]).unwrap();
    send_fd.write_all(send_bytes).unwrap();
    return ()
}