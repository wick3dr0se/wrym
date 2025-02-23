use std::collections::VecDeque;

use wrym_transport::{Transport, ReliableTransport};

use crate::{into_opcode_message, OPCODE_CLIENT_CONNECTED, OPCODE_CLIENT_DISCONNECTED, OPCODE_MESSAGE};

pub enum ClientEvent {
    Connected,
    Disconnected,
    MessageReceived(Vec<u8>)
}

pub struct Client<T: Transport> {
    transport: T,
    server_addr: String,
    events: VecDeque<ClientEvent>
}

impl<T: Transport> Client<T> {
    pub fn new(transport: T, server_addr: &str) -> Self {
        let client =  Self {
            transport,
            server_addr: server_addr.to_string(),
            events: VecDeque::new()
        };

        client.transport.send_to(server_addr, &[OPCODE_CLIENT_CONNECTED]);

        client
    }

    pub fn poll(&mut self) {
        if let Some((_addr, mut bytes)) = self.transport.recv() {
            if bytes.is_empty() { return; }

            match bytes.remove(0) {
                OPCODE_CLIENT_CONNECTED => {
                    self.events.push_back(ClientEvent::Connected);
                }
                OPCODE_CLIENT_DISCONNECTED => {
                    self.events.push_back(ClientEvent::Disconnected);
                }
                OPCODE_MESSAGE => {
                    self.events.push_back(ClientEvent::MessageReceived(bytes));
                }
                _ => {}
            }
        }
    }

    pub fn recv_event(&mut self) -> Option<ClientEvent> {
        self.events.pop_front()
    }

    pub fn send(&self, bytes: &[u8]) {
        self.transport.send_to(&self.server_addr, &into_opcode_message(bytes));
    }

    pub fn disconnect(&self) {
        self.transport.send_to(&self.server_addr, &[OPCODE_CLIENT_DISCONNECTED]);
    }
}

impl<T: Transport + ReliableTransport> Client<T> {
    pub fn send_reliable(&self, bytes: &[u8], ordered: bool) {
        self.transport.send_reliable_to(
            &self.server_addr, &into_opcode_message(bytes), ordered
        );
    }
}

impl<T: Transport> Drop for Client<T> {
    fn drop(&mut self) {
        self.disconnect();
    }
}