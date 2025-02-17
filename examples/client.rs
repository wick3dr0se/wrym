use wrym::client::Client;
use wrym_udp::client::UdpTransport;
//use wrym_webtransport::client::WebTransport;

#[tokio::main]
async fn main() {
    let transport = UdpTransport::new("127.0.0.1:0", "127.0.0.1:8080");
    //let transport = WebTransport::new("https://[::1]:8080").await;
    let client = Client::new(transport);

    //loop {
    //    client.poll();

        client.send(b"test").await;
    //}
}