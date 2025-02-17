pub use async_trait::async_trait;

#[async_trait]
pub trait Transport: 'static + Send + Sync {
    async fn send_to(&self, addr: &str, data: &[u8]);
    async fn recv(&mut self) -> Option<(Vec<u8>, String)>;
}

#[async_trait]
pub trait ReliableTransport: 'static + Send + Sync {
    async fn send_reliable_to(&self, addr: &str, data: &[u8], ordered: bool);
}