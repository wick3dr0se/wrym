use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use std::net::SocketAddr;

lazy_static! {
    pub static ref RELIABLE_CHANNEL: Arc<RwLock<ReliableChannel>> = Arc::new(RwLock::new(ReliableChannel::default()));
}

#[derive(Default)]
pub struct ReliableChannel {
    pub pending_messages: Arc<Mutex<HashMap<u32, Vec<u8>>>>,
    pub received_acks: Arc<Mutex<HashSet<u32>>>,
    pub sequence: Arc<Mutex<u32>>,
    pub client_sequences: Arc<Mutex<HashMap<SocketAddr, u32>>>
}

impl ReliableChannel {
    pub async fn next_sequence(&mut self) -> u32 {
        let mut seq = self.sequence.lock().await;
        *seq = seq.wrapping_add(1);

        *seq
    }

    pub async fn handle_reliable_message(&self, addr: SocketAddr, seq: u32, payload: Vec<u8>) -> Option<Vec<u8>> {
        let mut client_sequences = self.client_sequences.lock().await;
        let mut pending = self.pending_messages.lock().await;
        let expected_seq = client_sequences.get(&addr).unwrap_or(&0) + 1;
        let mut ordered_payloads = Vec::new();

        if seq < expected_seq {
            return None;
        }

        pending.insert(seq, payload);
        
        while let Some(msg) = pending.remove(&expected_seq) {
            ordered_payloads.push(msg);
            *client_sequences.entry(addr).or_insert(0) = expected_seq;
        }

        ordered_payloads.pop()
    }

    pub async fn acknowledge(&self, seq: u32) {
        let mut received_acks = self.received_acks.lock().await;
        let mut pending = self.pending_messages.lock().await;

        received_acks.insert(seq);
        pending.remove(&seq);
    }
}