use wrym::server::{Server, ServerEvent};

#[tokio::main]
async fn main() {
    let mut server = Server::new("127.0.0.1:8080").await;

    loop {
        if let Some(event) = server.recv_events().await {
            match event {
                ServerEvent::ClientConnected(addr) => {
                    println!("Connection received from client: {}", addr);
                }
                _ => {}
            }
        }
    }
}