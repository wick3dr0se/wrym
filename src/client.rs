use std::collections::VecDeque;

use wrym_transport::{Reliability, Transport as ClientTransport};

use crate::Opcode;

#[cfg(feature = "laminar")]
pub use wrym_laminar::LaminarTransport as Transport;
#[cfg(feature = "tcp")]
pub use wrym_tcp::client::TcpTransport as Transport;
#[cfg(feature = "udp")]
pub use wrym_udp::UdpTransport as Transport;

pub enum ClientEvent {
    Connected(u32),
    Disconnected,
    MessageReceived(Vec<u8>),
}

pub struct Client<T: ClientTransport> {
    transport: T,
    server_addr: String,
    id: Option<u32>,
    events: VecDeque<ClientEvent>,
}

impl<T: ClientTransport> Client<T> {
    pub fn new(transport: T, server_addr: &str) -> Self {
        let client = Self {
            transport,
            server_addr: server_addr.to_string(),
            id: None,
            events: VecDeque::new(),
        };

        let _ = client.transport.send_to(
            server_addr,
            &[Opcode::ClientConnected as u8],
            Reliability::ReliableOrdered { channel: 0 },
        );

        client
    }

    pub fn id(&self) -> Option<u32> {
        self.id
    }

    pub fn poll(&mut self) {
        self.transport.poll();
        while let Some((_addr, mut bytes)) = self.transport.recv() {
            if bytes.is_empty() {
                self.events.push_back(ClientEvent::Disconnected);
                break;
            }

            match bytes.remove(0).into() {
                Opcode::ClientConnected => {
                    if bytes.len() >= 4 {
                        let id = u32::from_le_bytes(bytes[..4].try_into().unwrap());
                        self.id = Some(id);

                        self.events.push_back(ClientEvent::Connected(id));
                    }
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

    pub fn send(&mut self, bytes: &[u8], reliability: Reliability) {
        if self
            .transport
            .send_to(
                &self.server_addr,
                &Opcode::Message.with_bytes(bytes),
                reliability,
            )
            .is_err()
        {
            self.events.push_back(ClientEvent::Disconnected);
        }
    }

    pub fn disconnect(&self) {
        let _ = self.transport.send_to(
            &self.server_addr,
            &[Opcode::ClientDisconnected as u8],
            Reliability::ReliableOrdered { channel: 0 },
        );
    }
}

impl<T: ClientTransport> Drop for Client<T> {
    fn drop(&mut self) {
        self.disconnect();
    }
}
