use std::net::SocketAddr;

use crate::Status;

pub trait AtrProtocol {
    fn send_packet(&self, ttl: u32, addr: SocketAddr, seq: u16);
    fn recv_packet(&self) -> (Status, String);
}
