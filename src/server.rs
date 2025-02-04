use std::{collections::HashSet, net::SocketAddr, sync::Arc};
use bincode::{deserialize, serialize};
use tokio::{net::UdpSocket, sync::{mpsc::{channel, Receiver, Sender}, Mutex}};

use crate::{channel::RELIABLE_CHANNEL, message::MessageType};

pub enum ServerEvent {
    ClientConnected(SocketAddr),
    ClientDisconnected(SocketAddr),
    MessageReceived(SocketAddr, Vec<u8>)
}

pub struct Server {
    socket: Arc<UdpSocket>,
    clients: Arc<Mutex<HashSet<SocketAddr>>>,
    event_tx: Sender<ServerEvent>,
    event_rx: Receiver<ServerEvent>
}

impl Server {
    pub async fn new(addr: &str) -> Self {
        let (event_tx, event_rx) = channel(32);
        let socket = Arc::new(UdpSocket::bind(addr).await.unwrap());
        let clients = Arc::new(Mutex::new(HashSet::new()));

        let server = Self {
            socket: socket.clone(),
            clients: clients.clone(),
            event_tx: event_tx.clone(),
            event_rx
        };

        tokio::spawn(Self::poll(socket, clients, event_tx));

        server
    }

    async fn poll(
        socket: Arc<UdpSocket>,
        clients: Arc<Mutex<HashSet<SocketAddr>>>,
        event_tx: Sender<ServerEvent>
    ) {
        let mut buf = [0; 1024];
    
        loop {
            let socket = socket.clone();

            if let Ok((len, addr)) = socket.recv_from(&mut buf).await {
                let message = buf[..len].to_vec();
                let mut clients = clients.lock().await;
    
                if !clients.contains(&addr) {
                    clients.insert(addr);
                    event_tx.send(ServerEvent::ClientConnected(addr)).await.unwrap();
                    println!("[Server] Client connected: {}", addr);
                }
    
                event_tx.send(ServerEvent::MessageReceived(addr, message.clone())).await.unwrap();
                println!("[Server] Received message from {}: {:?}", addr, message);

                Self::handle_received(socket, addr, message).await;
            }
        }
    }

    pub async fn handle_received(socket: Arc<UdpSocket>, addr: SocketAddr, msg: Vec<u8>) {
        if let Ok(msg) = deserialize::<MessageType>(&msg) {
            match msg {
                MessageType::Unreliable(payload) => {
                    println!("[Server] Unreliable payload from {}: {:?}", addr, payload);
                }
                MessageType::Reliable(payload, seq) => {
                    let reliable_channel = RELIABLE_CHANNEL.read().await;
    
                    if reliable_channel.handle_reliable_message(addr, seq, payload.clone()).await.is_some() {
                        println!("[Server] Reliable ordered payload from {}: {:?}", addr, payload);
    
                        let ack_packet = MessageType::Ack(seq);

                        if let Ok(serialized_ack) = serialize(&ack_packet) {
                            if let Err(e) = socket.send_to(&serialized_ack, addr).await {
                                println!("[Server] Failed to send ACK to {}: {}", addr, e);
                            } else {
                                println!("[Server] Sent ACK for sequence {} to {}", seq, addr);
                            }
                        }
                    }
                }
                MessageType::ReliableUnordered(payload, seq) => {
                    println!("[Server] Reliable unordered payload from {}: {:?}", addr, payload);
    
                    let ack_packet = MessageType::Ack(seq);
                    
                    if let Ok(serialized_ack) = serialize(&ack_packet) {
                        if let Err(e) = socket.send_to(&serialized_ack, addr).await {
                            println!("[Server] Failed to send ACK to {}: {}", addr, e);
                        } else {
                            println!("[Server] Sent ACK for sequence {} to {}", seq, addr);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub async fn recv_events(&mut self) -> Option<ServerEvent> {
        self.event_rx.recv().await
    }

    pub async fn send_to(&self, addr: SocketAddr, message: &[u8]) {
        self.socket.send_to(&message, addr).await.unwrap();
    }

    pub async fn broadcast(&self, message: &[u8]) {
        for addr in self.clients.lock().await.iter() {
            self.send_to(*addr, message).await;
        }
    }

    pub async fn remove_client(&self, addr: SocketAddr) {
        if self.clients.lock().await.remove(&addr) {
            self.event_tx.send(ServerEvent::ClientDisconnected(addr)).await.unwrap();
        }
    }
}