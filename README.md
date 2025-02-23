# Wrym
**Wrym** is a cross-platform, lightweight and flexible networking library, designed to simplify the development of networked applications. It provides a unified interface for various transport layers and networking protocols, making it easy to build scalable and adaptable systems

## Features
- **Unified Transport Interface**: Supports multiple transport layers through a common `Transport` trait, enabling seamless integration with different protocols
- **Extensible and Modular**: Easily extend the library to support new protocols, features, or use cases, making it adaptable to a wide range of networking scenarios
- **Cross-Platform**: Works seamlessly in both native and web environments (e.g., WebAssembly)

## Getting Started
Get **wrym**
`cargo add --git https://github.com/wick3dr0se/wrym wrym`

Example: Basic server
```rust
use wrym::{server::{Server, ServerConfig}, transport::SomeTransport};

fn main() {
    let transport = SomeTransport::new("127.0.0.1:8080").unwrap();
    let mut server = Server::new(transpor, ServerConfig::default()t);

    loop {
        server.poll();

        while let Some(event) = server.recv_event() {
            // handle events
        }
    }
}
```

See more examples in [examples/](examples/)

## Contributing
If you have a feature request, bug report, or want to contribute code, please open an issue or submit a pull request

Contributions are much appreciated!
