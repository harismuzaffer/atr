#![feature(io_error_more)]
#![feature(duration_millis_float)]
use core::fmt;
use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
    time::{Duration, Instant},
};

use clap::Parser;
use futures::{stream::FuturesUnordered, StreamExt};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::io::{unix::AsyncFd, Interest};

#[tokio::main]
async fn main() {
    println!("starting atr...");

    let args = Args::parse();

    let target_host = &args.target_host;
    let target_addrs = resolve_host_name(target_host);
    println!("target_ips are {:?}", target_addrs);
    let target_addr = target_addrs[0];

    let mut futures = FuturesUnordered::new();
    for ttl in 1..64 {
        let future = send_packet_t(ttl, target_addr);
        futures.push(future);
    }
    loop {
        tokio::select! {
            r = futures.next() => {
                match r.flatten() {
                    Some(i) => {
                        println!("Info: {}", i);
                    }
                    None => {
                        println!("No Info");
                        break;
                    }
                }
            }
            else => {
                println!("nothing matched, breaking");
                break
            },
        }
    }
}

fn resolve_host_name(host_name: &str) -> Vec<SocketAddr> {
    println!("hostname is {}", host_name);
    let ips: Vec<SocketAddr> = host_name.to_socket_addrs().unwrap().collect();
    ips
}

async fn send_packet_t(ttl: u32, addr: SocketAddr) -> Option<Info> {
    let timeout = Duration::from_secs(3);
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();
    socket.set_read_timeout(Some(timeout)).unwrap();
    socket.set_write_timeout(Some(timeout)).unwrap();
    socket.set_nonblocking(true).unwrap();
    socket.set_ttl(ttl).unwrap();
    let t_start = Instant::now();

    socket.connect(&SockAddr::from(addr));

    let async_fd = AsyncFd::new(socket).unwrap();
    let timeout = Duration::from_secs(3);
    let r = tokio::time::timeout(timeout, async {
        async_fd
            .async_io(Interest::WRITABLE, |socket| socket.take_error())
            .await
    })
    .await;

    match r {
        Err(_) => {
            println!("Timeout");
            None
        }
        Ok(result) => {
            let info = match result {
                Ok(None) => Info {
                    tt: t_start.elapsed().as_millis_f32(),
                    _status: Status::DONE,
                    status_line: String::from("DONE"),
                },
                Ok(Some(error_info)) => {
                    // println!("error info {:?}", error_info);
                    match error_info.kind() {
                        io::ErrorKind::ConnectionRefused => Info {
                            tt: t_start.elapsed().as_millis_f32(),
                            _status: Status::ERR,
                            status_line: String::from("*"),
                        },
                        io::ErrorKind::HostUnreachable => Info {
                            tt: t_start.elapsed().as_millis_f32(),
                            _status: Status::OK,
                            status_line: String::from("OK"),
                        },
                        _ => Info {
                            tt: t_start.elapsed().as_millis_f32(),
                            _status: Status::ERR,
                            status_line: String::from("***"),
                        },
                    }
                }
                Err(_) => Info {
                    tt: t_start.elapsed().as_millis_f32(),
                    _status: Status::ERR,
                    status_line: String::from("***"),
                },
            };
            Some(info)
        }
    }
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
