use std::{thread::sleep, time::Duration};

use bincode::{deserialize, serialize};
use wrym::{
    Reliability,
    server::{Server, ServerConfig, ServerEvent, Transport},
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
                ServerEvent::ClientConnected(id) => {
                    println!(
                        "New connection from client {} ({})",
                        id,
                        server.client_addr(id).unwrap()
                    );
                }
                ServerEvent::ClientDisconnected(id) => {
                    println!("Lost connection from client {}", id);
                }
                ServerEvent::MessageReceived(id, bytes) => {
                    let addr = server.client_addr(id).unwrap();
                    let msg = deserialize::<String>(&bytes).unwrap();
                    println!("Message received from client {} ({}): {:?}", id, addr, msg);

                    server.broadcast(
                        &serialize(&format!("Received '{}' from client {}", msg, addr)).unwrap(),
                        Reliability::Unreliable,
                    );
                }
            }
        }

        sleep(Duration::from_millis(10));
    }
}
