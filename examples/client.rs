use wrym::client::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::new("127.0.0.1:0", "127.0.0.1:8080");

    client.send_reliable(b"Hello World!", true).await;
    client.send_reliable(b"test", false).await;
    client.send_unreliable(b"msg").await;
}