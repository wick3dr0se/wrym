use std::time::Duration;

use bincode::{deserialize, serialize};
use tokio::{io::{AsyncBufReadExt, BufReader}, time};
use wrym::client::{Client, ClientEvent};
//use wrym_udp::UdpTransport;
use wrym_laminar::LaminarTransport;
//use wrym_webtransport::client::WebTransport;

const SERVER_ADDR: &str = "127.0.0.1:8080";
const CLIENT_ADDR: &str = "127.0.0.1:0";

#[tokio::main]
async fn main() {
    //let transport = UdpTransport::new(CLIENT_ADDR);
    let transport = LaminarTransport::new(CLIENT_ADDR);
    //let transport = WebTransport::new("https://[::1]:8080").await;
    let mut client = Client::new(transport, SERVER_ADDR);

    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut buf = String::new();

    println!("Client is running on {}", CLIENT_ADDR);

    loop {
        tokio::select! {
            _ = client.poll() => {
                if let Some(event) = client.recv_event() {
                    match event {
                        ClientEvent::MessageReceived(bytes) => {
                            println!("Message received from server: {:?}", deserialize::<String>(&bytes).unwrap());
                        }
                    }
                }
            }
            result = stdin.read_line(&mut buf) => {
                if result.is_ok() {
                    let msg = buf.trim();

                    if !msg.is_empty() {
                        client.send_reliable(&serialize(msg).unwrap(), true).await;
                    }

                    buf.clear();
                }
            }
            _ = time::sleep(Duration::from_millis(100)) => {}
        }
    }
}