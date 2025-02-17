use bincode::serialize;

use async_trait::async_trait;

#[async_trait]
pub trait Transport: 'static + Send + Sync {
    async fn send(&self, data: &[u8]);
    async fn recv(&mut self) -> Option<Vec<u8>>;
}

pub struct Client<T: Transport> {
    transport: T
}

impl<T: Transport> Client<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport
        }
    }

    pub async fn poll(&mut self) {
        if let Some(msg) = self.transport.recv().await {
            println!("Received message: {:?}", msg);
        }
    }

    pub async fn send(&self, msg: &[u8]) {
        self.transport.send(&serialize(msg).unwrap()).await;
    }
}