use std::collections::VecDeque;

use wrym_transport::{ReliableTransport, Transport};

use crate::Opcode;

pub enum ClientEvent {
    Connected,
    Disconnected,
    MessageReceived(Vec<u8>),
}

pub struct Client<T: Transport> {
    transport: T,
    server_addr: String,
    events: VecDeque<ClientEvent>,
}

impl<T: Transport> Client<T> {
    pub fn new(transport: T, server_addr: &str) -> Self {
        let client = Self {
            transport,
            server_addr: server_addr.to_string(),
            events: VecDeque::new(),
        };

        client
            .transport
            .send_to(server_addr, &[Opcode::ClientConnected as u8]);

        client
    }

    pub fn poll(&mut self) {
        if let Some((_addr, mut bytes)) = self.transport.recv() {
            if bytes.is_empty() {
                return;
            }

            match bytes.remove(0).into() {
                Opcode::ClientConnected => {
                    self.events.push_back(ClientEvent::Connected);
                }
                Opcode::ClientDisconnected => {
                    self.events.push_back(ClientEvent::Disconnected);
                }
                Opcode::Message => {
                    self.events.push_back(ClientEvent::MessageReceived(bytes));
                }
            }
        }
    }

    pub fn recv_event(&mut self) -> Option<ClientEvent> {
        self.events.pop_front()
    }

    pub fn send(&self, bytes: &[u8]) {
        self.transport
            .send_to(&self.server_addr, &Opcode::Message.with_bytes(bytes));
    }

    pub fn disconnect(&self) {
        self.transport
            .send_to(&self.server_addr, &[Opcode::ClientDisconnected as u8]);
    }
}

impl<T: Transport + ReliableTransport> Client<T> {
    pub fn send_reliable(&self, bytes: &[u8], ordered: bool) {
        self.transport.send_reliable_to(
            &self.server_addr,
            &Opcode::Message.with_bytes(bytes),
            ordered,
        );
    }
}

impl<T: Transport> Drop for Client<T> {
    fn drop(&mut self) {
        self.disconnect();
    }
}
