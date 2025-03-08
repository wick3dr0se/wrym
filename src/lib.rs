pub mod client;
pub mod server;
pub mod transport {
    cfg_if::cfg_if! {
        if #[cfg(feature = "tcp")] {
            pub mod server {
                pub use wrym_tcp::server::TcpTransport as Transport;
            }
            pub mod client {
                pub use wrym_tcp::client::TcpTransport as Transport;
            }
        } else if #[cfg(feature = "udp")] {
            pub use wrym_udp::UdpTransport as Transport;
        } else if #[cfg(feature = "laminar")] {
            pub use wrym_laminar::LaminarTransport as Transport;
        } else if #[cfg(feature = "webtransport")] {
            pub mod server {
                pub use wrym_webtransport::server::WebTransport as Transport;
            }
            pub mod client {
                pub use wrym_webtransport::client::WebTransport as Transport;
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Opcode {
    ClientConnected = 1,
    ClientDisconnected = 2,
    Message = 3,
}

impl From<u8> for Opcode {
    fn from(opcode: u8) -> Self {
        match opcode {
            1 => Opcode::ClientConnected,
            2 => Opcode::ClientDisconnected,
            3 => Opcode::Message,
            _ => panic!("Invalid opcode: {}", opcode),
        }
    }
}

impl Opcode {
    pub(crate) fn with_bytes(&self, bytes: &[u8]) -> Vec<u8> {
        let mut opcode_bytes = vec![*self as u8];
        opcode_bytes.extend_from_slice(bytes);
        opcode_bytes
    }
}
