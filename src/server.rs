use std::{
    collections::{HashMap, VecDeque},
    sync::atomic::{AtomicU32, Ordering},
    time::{Duration, Instant},
};

use wrym_transport::{Reliability, Transport as ServerTransport};

use crate::Opcode;

#[cfg(feature = "laminar")]
pub use wrym_laminar::LaminarTransport as Transport;
#[cfg(feature = "tcp")]
pub use wrym_tcp::server::TcpTransport as Transport;
#[cfg(feature = "udp")]
pub use wrym_udp::UdpTransport as Transport;

pub struct ServerConfig {
    pub client_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            client_timeout: Duration::from_secs(60),
        }
    }
}

pub struct ClientData {
    id: u32,
    last_activity: Instant,
}

pub enum ServerEvent {
    ClientConnected(u32),
    ClientDisconnected(u32),
    MessageReceived(u32, Vec<u8>),
}

pub struct Server<T: ServerTransport> {
    transport: T,
    config: ServerConfig,
    clients: HashMap<String, ClientData>,
    events: VecDeque<ServerEvent>,
}

impl<T: ServerTransport> Server<T> {
    pub fn new(transport: T, config: ServerConfig) -> Self {
        Self {
            transport,
            config,
            clients: HashMap::new(),
            events: VecDeque::new(),
        }
    }

    fn next_id(&self) -> u32 {
        static COUNTER: AtomicU32 = AtomicU32::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    pub fn client_id(&self, addr: &str) -> Option<u32> {
        self.clients.get(addr).map(|c| c.id)
    }

    pub fn client_addr(&self, id: u32) -> Option<&String> {
        self.clients
            .iter()
            .find(|(_, c)| c.id == id)
            .map(|(addr, _)| addr)
    }

    fn add_client(&mut self, addr: &str) {
        if self.clients.contains_key(addr) {
            return;
        }

        let id = self.next_id();

        self.clients.insert(
            addr.to_string(),
            ClientData {
                id,
                last_activity: Instant::now(),
            },
        );

        let _ = self.transport.send_to(
            addr,
            &Opcode::ClientConnected.with_bytes(&id.to_le_bytes()),
            Reliability::ReliableOrdered { channel: 0 },
        );
        self.events.push_back(ServerEvent::ClientConnected(id));
    }

    fn drop_client(&mut self, addr: &str) {
        if let Some(data) = self.clients.remove(addr) {
            self.events
                .push_back(ServerEvent::ClientDisconnected(data.id));
            let _ = self.transport.send_to(
                addr,
                &[Opcode::ClientDisconnected as u8],
                Reliability::ReliableOrdered { channel: 0 },
            );
        }
    }

    fn drop_inactive_clients(&mut self, timeout: Duration) {
        let to_disconnect: Vec<String> = self
            .clients
            .iter()
            .filter_map(|(addr, data)| {
                if Instant::now().duration_since(data.last_activity) > timeout {
                    Some(addr.to_owned())
                } else {
                    None
                }
            })
            .collect();

        for addr in to_disconnect {
            self.drop_client(&addr);
        }
    }

    pub fn poll(&mut self) {
        self.transport.poll();
        self.drop_inactive_clients(self.config.client_timeout);

        while let Some((addr, mut bytes)) = self.transport.recv() {
            if bytes.is_empty() {
                self.drop_client(&addr);
                continue;
            }

            if let Some(data) = self.clients.get_mut(&addr) {
                data.last_activity = Instant::now();
            }

            match bytes.remove(0).into() {
                Opcode::ClientConnected => self.add_client(&addr),
                Opcode::ClientDisconnected => self.drop_client(&addr),
                Opcode::Message => {
                    if let Some(client) = self.clients.get(&addr) {
                        self.events
                            .push_back(ServerEvent::MessageReceived(client.id, bytes));
                    }
                }
            }
        }
    }

    pub fn recv_event(&mut self) -> Option<ServerEvent> {
        self.events.pop_front()
    }

    pub fn send_to(&mut self, addr: &str, bytes: &[u8], reliability: Reliability) {
        if self
            .transport
            .send_to(addr, &Opcode::Message.with_bytes(bytes), reliability)
            .is_err()
        {
            self.drop_client(addr);
        }
    }

    pub fn broadcast(&mut self, bytes: &[u8], reliability: Reliability) {
        let mut dead = Vec::new();
        for addr in self.clients.keys() {
            if self
                .transport
                .send_to(addr, &Opcode::Message.with_bytes(bytes), reliability)
                .is_err()
            {
                dead.push(addr.clone());
            }
        }
        for addr in dead {
            self.drop_client(&addr);
        }
    }
}
