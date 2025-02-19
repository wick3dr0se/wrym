use std::collections::VecDeque;

use wrym_transport::{Transport, ReliableTransport};

pub enum ClientEvent {
    MessageReceived(Vec<u8>)
}

pub struct Client<T: Transport> {
    transport: T,
    server_addr: String,
    events: VecDeque<ClientEvent>
}

impl<T: Transport> Client<T> {
    pub fn new(transport: T, server_addr: &str) -> Self {
        Self {
            transport,
            server_addr: server_addr.to_string(),
            events: VecDeque::new()
        }
    }

    pub fn poll(&mut self) {
        if let Some((_addr, bytes)) = self.transport.recv() {
            self.events.push_back(ClientEvent::MessageReceived(bytes));
        }
    }

    pub fn recv_event(&mut self) -> Option<ClientEvent> {
        self.events.pop_front()
    }

    pub fn send(&self, bytes: &[u8]) {
        self.transport.send_to(&self.server_addr, bytes);
    }
}

impl<T: Transport + ReliableTransport> Client<T> {
    pub fn send_reliable(&self, bytes: &[u8], ordered: bool) {
        self.transport.send_reliable_to(&self.server_addr, bytes, ordered);
    }
}