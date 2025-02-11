use std::{
    net::SocketAddr, sync::Arc, time::{Duration, Instant}
};

use mio::{Events, Interest, Poll, Token};
use tokio::sync::Mutex;
use bincode::{deserialize, serialize};
use mio::net::UdpSocket;

use crate::message::MessageType;

const SERVER: Token = Token(0);

pub struct Client {
    socket: Arc<Mutex<UdpSocket>>,
    server_addr: SocketAddr,
    next_seq: u32,
    unacked_messages: Arc<Mutex<Vec<(u32, Vec<u8>, Instant, usize)>>>
}

pub enum ClientEvent {
    MessageSent(SocketAddr, Vec<u8>),
    AcknowledgmentReceived(SocketAddr, u32),
}

impl Client {
    pub fn new(addr: &str, server_addr: &str) -> Self {
        let socket = Arc::new(Mutex::new(UdpSocket::bind(addr.parse().unwrap()).unwrap()));
        let server_addr = server_addr.parse().unwrap();
        let unacked_messages = Arc::new(Mutex::new(Vec::new()));

        tokio::spawn(Self::poll(socket.clone(), server_addr, unacked_messages.clone()));

        let client = Self {
            socket,
            server_addr,
            next_seq: 0,
            unacked_messages
        };


        client
    }

    pub async fn poll(
        socket: Arc<Mutex<UdpSocket>>,
        server_addr: SocketAddr,
        unacked_messages: Arc<Mutex<Vec<(u32, Vec<u8>, Instant, usize)>>>
    ) {
        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(128);
        let mut buf = [0; 1024];
    
        {
            poll.registry()
                .register(&mut *socket.lock().await, SERVER, Interest::READABLE)
                .unwrap();
        }
    
        loop {
            poll.poll(&mut events, None).unwrap();
    
            for event in events.iter() {
                if event.token() == SERVER {
                    let sock = socket.lock().await;
    
                    if let Ok((len, _)) = sock.recv_from(&mut buf) {
                        let message = deserialize::<MessageType>(&buf[..len]).unwrap();
                        if let MessageType::Ack(seq) = message {
                            println!("Received ACK for sequence {}", seq);
    
                            unacked_messages.lock().await.retain(|(s, _, _, _)| *s != seq);
                        }
                    }
                }
            }
    
            Self::retransmit_unacked_messages(&socket, server_addr, unacked_messages.clone()).await;
        }
    }

    async fn retransmit_unacked_messages(
        socket: &Arc<Mutex<UdpSocket>>,
        server_addr: SocketAddr,
        unacked_messages: Arc<Mutex<Vec<(u32, Vec<u8>, Instant, usize)>>>
    ) {
        let now = Instant::now();
    
        for (seq, msg, sent_time, retries) in unacked_messages.lock().await.iter_mut() {
            if now.duration_since(*sent_time) > Duration::from_millis(200) {
                if *retries < 5 {
                    println!("Retransmitting message with sequence {}", seq);
    
                    let packet = MessageType::Reliable(msg.clone(), *seq);
                    socket
                        .lock()
                        .await
                        .send_to(&serialize(&packet).unwrap(), server_addr)
                        .unwrap();
    
                    *sent_time = Instant::now();
                    *retries += 1;
                }
            }
        }
    }

    pub async fn send_unreliable(&self, msg: &[u8]) {
        let packet = MessageType::Unreliable(msg.to_vec());
        println!("Sent unreliable message: {:?}", msg);

        self.socket
            .lock()
            .await
            .send_to(&serialize(&packet).unwrap(), self.server_addr)
            .unwrap();
    }

    pub async fn send_reliable(&mut self, msg: &[u8], ordered: bool) {
        let seq = self.next_seq;
        self.next_seq += 1;

        let packet = if ordered {
            println!("Sent reliable ordered message: {:?} with seq {}", msg, seq);
            MessageType::Reliable(msg.to_vec(), seq)
        } else {
            println!("Sent reliable unordered message: {:?} with seq {}", msg, seq);
            MessageType::ReliableUnordered(msg.to_vec(), seq)
        };

        self.socket
            .lock()
            .await
            .send_to(&serialize(&packet).unwrap(), self.server_addr)
            .unwrap();

        self.unacked_messages.lock().await.push((seq, msg.to_vec(), Instant::now(), 0));
    }
}
