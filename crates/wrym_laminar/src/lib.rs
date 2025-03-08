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
    fn send_to(&self, addr: &str, bytes: &[u8]) {
        let addr = addr.parse().unwrap();
        let packet = Packet::unreliable(addr, bytes.to_vec());

        self.socket.borrow_mut().send(packet).unwrap();
    }

    fn recv(&mut self) -> Option<(String, Vec<u8>)> {
        let mut socket = self.socket.borrow_mut();

        socket.manual_poll(Instant::now());

        if let Some(event) = socket.recv() {
            if let SocketEvent::Packet(packet) = event {
                return Some((packet.addr().to_string(), packet.payload().to_vec()));
            }
        }

        None
    }
}

impl ReliableTransport for LaminarTransport {
    fn send_reliable_to(&self, addr: &str, bytes: &[u8], ordered: bool) {
        let addr = addr.parse().unwrap();
        let packet = if ordered {
            Packet::reliable_ordered(addr, bytes.to_vec(), None)
        } else {
            Packet::reliable_unordered(addr, bytes.to_vec())
        };

        self.socket.borrow_mut().send(packet).unwrap();
    }
}
