use std::net::UdpSocket;

use async_trait::async_trait;
use wrym::server::Transport;

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
    async fn send_to(&self, data: &[u8], addr: &str) {
        self.socket.send_to(data, addr).unwrap();
    }

    async fn recv(&mut self) -> Option<(Vec<u8>, String)> {
        let mut buf = [0; 1024];

        if let Ok((len, addr)) = self.socket.recv_from(&mut buf) {
            return Some((buf[..len].to_vec(), addr.to_string()));
        }

        None
    }
}