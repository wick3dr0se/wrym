use std::time::Duration;

use bincode::{deserialize, serialize};
use tokio::time::sleep;
use wrym::server::{Server, ServerEvent};
#[cfg(feature = "udp")]
use wrym_udp::UdpTransport;
#[cfg(feature = "laminar")]
use wrym_laminar::LaminarTransport;
#[cfg(feature = "webtransport")]
use wrym_webtransport::server::WebTransport;

const SERVER_ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() {
    #[cfg(feature = "udp")]
    let transport = UdpTransport::new(SERVER_ADDR);
    #[cfg(feature = "laminar")]
    let transport = LaminarTransport::new(SERVER_ADDR);
    #[cfg(feature = "webtransport")]
    let transport = WebTransport::new("some_cert.file", "some_key.file").await;

    let mut server = Server::new(transport);

    println!("Server is running on {}", SERVER_ADDR);

    loop {
        server.poll();

        while let Some(event) = server.recv_event() {
            match event {
                ServerEvent::ClientConnected(addr) => {
                    println!("New connection from client {}", addr);
                }
                ServerEvent::ClientDisconnected(addr) => {
                    println!("Lost connection from client {}", addr);
                }
                ServerEvent::MessageReceived(addr, bytes) => {
                    let msg = deserialize::<String>(&bytes).unwrap();
                    println!("Message received from client {}: {:?}", addr, msg);

                    server.broadcast(&serialize(&format!("Received '{}' from some client", msg)).unwrap());
                }
            }
        }

        sleep(Duration::from_millis(10)).await;
    }
}