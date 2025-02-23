use std::{process::exit, time::Duration};

use bincode::{deserialize, serialize};
use tokio:: time::sleep;
use wrym::client::{Client, ClientEvent};
#[cfg(feature = "udp")]
use wrym_udp::UdpTransport;
#[cfg(feature = "laminar")]
use wrym_laminar::LaminarTransport;
#[cfg(feature = "webtransport")]
use wrym_webtransport::client::WebTransport;

const SERVER_ADDR: &str = "127.0.0.1:8080";
const CLIENT_ADDR: &str = "127.0.0.1:0";

#[tokio::main]
async fn main() {
    #[cfg(feature = "udp")]
    let transport = UdpTransport::new(CLIENT_ADDR);
    #[cfg(feature = "laminar")]
    let transport = LaminarTransport::new(CLIENT_ADDR);
    #[cfg(feature = "webtransport")]
    let transport = WebTransport::new("https://[::1]:8080").await;
    let mut client = Client::new(transport, SERVER_ADDR);

    println!("Client is running on {}", CLIENT_ADDR);

    loop {
        client.poll();

        while let Some(event) = client.recv_event() {
            match event {
                ClientEvent::Connected => {
                    println!("Server {} acknowledged our connection!", SERVER_ADDR);

                    client.send(&serialize("Hello").unwrap());
                }
                ClientEvent::Disconnected => {
                    println!("Lost connection to server {}", SERVER_ADDR);

                    exit(0);
                }
                ClientEvent::MessageReceived(bytes) => {
                    println!("Message received from server: {:?}", deserialize::<String>(&bytes).unwrap());
                }
            }
        }

        sleep(Duration::from_millis(100)).await;
    }
}