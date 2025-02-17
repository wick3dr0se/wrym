use std::{collections::{HashMap, VecDeque}, time::Instant};

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

use async_trait::async_trait;

#[async_trait]
pub trait Transport: 'static + Send + Sync {
    async fn send_to(&self, data: &[u8], addr: &str);
    async fn recv(&mut self) -> Option<(Vec<u8>, String)>;
}

pub struct ClientData {
    last_activity: Instant
}

#[derive(Serialize, Deserialize)]
pub enum ServerEvent {
    ClientConnected(String),
    ClientDisconnected(String),
    MessageReceived(String, Vec<u8>)
}

pub struct Server<T: Transport> {
    transport: T,
    clients: HashMap<String, ClientData>,
    events: VecDeque<ServerEvent>
}

impl<T: Transport> Server<T> {
    pub fn new(transport: T) -> Self {        
        Self {
            transport,
            clients: HashMap::new(),
            events: VecDeque::new()
        }
    }

    pub async fn poll(&mut self) {
        if let Some((msg, addr)) = self.transport.recv().await {
            if self.clients
                .insert(addr.clone(), ClientData { last_activity: Instant::now() })
                .is_none()
            {
                self.events.push_back(ServerEvent::ClientConnected(addr.clone()));
            }
            
            self.events
                .push_back(ServerEvent::MessageReceived(addr, deserialize(&msg).unwrap()));
        }
    }

    pub fn recv_event(&mut self) -> Option<ServerEvent> {
        self.events.pop_front()
    }

    pub async fn send_to(&self, addr: &str, msg: &[u8]) {
        self.transport.send_to(&serialize(msg).unwrap(), addr).await;
    }
}