use std::net::UdpSocket;

use async_trait::async_trait;
use wrym::client::Transport;

pub struct UdpTransport {
    socket: UdpSocket,
    server_addr: String
}

impl UdpTransport {
    pub fn new(addr: &str, server_addr: &str) -> Self {
        let socket = UdpSocket::bind(addr).unwrap();
        socket.set_nonblocking(true).unwrap();

        Self {
            socket,
            server_addr: server_addr.to_string()
        }
    }
}

#[async_trait]
impl Transport for UdpTransport {
    async fn send(&self, data: &[u8]) {
        self.socket.send_to(data, self.server_addr.to_owned()).unwrap();
    }

    async fn recv(&mut self) -> Option<Vec<u8>> {
        let mut buf = [0; 1024];

        if let Ok((len, _addr)) = self.socket.recv_from(&mut buf) {
            return Some(buf[..len].to_vec());
        }

        None
    }
}