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