use std::{cell::RefCell, time::Instant};

use laminar::{Packet, Socket, SocketEvent};
use wrym_transport::{ReliableTransport, Transport};

pub struct LaminarTransport {
    socket: RefCell<Socket>,
}

impl LaminarTransport {
    pub fn new(bind_addr: &str) -> Self {
        let socket = RefCell::new(Socket::bind(bind_addr).unwrap());
        Self { socket }
    }
}

impl Transport for LaminarTransport {
    fn poll(&mut self) {
        self.socket.borrow_mut().manual_poll(Instant::now());
    }

    fn recv(&mut self) -> Option<(String, Vec<u8>)> {
        if let Some(event) = self.socket.borrow_mut().recv() {
            if let SocketEvent::Packet(packet) = event {
                return Some((packet.addr().to_string(), packet.payload().to_vec()));
            }
        }

        None
    }

    fn send_to(&self, addr: &str, bytes: &[u8]) {
        let packet = Packet::unreliable(addr.parse().unwrap(), bytes.to_vec());
        self.socket.borrow_mut().send(packet).unwrap();
    }
}

impl ReliableTransport for LaminarTransport {
    fn send_reliable_to(&self, addr: &str, bytes: &[u8], channel: Option<u8>) {
        let addr = addr.parse().unwrap();
        let packet = match channel {
            Some(ch) => Packet::reliable_ordered(addr, bytes.to_vec(), Some(ch)),
            None => Packet::reliable_unordered(addr, bytes.to_vec()),
        };
        self.socket.borrow_mut().send(packet).unwrap();
    }
}
