use std::{mem::MaybeUninit, net::SocketAddr};

use pnet::packet::{
    icmp::{echo_request::MutableEchoRequestPacket, IcmpCode, IcmpTypes},
    util, Packet,
};
use socket2::{SockAddr, Socket};

use crate::Status;

use super::protocol::AtrProtocol;

pub struct Icmp<'a> {
    socket: &'a Socket,
}

impl<'a> Icmp<'a> {
    pub fn new(socket: &'a Socket) -> Self {
        Self { socket }
    }

    fn create_echo_packet(&self, seq: u16) -> [u8; 28] {
        let mut buf = [0u8; 28];

        let mut echo_packet = MutableEchoRequestPacket::new(&mut buf).unwrap();
        echo_packet.set_icmp_type(IcmpTypes::EchoRequest);
        echo_packet.set_icmp_code(IcmpCode::new(0));
        echo_packet.set_identifier(0x1234);
        echo_packet.set_sequence_number(seq);
        echo_packet.set_checksum(util::checksum(echo_packet.packet(), 1));

        return buf;
    }
}

impl<'a> AtrProtocol for Icmp<'a> {
    fn send_packet(&self, ttl: u32, addr: SocketAddr, seq: u16) {
        self.socket.set_ttl(ttl).unwrap();

        let packet = self.create_echo_packet(seq);

        self.socket.send_to(&packet, &SockAddr::from(addr)).unwrap();
    }

    fn recv_packet(&self) -> (Status, String) {
        let mut rbuf: [MaybeUninit<u8>; 512] = unsafe { MaybeUninit::uninit().assume_init() };
        let resp = self.socket.recv_from(&mut rbuf);
        let icmp_type = unsafe { rbuf[20].assume_init() };
        match resp {
            Ok((_size, address)) => {
                if icmp_type == 11 {
                    (
                        Status::OK,
                        address.as_socket_ipv4().unwrap().ip().to_string(),
                    )
                } else {
                    (
                        Status::DONE,
                        address.as_socket_ipv4().unwrap().ip().to_string(),
                    )
                }
            }
            Err(_error) => (Status::ERR, "***".to_string()),
        }
    }
}
