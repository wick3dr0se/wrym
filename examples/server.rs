use std::time::Duration;

use bincode::{deserialize, serialize};
use wrym::server::{Server, ServerEvent};
//use wrym_udp::UdpTransport;
use wrym_laminar::LaminarTransport;
//use wrym_webtransport::server::WebTransport;

const SERVER_ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() {
    //let transport = UdpTransport::new(SERVER_ADDR);
    let transport = LaminarTransport::new(SERVER_ADDR);
    //let transport = WebTransport::new("some_cert.file", "some_key.file").await;
    let mut server = Server::new(transport);

    println!("Server is running on {}", SERVER_ADDR);

    loop {
        server.poll(Duration::from_secs(60)).await;

        if let Some(event) = server.recv_event() {
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

                    server.broadcast(&serialize(&format!("Received '{}' from some client", msg)).unwrap()).await;
                }
            }
        }
    }
}