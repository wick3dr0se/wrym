use std::collections::HashMap;

use tokio::{runtime::Handle, task};
use wtransport::{Connection, Endpoint, Identity, ServerConfig, tls::{CertificateChain, PrivateKey}};
use wrym_transport::Transport;

pub struct WebTransport {
    connections: HashMap<String, Connection>
}

impl WebTransport {
    pub async fn new(cert: &str, key: &str) -> Self {
        let cert_chain = CertificateChain::load_pemfile(cert)
            .await
            .expect("Failed to load certificate chain");
        let private_key = PrivateKey::load_pemfile(key)
            .await
            .expect("Failed to load private key");
        let mut connections = HashMap::new();
        let identity = Identity::new(cert_chain, private_key);
        let config = ServerConfig::builder()
            .with_bind_default(4433)
            .with_identity(identity)
            .build();
        let endpoint = Endpoint::server(config)
            .expect("Failed to create endpoint");
        let transport = Self { connections: connections.clone() };
        let connection = endpoint.accept().await.await.unwrap().accept().await.unwrap();
        let addr = connection.remote_address();
        
        connections.insert(addr.to_string(), connection);

        transport
    }

    pub async fn async_recv(&self) -> Option<(String, Vec<u8>)> {
        for (addr, conn) in &self.connections {
            match conn.receive_datagram().await {
                Ok(data) => return Some((addr.to_owned(), data.payload().to_vec())),
                Err(_) => continue
            }
        }

        None
    }
}

impl Transport for WebTransport {
    fn send_to(&self, addr: &str, bytes: &[u8]) {        
        if let Some(conn) = self.connections.get(addr) {
            conn.send_datagram(bytes.to_vec()).unwrap();
        }
    }

    fn recv(&mut self) -> Option<(String, Vec<u8>)> {
        task::block_in_place(|| {
            Handle::current().block_on(self.async_recv())
        })
    }
}