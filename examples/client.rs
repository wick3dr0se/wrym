use wrym::client::Client;

#[tokio::main]
async fn main() {
    let client = Client::new("127.0.0.1:0", "127.0.0.1:8080").await;
    let message = "Hello World!";
    
    println!("Client is running on a random port");

    client.send(message.as_bytes()).await;
    println!("Sent {:?} as bytes", message);

    if let Some(resp) = client.recv().await {
        println!("Recieved: {:?}", resp);
    }
}