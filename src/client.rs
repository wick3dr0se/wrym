use tokio::net::UdpSocket;
use std::net::SocketAddr;

pub struct Client {
    pub socket: UdpSocket,
    pub server_addr: SocketAddr
}

impl Client {
    pub async fn new(addr: &str, server_addr: &str) -> Self {
        let socket = UdpSocket::bind(addr).await.unwrap();
        let server_addr = server_addr.parse().unwrap();

        Self { socket, server_addr }
    }

    pub async fn send(&self, message: &[u8]) {        
        self.socket.send_to(message, self.server_addr).await.unwrap();
    }

    pub async fn recv(&self) -> Option<Vec<u8>> {
        let mut buf = [0; 1024];
        
        match self.socket.recv_from(&mut buf).await {
            Ok((len, _)) => Some(buf[..len].to_vec()),
            Err(_) => None
        }
    } 
}