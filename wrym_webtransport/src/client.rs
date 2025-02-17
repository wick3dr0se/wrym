use wrym::transport::{async_trait, Transport};
use wtransport::{ClientConfig, Connection, Endpoint};

pub struct WebTransport {
    connection: Option<(String, Connection)>
}

impl WebTransport {
    pub async fn new(server_addr: &str) -> Self {
        let config = ClientConfig::default();
        let endpoint = Endpoint::client(config).unwrap();
        let connection = endpoint.connect(server_addr).await.unwrap();

        Self {
            connection: Some((server_addr.to_string(), connection))
        }
    }
}

#[async_trait]
impl Transport for WebTransport {
    async fn send_to(&self, _addr: &str, bytes: &[u8]) {
        if let Some((_addr, conn)) = self.connection.as_ref() {
            conn.send_datagram(bytes.to_vec()).unwrap();
        }
    }

    async fn recv(&mut self) -> Option<(Vec<u8>, String)> {
        if let Some((addr, conn)) = self.connection.as_mut() {
            match conn.receive_datagram().await {
                Ok(data) => return Some((data.payload().to_vec(), addr.to_owned())),
                Err(_) => return None
            }
        }
        None
    }
}
