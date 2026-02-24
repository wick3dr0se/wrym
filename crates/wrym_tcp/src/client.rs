use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::TcpStream;

use wrym_transport::{Reliability, Transport};

pub struct TcpTransport {
    stream: RefCell<TcpStream>,
    read_buffer: Vec<u8>,
}

impl TcpTransport {
    pub fn new(addr: &str) -> Self {
        let stream = TcpStream::connect(addr).unwrap();
        stream.set_nonblocking(true).unwrap();

        Self {
            stream: RefCell::new(stream),
            read_buffer: Vec::new(),
        }
    }
}

impl Transport for TcpTransport {
    fn recv(&mut self) -> Option<(String, Vec<u8>)> {
        let mut stream = self.stream.borrow_mut();
        let mut temp = [0u8; 1024];

        if let Ok(n) = stream.read(&mut temp) {
            if n > 0 {
                self.read_buffer.extend_from_slice(&temp[..n]);
            }
        }

        if self.read_buffer.len() < 4 {
            return None;
        }

        let len = u32::from_be_bytes([
            self.read_buffer[0],
            self.read_buffer[1],
            self.read_buffer[2],
            self.read_buffer[3],
        ]) as usize;

        if self.read_buffer.len() < 4 + len {
            return None;
        }

        let payload = self.read_buffer[4..4 + len].to_vec();
        self.read_buffer.drain(..4 + len);

        Some((stream.peer_addr().unwrap().to_string(), payload))
    }

    fn send_to(&self, _addr: &str, bytes: &[u8], _reliability: Reliability) {
        let mut stream = self.stream.borrow_mut();
        let len = (bytes.len() as u32).to_be_bytes();
        stream.write_all(&len).unwrap();
        stream.write_all(bytes).unwrap();
    }
}
