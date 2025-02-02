use std::{collections::HashSet, net::SocketAddr, sync::Arc};
use tokio::{net::UdpSocket, sync::{mpsc::{channel, Receiver, Sender}, Mutex}};

pub enum ServerEvent {
    ClientConnected(SocketAddr),
    ClientDisconnected(SocketAddr)
}

pub struct Server {
    socket: Arc<UdpSocket>,
    clients: Arc<Mutex<HashSet<SocketAddr>>>,
    event_tx: Sender<ServerEvent>,
    event_rx: Receiver<ServerEvent>,
    msg_rx: Receiver<(SocketAddr, Vec<u8>)>
}

impl Server {
    pub async fn new(addr: &str) -> Self {
        let (event_tx, event_rx) = channel(32);
        let (msg_tx, msg_rx) = channel(32);
        let socket = Arc::new(UdpSocket::bind(addr).await.unwrap());
        let clients = Arc::new(Mutex::new(HashSet::new()));


        tokio::spawn(Self::poll(
            socket.clone(),
            clients.clone(),
            event_tx.clone(),
            msg_tx.clone()
        ));

        Self {
            socket,
            clients,
            event_tx,
            event_rx,
            msg_rx
        }
    }

    async fn poll(
        socket: Arc<UdpSocket>,
        clients: Arc<Mutex<HashSet<SocketAddr>>>,
        event_tx: Sender<ServerEvent>,
        msg_tx: Sender<(SocketAddr, Vec<u8>)>
    ) {
        let mut buf = [0; 1024];

        loop {
            if let Ok((len, addr)) = socket.recv_from(&mut buf).await {
                let message = buf[..len].to_vec();
                let mut clients = clients.lock().await;

                if !clients.contains(&addr) {
                    clients.insert(addr);
                    event_tx.send(ServerEvent::ClientConnected(addr)).await.unwrap();
                }

                msg_tx.send((addr, message)).await.unwrap();
            }
        }
    }

    pub async fn recv_messages(&mut self) -> Option<(SocketAddr, Vec<u8>)> {
        self.msg_rx.recv().await
    }

    pub async fn recv_events(&mut self) -> Option<ServerEvent> {
        self.event_rx.recv().await
    }

    pub async fn send_to(&self, addr: SocketAddr, message: &[u8]) {
        self.socket.send_to(&message, addr).await.unwrap();
    }

    pub async fn broadcast(&self, message: &[u8]) {
        for addr in self.clients.lock().await.iter() {
            self.send_to(*addr, message).await;
        }
    }

    pub async fn remove_client(&self, addr: SocketAddr) {
        if self.clients.lock().await.remove(&addr) {
            self.event_tx.send(ServerEvent::ClientDisconnected(addr)).await.unwrap();
        }
    }
}