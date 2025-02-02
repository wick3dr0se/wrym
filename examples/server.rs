use wrym::server::{Server, ServerEvent};

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let mut server = Server::new(addr).await;
    
    println!("Server is running on {}", addr);

    loop {
        if let Some((addr, msg)) = server.recv_messages().await {
            println!("Received {:?} from {}", msg, addr);
        }

        if let Some(event) = server.recv_events().await {
            match event {
                ServerEvent::ClientConnected(addr) => {
                    println!("Client connected: {}", addr);
                }
                ServerEvent::ClientDisconnected(addr) => {
                    println!("Client disconnected: {}", addr);
                }
            }
        }
    }
}
