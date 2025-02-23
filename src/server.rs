use std::{collections::{HashMap, VecDeque}, time::{Duration, Instant}};

use wrym_transport::{Transport, ReliableTransport};

use crate::{into_opcode_message, OPCODE_CLIENT_CONNECTED, OPCODE_CLIENT_DISCONNECTED, OPCODE_MESSAGE};

pub struct ServerConfig {
    pub client_timeout: Duration
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            client_timeout: Duration::from_secs(60)
        }
    }
}

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
    config: ServerConfig,
    clients: HashMap<String, ClientData>,
    events: VecDeque<ServerEvent>
}

impl<T: Transport> Server<T> {
    pub fn new(transport: T, config: ServerConfig) -> Self {
        Self {
            transport,
            config,
            clients: HashMap::new(),
            events: VecDeque::new()
        }
    }

    fn drop_inactive_clients(&mut self, timeout: Duration) {
        let to_disconnect: Vec<String> = self.clients.iter()
            .filter_map(|(addr, data)| {
                if Instant::now().duration_since(data.last_activity) > timeout {
                    Some(addr.to_owned())
                } else {
                    None
                }
            })
            .collect();
    
        for addr in to_disconnect {
            self.transport.send_to(&addr, &[OPCODE_CLIENT_DISCONNECTED]);
            self.clients.remove(&addr);
            self.events.push_back(ServerEvent::ClientDisconnected(addr));
        }
    }    

    pub fn poll(&mut self) {
        self.drop_inactive_clients(self.config.client_timeout);

        if let Some((addr, mut bytes)) = self.transport.recv() {
            if bytes.is_empty() { return; }

            match bytes.remove(0) {
                OPCODE_CLIENT_CONNECTED => {
                    if self.clients.insert(
                        addr.clone(), ClientData { last_activity: Instant::now() }
                    ).is_none() {
                        self.transport.send_to(&addr, &[OPCODE_CLIENT_CONNECTED]);
                        self.events.push_back(ServerEvent::ClientConnected(addr));
                    }
                }
                OPCODE_CLIENT_DISCONNECTED => {
                    if self.clients.remove(&addr).is_some() {
                        self.events.push_back(ServerEvent::ClientDisconnected(addr));
                    }
                }
                OPCODE_MESSAGE => {
                    self.events.push_back(ServerEvent::MessageReceived(addr, bytes));
                }
                _ => {}
            }
        }
    }

    pub fn recv_event(&mut self) -> Option<ServerEvent> {
        self.events.pop_front()
    }

    pub fn send_to(&self, addr: &str, bytes: &[u8]) {
        self.transport.send_to(addr, &into_opcode_message(bytes));
    }

    pub fn broadcast(&self, bytes: &[u8]) {
        let msg = into_opcode_message(bytes);

        for addr in self.clients.keys() {
            self.transport.send_to(addr, &msg);
        }
    }
}

impl<T: Transport + ReliableTransport> Server<T> {
    pub fn send_reliable_to(&self, addr: &str, bytes: &[u8], ordered: bool) {
        self.transport.send_reliable_to(addr, &into_opcode_message(bytes), ordered);
    }

    pub fn broadcast_reliable(&self, bytes: &[u8], ordered: bool) {
        let msg = into_opcode_message(bytes);
        
        for addr in self.clients.keys() {
            self.transport.send_reliable_to(addr, &msg, ordered)
        }
    }
}