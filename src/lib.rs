pub mod server;
pub mod client;
pub mod transport {
    cfg_if::cfg_if! {
        if #[cfg(feature = "udp")] {
            pub use wrym_udp::UdpTransport;
        } else if #[cfg(feature = "laminar")] {
            pub use wrym_laminar::LaminarTransport;
        } else if #[cfg(feature = "webtransport")] {
            pub use wrym_webtransport::WebTransport;
        }
    }
}

const OPCODE_CLIENT_CONNECTED: u8 = 1;
const OPCODE_CLIENT_DISCONNECTED: u8 = 2;
const OPCODE_MESSAGE: u8 = 3;

pub(crate) fn into_opcode_message(bytes: &[u8]) -> Vec<u8> {
    let mut msg = vec![OPCODE_MESSAGE];
    msg.extend_from_slice(bytes);
    
    msg
}