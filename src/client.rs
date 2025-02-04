use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::UdpSocket, time::interval};
use bincode::{serialize, deserialize};

use crate::{channel::RELIABLE_CHANNEL, message::MessageType};

pub struct Client {
    pub socket: Arc<UdpSocket>,
    pub server_addr: SocketAddr
}

impl Client {
    pub async fn new(addr: &str, server_addr: &str) -> Self {        
        Self {
            socket: Arc::new(UdpSocket::bind(addr).await.unwrap()),
            server_addr: server_addr.parse().unwrap()
        }
    }
    
    pub async fn send_unreliable(&self, message: &[u8]) {
        let packet = MessageType::Unreliable(message.to_vec());
        println!("[Client] Sent unreliable message: {:?}", message);

        self.socket.send_to(&serialize(&packet).unwrap(), self.server_addr).await.unwrap();
    }

    pub async fn send_reliable(&mut self, message: &[u8], ordered: bool) {
        let reliable_channel = RELIABLE_CHANNEL.clone();
        let mut reliable_channel = reliable_channel.write().await;
        let seq = reliable_channel.next_sequence().await;
        let packet = if ordered {
            println!("[Client] Sent reliable message with sequence {}: {:?}", seq, message);
            MessageType::Reliable(message.to_vec(), seq)
        } else {
            println!("[Client] Sent reliable unordered message with sequence {}: {:?}", seq, message);
            MessageType::ReliableUnordered(message.to_vec(), seq)
        };
        
        self.socket.send_to(&serialize(&packet).unwrap(), self.server_addr).await.unwrap();

        if !reliable_channel.received_acks.lock().await.contains(&seq) {
            reliable_channel.pending_messages.lock().await.insert(seq, message.to_vec());

            drop(reliable_channel);

            self.schedule_retransmission(seq, ordered).await;
        }
    }

    async fn schedule_retransmission(&self, seq: u32, ordered: bool) {    
        let socket = self.socket.clone();
        let server_addr = self.server_addr;
        let pending_messages = RELIABLE_CHANNEL.read().await.pending_messages.clone();
        let received_acks = RELIABLE_CHANNEL.read().await.received_acks.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(200));
    
            for attempt in 1..=5 {
                println!("[Client] Checking for ACK for sequence {} (attempt {})", seq, attempt);
    
                if received_acks.lock().await.contains(&seq) {
                    println!("[Client] ACK received for sequence {}, stopping retransmissions", seq);
                    return;
                }

                if let Some(msg) = pending_messages.lock().await.get(&seq).cloned() {
                    let packet = if ordered {
                        MessageType::Reliable(msg, seq)
                    } else {
                        MessageType::ReliableUnordered(msg, seq)
                    };
    
                    println!("[Client] Retransmitting sequence {} (attempt {})", seq, attempt);

                    if let Err(e) = socket.send_to(&serialize(&packet).unwrap(), server_addr).await {
                        println!("[Client] Failed to retransmit sequence {}: {}", seq, e);
                    }
                }
    
                interval.tick().await;
            }
    
            if !received_acks.lock().await.contains(&seq) {
                println!("[Client] Max attempts reached for sequence {}", seq);
            }
        });
    }

    pub async fn recv(&self, timeout_secs: u64) -> Option<Vec<u8>> {
        let mut buf = [0; 1024];
    
        println!("[Client] Waiting for message with timeout {} secs", timeout_secs);
        
        let (len, _addr) = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            self.socket.recv_from(&mut buf)
        ).await.ok().unwrap().unwrap();
    
        println!("[Client] Received {} bytes", len);
        
        if let Ok(msg) = deserialize::<MessageType>(&buf[..len]) {
            return match msg {
                MessageType::Unreliable(payload)
                | MessageType::Reliable(payload, _)
                | MessageType::ReliableUnordered(payload, _) => Some(payload),
                MessageType::Ack(seq) => {
                    println!("[Client] Received ACK for sequence {}", seq);

                    let reliable_channel = RELIABLE_CHANNEL.clone();
                    let reliable_channel = reliable_channel.read().await;

                    reliable_channel.acknowledge(seq).await;
                    None
                }
            };
        }
    
        println!("[Client] Failed to deserialize message");
        None
    }
    
}