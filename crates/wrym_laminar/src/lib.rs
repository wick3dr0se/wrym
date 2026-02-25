use std::{cell::RefCell, time::Instant};

use laminar::{Packet, Socket, SocketEvent};
use wrym_transport::{Reliability, Transport};

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

    fn send_to(&self, addr: &str, bytes: &[u8], reliability: Reliability) -> std::io::Result<()> {
        let addr = addr.parse().unwrap();
        let packet = match reliability {
            Reliability::Unreliable => Packet::unreliable(addr, bytes.to_vec()),
            Reliability::ReliableUnordered => Packet::reliable_unordered(addr, bytes.to_vec()),
            Reliability::ReliableOrdered { channel } => {
                Packet::reliable_ordered(addr, bytes.to_vec(), Some(channel))
            }
        };
        self.socket.borrow_mut().send(packet).ok();
        Ok(())
    }
}
