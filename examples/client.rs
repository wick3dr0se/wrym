use bincode::serialize;
use wrym::client::Client;
//use wrym_udp::UdpTransport;
use wrym_laminar::LaminarTransport;
//use wrym_webtransport::client::WebTransport;

#[tokio::main]
async fn main() {
    //let transport = UdpTransport::new("127.0.0.1:0");
    let transport = LaminarTransport::new("127.0.0.1:0");
    //let transport = WebTransport::new("https://[::1]:8080").await;
    let mut client = Client::new(transport, "127.0.0.1:8080");

    //loop {
        client.poll().await;


        client.send_reliable(&serialize("Testinggg").unwrap(), false).await;
        client.send_reliable(&serialize("test").unwrap(), true).await;
        client.send(&serialize("Some_messages").unwrap()).await;
        client.send(&serialize("testingggg").unwrap()).await;
        client.send(&serialize("xxxxxxxx").unwrap()).await;
        client.send_reliable(&serialize("IS THIS WORKIGN??").unwrap(), true).await;
    //}
}