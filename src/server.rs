use std::{collections::{HashMap, HashSet}, net::SocketAddr, time::Instant};

use mio::{net::UdpSocket, Events, Interest, Poll, Token};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use bincode::{deserialize, serialize};

use crate::message::MessageType;

const SERVER: Token = Token(0);

pub enum ServerEvent {
    ClientConnected(SocketAddr),
    ClientDisconnected(SocketAddr),
    MessageReceived(SocketAddr, Vec<u8>)
}

pub struct Server {
    event_rx: Receiver<ServerEvent>
}

impl Server {
    pub async fn new(addr: &str) -> Self {
        let socket = UdpSocket::bind(addr.parse().unwrap()).unwrap();

        let (event_tx, event_rx) = channel(100);
        let server = Self { event_rx };

        tokio::spawn(Self::poll(socket, event_tx));

        server
    }

    async fn poll(mut socket: UdpSocket, event_tx: Sender<ServerEvent>) {
        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(128);
        let mut clients = HashSet::new();
        let mut client_sequences = HashMap::new();
        let mut last_activity = HashMap::new();
        let mut buf = [0; 1024];

        poll.registry()
            .register(&mut socket, SERVER, Interest::READABLE)
            .unwrap();

        loop {
            poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                if event.token() == SERVER {
                    if let Ok((len, addr)) = socket.recv_from(&mut buf) {
                        let message = &buf[..len];

                        if clients.insert(addr) {
                            event_tx.send(ServerEvent::ClientConnected(addr)).await.unwrap();
                            println!("Client connected: {}", addr);
                        }

                        last_activity.insert(addr, Instant::now());

                        event_tx
                            .send(ServerEvent::MessageReceived(addr, message.to_vec()))
                            .await
                            .unwrap();

                        println!("Message received from {}: {:?}", addr, message);

                        Self::handle_message(&mut socket, addr, &message, &mut client_sequences).await;
                    }
                }
            }
        }
    }

    async fn handle_message(
        socket: &mut UdpSocket,
        addr: SocketAddr,
        msg: &[u8],
        client_sequences: &mut HashMap<SocketAddr, u32>,
    ) {
        if let Ok(message) = deserialize::<MessageType>(msg) {
            match message {
                MessageType::Unreliable(payload) => {
                    println!("Unreliable payload from {}: {:?}", addr, payload);
                }
                MessageType::Reliable(payload, seq) => {
                    let last_seq = client_sequences.entry(addr).or_insert(0);

                    if seq <= *last_seq && *last_seq != 0 {
                        println!("Duplicate message from {} with sequence {}", addr, seq);
                        return;
                    }

                    *last_seq = seq;

                    println!("Reliable ordered payload from {}: {:?}", addr, payload);

                    let ack = MessageType::Ack(seq);
                    socket.send_to(&serialize(&ack).unwrap(), addr).unwrap();
                }
                MessageType::ReliableUnordered(payload, seq) => {
                    println!("Reliable unordered payload from {}: {:?}", addr, payload);

                    let ack = MessageType::Ack(seq);
                    socket.send_to(&serialize(&ack).unwrap(), addr).unwrap();
                }
                _ => {}
            }
        }
    }

    pub async fn recv_events(&mut self) -> Option<ServerEvent> {
        self.event_rx.recv().await
    }
}
