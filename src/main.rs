#![feature(io_error_more)]
#![feature(duration_millis_float)]
use core::fmt;
use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
    time::{Duration, Instant},
};

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
        let info: Info = send_packet_t(&socket, target_addr);
        println!("{}", info);
        match info._status {
            Status::DONE => {
                break;
            }
            _ => {
                continue;
            }
        }
    }

}

fn resolve_host_name(host_name: &str) -> Vec<SocketAddr> {
    println!("hostname is {}", host_name);
    let ips: Vec<SocketAddr> = host_name.to_socket_addrs().unwrap().collect();
    ips
}

fn send_packet_t(socket: &Socket, addr: SocketAddr) -> Info {
    let t_start = Instant::now();
    let info: Info;
    match socket.connect(&SockAddr::from(addr)) {
        Ok(_resp) => {
            info = Info {
                tt: t_start.elapsed().as_millis_f32(),
                _status: Status::DONE,
                status_line: String::from("DONE"),
            };
        }
        Err(error_info) => match error_info.kind() {
            io::ErrorKind::ConnectionRefused => {
                info = Info {
                    tt: t_start.elapsed().as_millis_f32(),
                    _status: Status::ERR,
                    status_line: String::from("*"),
                };
            }
            io::ErrorKind::HostUnreachable => {
                info = Info {
                    tt: t_start.elapsed().as_millis_f32(),
                    _status: Status::OK,
                    status_line: String::from("OK"),
                };
            }
            _ => {
                info = Info {
                    tt: t_start.elapsed().as_millis_f32(),
                    _status: Status::ERR,
                    status_line: String::from("***"),
                };
            }
        },
    }

    info
}

struct Info {
    tt: f32,
    _status: Status,
    status_line: String,
}

enum Status {
    DONE,
    OK,
    ERR,
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3} ms {}", self.tt, self.status_line)
    }
}
