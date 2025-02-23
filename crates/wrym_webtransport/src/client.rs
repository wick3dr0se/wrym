use tokio::runtime::Handle;
use wtransport::{ClientConfig, Connection, Endpoint};
use wrym_transport::Transport;

pub struct WebTransport {
    connection: Option<(String, Connection)>
}

impl WebTransport {
    pub async fn async_new(server_addr: &str) -> Self {
        let config = ClientConfig::default();
        let endpoint = Endpoint::client(config).unwrap();
        let connection = endpoint.connect(server_addr).await.unwrap();

        Self { connection: Some((server_addr.to_string(), connection)) }
    }

    pub fn new(server_addr: &str) -> Self {
        Handle::current().block_on(Self::async_new(server_addr))
    }

    pub async fn async_recv(&self) -> Option<(String, Vec<u8>)> {
        if let Some((addr, conn)) = &self.connection {
            match conn.receive_datagram().await {
                Ok(data) => return Some(( addr.to_owned(), data.payload().to_vec())),
                Err(_) => return None
            }
        }
        
        None
    }
}

impl Transport for WebTransport {
    fn send_to(&self, _addr: &str, bytes: &[u8]) {
        if let Some((_addr, conn)) = self.connection.as_ref() {
            conn.send_datagram(bytes.to_vec()).unwrap();
        }
    }

    fn recv(&mut self) -> Option<(String, Vec<u8>)> {
        Handle::current().block_on(self.async_recv())
    }
}