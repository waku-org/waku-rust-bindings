# Waku rust bindgen bindings

[<img alt="github" src="https://img.shields.io/badge/github-Github-red?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/logos-messaging/logos-delivery-rust-bindings)
[<img alt="crates.io" src="https://img.shields.io/crates/v/waku-bindings.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/waku-sys)
[<img alt="docs.rs" src="https://img.shields.io/badge/doc/waku-bindings-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/waku-sys)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/logos-messaging/logos-delivery-rust-bindings/main.yml?branch=master" height="20">](https://github.com/logos-messaging/logos-delivery-rust-bindings/actions/workflows/main.yml?query=branch%3Amaster)

Auto-generated Rust FFI bindings (via `bindgen`) on top of the
[logos-delivery C FFI](https://github.com/logos-messaging/logos-delivery/blob/master/library/libwaku.h).

## Usage

These are low-level auto-generated bindings. For a proper Rust API check [`waku-bindings`](https://crates.io/crates/waku-bindings).

Add this to your `Cargo.toml`:

```toml
[dependencies]
waku-sys = "1.0.0"
```


## About [Waku](https://waku.org/)

Waku is the communication layer for Web3. Decentralized communication that scales.

Private. Secure. Runs anywhere.

Read the [Waku docs](https://docs.waku.org/)
