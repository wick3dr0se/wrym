use std::net::UdpSocket;

use wrym_transport::{async_trait, Transport};

pub struct UdpTransport {
    socket: UdpSocket
}

impl UdpTransport {
    pub fn new(addr: &str) -> Self {
        let socket = UdpSocket::bind(addr).unwrap();
        socket.set_nonblocking(true).unwrap();

        Self { socket }
    }
}

#[async_trait]
impl Transport for UdpTransport {
    async fn send_to(&self, addr: &str, bytes: &[u8]) {
        self.socket.send_to(bytes, addr).unwrap();
    }

    async fn recv(&mut self) -> Option<(String, Vec<u8>)> {
        let mut buf = [0; 1024];

        if let Ok((len, addr)) = self.socket.recv_from(&mut buf) {
            return Some((addr.to_string(), buf[..len].to_vec()));
        }

        None
    }
}