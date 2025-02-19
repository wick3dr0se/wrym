pub trait Transport {
    fn send_to(&self, addr: &str, bytes: &[u8]);
    fn recv(&mut self) -> Option<(String, Vec<u8>)>;
}

pub trait ReliableTransport {
    fn send_reliable_to(&self, addr: &str, bytes: &[u8], ordered: bool);
}