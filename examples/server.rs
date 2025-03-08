use std::{thread::sleep, time::Duration};

use bincode::{deserialize, serialize};
use wrym::{
    server::{Server, ServerConfig, ServerEvent},
    transport::server::Transport,
};

const SERVER_ADDR: &str = "127.0.0.1:8080";

fn main() {
    let transport = Transport::new(SERVER_ADDR);
    let mut server = Server::new(transport, ServerConfig::default());

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

                    server.broadcast(
                        &serialize(&format!("Received '{}' from client {}", msg, addr)).unwrap(),
                    );
                }
            }
        }

        sleep(Duration::from_millis(10));
    }
}
