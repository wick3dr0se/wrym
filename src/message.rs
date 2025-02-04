use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum MessageType {
    Unreliable(Vec<u8>),
    Reliable(Vec<u8>, u32),
    ReliableUnordered(Vec<u8>, u32),
    Ack(u32)
}