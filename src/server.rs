use std::{collections::{HashMap, VecDeque}, time::{Duration, Instant}};

use wrym_transport::{ReliableTransport, Transport};

pub struct ClientData {
    last_activity: Instant
}

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

    fn drop_inactive_clients(&mut self, timeout: Duration) {
        self.clients.retain(|addr, data| {
            if Instant::now().duration_since(data.last_activity) > timeout {
                self.events.push_back(ServerEvent::ClientDisconnected(addr.clone()));
                
                false
            } else {
                true
            }
        });
    }

    pub async fn poll(&mut self, timeout: Duration) {
        self.drop_inactive_clients(timeout);

        if let Some((addr, bytes)) = self.transport.recv().await {
            if self.clients.insert(addr.to_string(), ClientData { last_activity: Instant::now() }).is_none() {
                self.events.push_back(ServerEvent::ClientConnected(addr.to_string()));
            }

            self.events.push_back(ServerEvent::MessageReceived(addr.to_string(), bytes));
        }
    }

    pub fn recv_event(&mut self) -> Option<ServerEvent> {
        self.events.pop_front()
    }

    pub async fn send_to(&self, addr: &str, bytes: &[u8]) {
        self.transport.send_to(addr, bytes).await;
    }

    pub async fn broadcast(&self, bytes: &[u8]) {
        for addr in self.clients.keys() {
            self.transport.send_to(addr, bytes).await;
        }
    }
}

impl<T: Transport + ReliableTransport> Server<T> {
    pub async fn send_reliable_to(&self, addr: &str, bytes: &[u8], ordered: bool) {
        self.transport.send_reliable_to(addr, bytes, ordered).await;
    }

    pub async fn broadcast_reliable(&self, bytes: &[u8], ordered: bool) {
        for addr in self.clients.keys() {
            self.transport.send_reliable_to(addr, bytes, ordered).await
        }
    }
}