use crate::transport::{ReliableTransport, Transport};

pub struct Client<T: Transport> {
    transport: T,
    server_addr: String
}

impl<T: Transport> Client<T> {
    pub fn new(transport: T, server_addr: &str) -> Self {
        Self {
            transport,
            server_addr: server_addr.to_string()
        }
    }

    pub async fn poll(&mut self) {
        if let Some(bytes) = self.transport.recv().await {
            println!("Received message: {:?}", bytes);
        }
    }

    pub async fn send(&self, bytes: &[u8]) {
        self.transport.send_to(&self.server_addr, bytes).await;
    }
}

impl<T: Transport + ReliableTransport> Client<T> {
    pub async fn send_reliable(&self, bytes: &[u8], ordered: bool) {
        self.transport.send_reliable_to(&self.server_addr, bytes, ordered).await;
    }
}