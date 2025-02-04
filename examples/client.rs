use tokio::io::{AsyncBufReadExt, BufReader};
use wrym::client::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::new("127.0.0.1:0", "127.0.0.1:8080").await;
    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut buf = String::new();

    println!("Client is running on a random port");

    loop {
        println!("Enter a message: ");
        buf.clear();

        if stdin.read_line(&mut buf).await.is_ok() {
            let msg = buf.trim();
            
            if !msg.is_empty() {
                client.send_reliable(msg.as_bytes(), true).await;
                println!("Sent {:?} as bytes", msg);
            }
        }


    }
}