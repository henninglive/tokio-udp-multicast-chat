## tokio-udp-multicast-chat
This example implements a very simple CLI chat client which communicates over UDP multicast.
It’s implemented with the [tokio](https://github.com/tokio-rs/tokio) crate, an asynchronous
runtime for writing event-driven, non-blocking applications with the [Rust](https://www.rust-lang.org/).

The code is trivial, but there is one interesting
detail to note. To allow communication between instances on the same host we need to enable
`SO_REUSEADDR` for the UDP socket. The [tokio](https://github.com/tokio-rs/tokio) API
doesn’t expose a direct way to do this, we instead use the
[socket2](https://github.com/alexcrichton/socket2-rs) crate to construct a custom socket
that we upgrade to `std::net::UdpSocket` and then into a `tokio::net::UdpSocket`.


### Build and Run
1. Ensure you have current version of `cargo` and [Rust](https://www.rust-lang.org/) installed
2. Clone the project `$ git clone https://github.com/henninglive/tokio-udp-multicast-chat/ && cd tokio-udp-multicast-chat`
3. Build the project `$ cargo build --release` (NOTE: There is a large performance differnce when compiling without optimizations, so I recommend alwasy using `--release` to enable to them)
4. Once complete, the binary will be located at `target/release/tokio-udp-multicast-chat`
5. Use `$ cargo run --release` to build and then run, in one step
