#![feature(io_error_more)]
#![feature(duration_millis_float)]
use core::fmt;
use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
    time::{Duration, Instant},
};

use clap::Parser;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

#[tokio::main]
async fn main() {
    println!("starting atr...");

    let args = Args::parse();

    let target_host = &args.target_host;
    let target_addrs = resolve_host_name(target_host);
    println!("target_ips are {:?}", target_addrs);
    let target_addr = target_addrs[0];

    for ttl in 1..64 {
        let info = send_packet_t(ttl, target_addr);
        println!("{}", info);
        if let Status::DONE = info._status {
            break;
        }
    }
}

fn resolve_host_name(host_name: &str) -> Vec<SocketAddr> {
    println!("hostname is {}", host_name);
    let ips: Vec<SocketAddr> = host_name.to_socket_addrs().unwrap().collect();
    ips
}

fn send_packet_t(ttl: u32, addr: SocketAddr) -> Info {
    let timeout = Duration::from_secs(1);
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();
    socket.set_read_timeout(Some(timeout)).unwrap();
    socket.set_write_timeout(Some(timeout)).unwrap();
    socket.set_ttl(ttl).unwrap();
    let t_start = Instant::now();

    let r = socket.connect(&SockAddr::from(addr));

    match r {
        Ok(_) => Info {
            ttl,
            tt: t_start.elapsed().as_millis_f32(),
            _status: Status::DONE,
            status_line: String::from("DONE"),
        },
        Err(error) => {
            let info = match error.kind() {
                io::ErrorKind::ConnectionRefused => Info {
                    ttl,
                    tt: t_start.elapsed().as_millis_f32(),
                    _status: Status::ERR,
                    status_line: String::from("*"),
                },
                io::ErrorKind::HostUnreachable => Info {
                    ttl,
                    tt: t_start.elapsed().as_millis_f32(),
                    _status: Status::OK,
                    status_line: String::from("OK"),
                },
                _ => Info {
                    ttl,
                    tt: t_start.elapsed().as_millis_f32(),
                    _status: Status::ERR,
                    status_line: String::from("***"),
                },
            };
            info
        }
    }
}

struct Info {
    ttl: u32,
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
        write!(f, "{} {:.3} ms {}", self.ttl, self.tt, self.status_line)
    }
}

/// trace route of a host
#[derive(Parser, Debug)]
struct Args {
    /// Protocol to be used e.g. tcp
    #[clap(short, long, default_value = "tcp")]
    protocol: String,

    /// target host in IP v4 format
    #[clap(short, long)]
    target_host: String,
}
