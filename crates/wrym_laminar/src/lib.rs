use std::{sync::Arc, time::Instant};

use wrym_transport::{async_trait, ReliableTransport, Transport};
use laminar::{Packet, Socket, SocketEvent};
use tokio::{sync::Mutex, task::spawn_blocking};

pub struct LaminarTransport {
    socket: Arc<Mutex<Socket>>
}

impl LaminarTransport {
    pub fn new(bind_addr: &str) -> Self {
        let socket = Arc::new(Mutex::new(Socket::bind(bind_addr).unwrap()));

        Self { socket }
    }
}

#[async_trait]
impl Transport for LaminarTransport {
    async fn send_to(&self, addr: &str, bytes: &[u8]) {
        let addr = addr.parse().unwrap();
        let packet = Packet::unreliable(addr, bytes.to_vec());
        let mut socket = self.socket.lock().await;

        socket.send(packet).unwrap();
    }

    async fn recv(&mut self) -> Option<(String, Vec<u8>)> {
        let socket = self.socket.clone();

        socket.lock().await.manual_poll(Instant::now());

        spawn_blocking(move || {
            while let Some(event) = socket.blocking_lock().recv() {
                if let SocketEvent::Packet(packet) = event {
                    return Some((packet.addr().to_string(), packet.payload().to_vec()));
                }
            }

            None
        }).await.unwrap_or(None)
    }
}

#[async_trait]
impl ReliableTransport for LaminarTransport {
    async fn send_reliable_to(&self, addr: &str, bytes: &[u8], ordered: bool) {
        let addr = addr.parse().unwrap();
        let packet = if ordered {
            Packet::reliable_ordered(addr, bytes.to_vec(), None)  
        } else {
            Packet::reliable_unordered(addr, bytes.to_vec())
        };
        let mut socket = self.socket.lock().await;
        
        socket.send(packet).unwrap();
    }
}