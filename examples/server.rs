use wrym::server::{Server, ServerEvent};
use wrym_udp::server::UdpTransport;
//use wrym_webtransport::server::WebTransport;

#[tokio::main]
async fn main() {
    let transport = UdpTransport::new("127.0.0.1:8080");
    //let transport = WebTransport::new("some_cert.file", "some_key.file").await;
    let mut server = Server::new(transport);

    loop {
        server.poll().await;

        if let Some(event) = server.recv_event() {
            match event {
                ServerEvent::ClientConnected(addr) => {
                    println!("New connection from client {}", addr);
                }
                ServerEvent::MessageReceived(addr, msg) => {
                    println!("Message received from client {}: {:?}", addr, msg);
                }
                _ => {}
            }
        }
    }
}