use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use wrym_transport::{Reliability, Transport};

pub struct TcpTransport {
    clients: HashMap<String, RefCell<TcpStream>>,
    listener: TcpListener,
    read_buffers: HashMap<String, Vec<u8>>,
}

impl TcpTransport {
    pub fn new(addr: &str) -> Self {
        let listener = TcpListener::bind(addr).unwrap();
        listener.set_nonblocking(true).unwrap();

        Self {
            clients: HashMap::new(),
            listener,
            read_buffers: HashMap::new(),
        }
    }

    fn accept_new_clients(&mut self) {
        while let Ok((stream, addr)) = self.listener.accept() {
            stream.set_nonblocking(true).unwrap();
            let key = addr.to_string();
            self.read_buffers.insert(key.clone(), Vec::new());
            self.clients.insert(key, RefCell::new(stream));
        }
    }
}

impl Transport for TcpTransport {
    fn poll(&mut self) {
        self.accept_new_clients();
    }

    fn recv(&mut self) -> Option<(String, Vec<u8>)> {
        for (addr, stream_cell) in &self.clients {
            let mut stream = stream_cell.borrow_mut();
            let mut temp = [0u8; 1024];

            if let Ok(n) = stream.read(&mut temp) {
                if n > 0 {
                    let buf = self.read_buffers.get_mut(addr).unwrap();
                    buf.extend_from_slice(&temp[..n]);

                    if buf.len() < 4 {
                        continue;
                    }

                    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;

                    if buf.len() < 4 + len {
                        continue;
                    }

                    let payload = buf[4..4 + len].to_vec();
                    buf.drain(..4 + len);

                    return Some((addr.clone(), payload));
                }
            }
        }

        None
    }

    fn send_to(&self, addr: &str, bytes: &[u8], _reliability: Reliability) {
        if let Some(stream_cell) = self.clients.get(addr) {
            let mut stream = stream_cell.borrow_mut();
            let len = (bytes.len() as u32).to_be_bytes();
            stream.write_all(&len).unwrap();
            stream.write_all(bytes).unwrap();
        }
    }
}
