use std::{process::exit, thread::sleep, time::Duration};

use bincode::{deserialize, serialize};
use wrym::{
    client::{Client, ClientEvent},
    transport::client::Transport,
};

const SERVER_ADDR: &str = "127.0.0.1:8080";
const CLIENT_ADDR: &str = "127.0.0.1:0";

fn main() {
    let transport = Transport::new(SERVER_ADDR);
    let mut client = Client::new(transport, SERVER_ADDR);

    println!("Client is running on {}", CLIENT_ADDR);

    loop {
        client.poll();

        while let Some(event) = client.recv_event() {
            match event {
                ClientEvent::Connected(_id) => {
                    println!("Server {} acknowledged our connection!", SERVER_ADDR);

                    client.send(&serialize("Hello").unwrap());
                }
                ClientEvent::Disconnected => {
                    println!("Lost connection to server {}", SERVER_ADDR);

                    exit(0);
                }
                ClientEvent::MessageReceived(bytes) => {
                    println!(
                        "Message received from server: {:?}",
                        deserialize::<String>(&bytes).unwrap()
                    );
                }
            }
        }

        sleep(Duration::from_millis(100));
    }
}
