use std::net::UdpSocket;

use wrym_transport::{Reliability, Transport};

pub struct UdpTransport {
    socket: UdpSocket,
}

impl UdpTransport {
    pub fn new(bind_addr: &str) -> Self {
        let socket = UdpSocket::bind(bind_addr).expect("Failed to bind UDP socket");
        socket
            .set_nonblocking(true)
            .expect("Failed to set nonblocking");

        Self { socket }
    }
}

impl Transport for UdpTransport {
    fn recv(&mut self) -> Option<(String, Vec<u8>)> {
        let mut buf = [0u8; 1500]; // safe MTU size

        match self.socket.recv_from(&mut buf) {
            Ok((len, addr)) => Some((addr.to_string(), buf[..len].to_vec())),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => None,
            Err(_) => None, // no disconnect concept in UDP
        }
    }

    fn send_to(&self, addr: &str, bytes: &[u8], _reliability: Reliability) -> std::io::Result<()> {
        self.socket.send_to(bytes, addr)?;
        Ok(())
    }
}
