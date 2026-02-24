use std::collections::HashMap;
use std::net::UdpSocket;

use wrym_transport::{Reliability, Transport};

pub struct UdpTransport {
    socket: UdpSocket,
    read_buffers: HashMap<String, Vec<u8>>, // per remote address
}

impl UdpTransport {
    pub fn new(bind_addr: &str) -> Self {
        let socket = UdpSocket::bind(bind_addr).unwrap();
        socket.set_nonblocking(true).unwrap();

        Self {
            socket,
            read_buffers: HashMap::new(),
        }
    }
}

impl Transport for UdpTransport {
    fn poll(&mut self) {
        // UDP is stateless; no internal tick needed here
    }

    fn recv(&mut self) -> Option<(String, Vec<u8>)> {
        let mut buf = [0; 1024];

        if let Ok((len, addr)) = self.socket.recv_from(&mut buf) {
            let key = addr.to_string();
            let payload = &buf[..len];

            // For UDP, each recv is a full datagram — no need for framing
            // But we can optionally support length-prefixed messages if desired
            self.read_buffers
                .entry(key.clone())
                .or_default()
                .extend_from_slice(payload);

            // Here we just return the datagram as-is
            return Some((key, payload.to_vec()));
        }

        None
    }

    fn send_to(&self, addr: &str, bytes: &[u8], _reliability: Reliability) {
        self.socket.send_to(bytes, addr).unwrap();
    }
}
