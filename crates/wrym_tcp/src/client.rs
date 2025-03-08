use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::TcpStream;

use wrym_transport::Transport;

pub struct TcpTransport {
    stream: RefCell<TcpStream>,
}

impl TcpTransport {
    pub fn new(addr: &str) -> Self {
        let stream = TcpStream::connect(addr).unwrap();
        stream.set_nonblocking(true).unwrap();

        Self {
            stream: RefCell::new(stream),
        }
    }
}

impl Transport for TcpTransport {
    fn send_to(&self, _addr: &str, bytes: &[u8]) {
        let mut stream = self.stream.borrow_mut();
        stream.write_all(bytes).unwrap();
    }

    fn recv(&mut self) -> Option<(String, Vec<u8>)> {
        let mut buf = [0; 1024];
        let mut stream = self.stream.borrow_mut();

        if let Ok(len) = stream.read(&mut buf) {
            if len > 0 {
                return Some((stream.peer_addr().unwrap().to_string(), buf[..len].to_vec()));
            }
        }

        None
    }
}
