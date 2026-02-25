use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::TcpStream;

use wrym_transport::{Reliability, Transport};

pub struct TcpTransport {
    stream: RefCell<TcpStream>,
    read_buffer: Vec<u8>,
    disconnected: RefCell<bool>,
}

impl TcpTransport {
    pub fn new(addr: &str) -> Self {
        let stream = TcpStream::connect(addr).expect("Failed to connect to server");
        let _ = stream.set_nonblocking(true);

        Self {
            stream: RefCell::new(stream),
            read_buffer: Vec::new(),
            disconnected: RefCell::new(false),
        }
    }
}

impl Transport for TcpTransport {
    fn recv(&mut self) -> Option<(String, Vec<u8>)> {
        if *self.disconnected.borrow() {
            return None;
        }

        let mut stream = self.stream.borrow_mut();
        let mut temp = [0u8; 1024];

        match stream.read(&mut temp) {
            Ok(n) => {
                self.read_buffer.extend_from_slice(&temp[..n]);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => {
                *self.disconnected.borrow_mut() = true;
                return Some(("server".into(), Vec::new()));
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

        Some(("server".into(), payload))
    }

    fn send_to(&self, _addr: &str, bytes: &[u8], _reliability: Reliability) -> std::io::Result<()> {
        if *self.disconnected.borrow() {
            return Err(std::io::ErrorKind::NotConnected.into());
        }
        let mut stream = self.stream.borrow_mut();
        let len = (bytes.len() as u32).to_be_bytes();
        stream.write_all(&len).or_else(|e| {
            *self.disconnected.borrow_mut() = true;
            Err(e)
        })?;
        stream.write_all(bytes).or_else(|e| {
            *self.disconnected.borrow_mut() = true;
            Err(e)
        })?;
        Ok(())
    }
}
