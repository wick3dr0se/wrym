use wrym::client::Transport;
use wtransport::{ClientConfig, Connection, Endpoint};
use async_trait::async_trait;

pub struct WebTransport {
    connection: Option<Connection>
}

impl WebTransport {
    pub async fn new(server_addr: &str) -> Self {
        let config = ClientConfig::default();
        let endpoint = Endpoint::client(config).unwrap();
        let connection = endpoint.connect(server_addr).await.unwrap();

        Self {
            connection: Some(connection)
        }
    }
}

#[async_trait]
impl Transport for WebTransport {
    async fn send(&self, msg: &[u8]) {        
        if let Some(conn) = self.connection.as_ref() {
            conn.send_datagram(msg.to_vec()).unwrap();
        }
    }

    async fn recv(&mut self) -> Option<Vec<u8>> {
        if let Some(conn) = self.connection.as_mut() {
            match conn.receive_datagram().await {
                Ok(data) => return Some(data.payload().to_vec()),
                Err(_) => return None,
            }
        }
        None
    }
}
