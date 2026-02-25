#[derive(Copy, Clone)]
pub enum Reliability {
    Unreliable,
    ReliableUnordered,
    ReliableOrdered { channel: u8 },
}

pub trait Transport {
    fn poll(&mut self) {}
    fn recv(&mut self) -> Option<(String, Vec<u8>)>;
    fn send_to(&self, addr: &str, bytes: &[u8], reliability: Reliability) -> std::io::Result<()>;
}
