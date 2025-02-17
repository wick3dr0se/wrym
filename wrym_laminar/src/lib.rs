use std::{sync::Arc, time::Instant};

use wrym::transport::{async_trait, ReliableTransport, Transport};
use laminar::{Packet, Socket, SocketEvent};
use tokio::{sync::Mutex, task};

pub struct LaminarTransport {
    socket: Arc<Mutex<Socket>>
}

impl LaminarTransport {
    pub fn new(bind_addr: &str) -> Self {
        let socket = Socket::bind(bind_addr).unwrap();
        let socket = Arc::new(Mutex::new(socket));

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
        socket.manual_poll(Instant::now())
    }

    async fn recv(&mut self) -> Option<(Vec<u8>, String)> {
        let socket = self.socket.clone();

        task::spawn(async move {
            socket.lock().await.manual_poll(Instant::now());
        });

        if let Some(event) = self.socket.lock().await.recv() {
            if let SocketEvent::Packet(packet) = event {
                return Some((packet.payload().to_vec(), packet.addr().to_string()));
            }
        }

        None
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
        socket.manual_poll(Instant::now());
    }
}