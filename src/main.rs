#![feature(io_error_more)]
#![feature(duration_millis_float)]
use core::fmt;
use std::{io, net::{SocketAddr, ToSocketAddrs}, time::{Duration, Instant}};

use socket2::{Domain, Protocol, SockAddr, Socket, Type};


fn main() {
    println!("starting atr...");

    let target_host = "stackpointer.dev:443";
    let target_addrs = resolve_host_name(target_host);
    println!("target_ips are {:?}", target_addrs);
    let target_addr = target_addrs[0];
    let timeout = Duration::from_secs(1);
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();
    println!("socket created {:?} -> ", socket);

    for ttl in 1..64 {
        socket.set_ttl(ttl).unwrap();
        socket.set_read_timeout(Some(timeout)).unwrap();
        send_packet(&socket, target_addr);
    }
}

fn resolve_host_name(host_name: &str) -> Vec<SocketAddr>{
    println!("hostname is {}", host_name);
    let ips: Vec<SocketAddr> = host_name.to_socket_addrs().unwrap().collect();
    ips
}

fn send_packet(socket: &Socket, addr: SocketAddr) {
    let t_start = Instant::now();
    match socket.connect(&SockAddr::from(addr)) {
        Ok(resp) => {
            let info = Info {
                tt: t_start.elapsed().as_millis_f32(),
                status: String::from("DONE")
            };
            println!("{}", info);
        },
        Err(error_info) => {
            match error_info.kind() {
                io::ErrorKind::ConnectionRefused => {
                    let info = Info {
                        tt: t_start.elapsed().as_millis_f32(),
                        status: String::from("*")
                    };
                    println!("{}", info);
                },
                io::ErrorKind::HostUnreachable => {
                    let info = Info {
                        tt: t_start.elapsed().as_millis_f32(),
                        status: String::from("OK")
                    };
                    println!("{}", info);
                },
                _ => {
                    let info = Info {
                        tt: t_start.elapsed().as_millis_f32(),
                        status: String::from("***")
                    };
                    println!("{}", info);
                },
            }
        }
    }
}

struct Info {
    tt: f32,
    status: String
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.tt, self.status)
    }
}

