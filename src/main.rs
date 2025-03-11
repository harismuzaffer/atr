#![feature(io_error_more)]
#![feature(duration_millis_float)]

mod protocols {
    pub mod icmp;
    pub mod protocol;
    pub mod tcp;
}

use core::fmt;
use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
    time::{Duration, Instant},
};

use clap::Parser;
use protocols::{icmp::Icmp, protocol::AtrProtocol};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

#[tokio::main]
async fn main() {
    println!("starting atr...");

    let args = Args::parse();

    let target_host = &args.target_host;
    let target_addrs = resolve_host_name(target_host);
    println!("target_ips are {:?}", target_addrs);
    let target_addr = target_addrs[0];

    let socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4)).unwrap();
    let timeout = Duration::from_millis(300);
    socket.set_read_timeout(Some(timeout)).unwrap();
    socket.set_write_timeout(Some(timeout)).unwrap();
    for ttl in 1..64 {
        let t_start = Instant::now();
        let atr_icmp = Icmp::new(&socket);
        atr_icmp.send_packet(ttl, target_addr, ttl as u16);
        let (status, src) = atr_icmp.recv_packet();

        let info = Info {
            ttl,
            tt: t_start.elapsed().as_millis_f32(),
            status,
            source: src,
        };

        println!("{}", info);

        if status == Status::DONE {
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
            status: Status::DONE,
            source: "".to_string(),
        },
        Err(error) => {
            let info = match error.kind() {
                io::ErrorKind::ConnectionRefused => Info {
                    ttl,
                    tt: t_start.elapsed().as_millis_f32(),
                    status: Status::ERR,
                    source: "".to_string(),
                },
                io::ErrorKind::HostUnreachable => Info {
                    ttl,
                    tt: t_start.elapsed().as_millis_f32(),
                    status: Status::OK,
                    source: "".to_string(),
                },
                _ => Info {
                    ttl,
                    tt: t_start.elapsed().as_millis_f32(),
                    status: Status::ERR,
                    source: "".to_string(),
                },
            };
            info
        }
    }
}

struct Info {
    ttl: u32,
    tt: f32,
    status: Status,
    source: String,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Status {
    DONE,
    OK,
    ERR,
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {:?} {:.3} ms",
            self.ttl, self.source, self.status, self.tt
        )
    }
}

/// trace route of a host
#[derive(Parser, Debug)]
struct Args {
    /// Protocol to be used e.g. tcp
    #[clap(short, long, default_value = "icmp")]
    protocol: String,

    /// target host in IP v4 format
    #[clap(short, long)]
    target_host: String,
}
