# Waku Rust bindings

[<img alt="github" src="https://img.shields.io/badge/github-Github-red?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/logos-messaging/logos-delivery-rust-bindings)
[<img alt="crates.io" src="https://img.shields.io/crates/v/waku-bindings.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/waku-bindings)
[<img alt="docs.rs" src="https://img.shields.io/badge/doc/waku-bindings-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/waku-bindings)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/logos-messaging/logos-delivery-rust-bindings/main.yml?branch=master" height="20">](https://github.com/logos-messaging/logos-delivery-rust-bindings/actions/workflows/main.yml?query=branch%3Amaster)

High-level Rust API on top of [`waku-sys`](https://crates.io/crates/waku-sys) bindgen bindings to the
[logos-delivery C FFI](https://github.com/logos-messaging/logos-delivery/blob/master/library/libwaku.h).


## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
waku-bindings = "1.0.0"
```


## Testing

### Prerequisites

- **Rust** (stable) — [rustup.rs](https://rustup.rs)
- **Nim 2.x** — required to compile the native `libwaku` library
- **Make**
- **GCC or Clang**

### Setup

Clone the repository:

```bash
git clone https://github.com/logos-messaging/logos-delivery-rust-bindings.git
cd logos-delivery-rust-bindings
```

The first `cargo build` / `cargo test` automatically:
1. Initializes and updates git submodules
2. Builds the native `libwaku` static library via `make libwaku STATIC=1`
3. Generates Rust FFI bindings via `bindgen`

### Running the tests

From the repo root or the `waku-bindings/` directory:

```bash
cargo test
```

> Tests must run **serially** — they spin up real Waku nodes that bind to local TCP ports. Running them in
> parallel causes port conflicts. The `serial_test` crate enforces this automatically.

To run a specific test:

```bash
cargo test <test_name>
```

For example:

```bash
cargo test default_echo
cargo test node_restart
```

To see log output:

```bash
cargo test -- --nocapture
```

### What the tests cover

| Test | Description |
|------|-------------|
| `default_echo` | Creates two nodes, subscribes both to a relay topic, publishes a message from node 1, and verifies node 2 receives it |
| `node_restart` | Creates a node, starts and stops it three times in a row to verify lifecycle correctness |


## About [Waku](https://waku.org/)

Waku is the communication layer for Web3. Decentralized communication that scales.

Private. Secure. Runs anywhere.

Waku is a suite of privacy-preserving, peer-to-peer messaging protocols.

Waku removes centralized third parties from messaging, enabling private, secure, censorship-free communication with no single point of failure.

Waku provides privacy-preserving capabilities, such as sender anonymity, metadata protection and unlinkability to personally identifiable information.

Waku is designed for generalized messaging, enabling human-to-human, machine-to-machine or hybrid communication.

Waku runs everywhere: desktop, server, including resource-restricted devices, such as mobile devices and browsers.

The first version of Waku had its origins in the Whisper protocol, with optimizations for scalability and usability. Waku v2 is a complete rewrite. Its relay protocol implements pub/sub over libp2p, and also introduces additional capabilities:

1. Retrieving historical messages for mostly-offline devices.
2. Adaptive nodes, allowing for heterogeneous nodes to contribute.
3. Bandwidth preservation for light nodes.

This makes it ideal for running a p2p protocol on mobile, or in other similarly resource-restricted environments.

Read the [Waku docs](https://docs.waku.org/)
