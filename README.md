# Waku Rust bindings

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![Build Status][actions-badge]][actions-url]
[![Codecov Status][codecov-badge]][codecov-url]

[crates-badge]: https://img.shields.io/crates/v/waku-bindings.svg
[crates-url]: https://crates.io/crates/waku-bindings
[docs-badge]: https://docs.rs/waku-bindings/badge.svg
[docs-url]: https://docs.rs/waku-bindings
[actions-badge]: https://github.com/logos-messaging/logos-messaging-rust-bindings/workflows/CI/badge.svg
[actions-url]: https://github.com/logos-messaging/logos-messaging-rust-bindings/actions/workflows/main.yml?query=workflow%3ACI+branch%3Amaster
[codecov-badge]: https://codecov.io/github/logos-messaging/logos-messaging-rust-bindings/branch/main/graph/badge.svg?token=H4CQWRUCUS
[codecov-url]: https://codecov.io/github/logos-messaging/logos-messaging-rust-bindings

Rust layer on top of [`logos-messaging-nim`](https://github.com/logos-messaging/logos-messaging-nim) [C FFI bindings](https://github.com/logos-messaging/logos-messaging-nim/blob/master/library/libwaku.h).


## Using the crate

For now this crate is consumed from a local checkout rather than a registry.

1. Clone the repo and check out the branch you want:

   ```sh
   git clone --recurse-submodules https://github.com/logos-messaging/logos-delivery-rust-bindings
   cd logos-delivery-rust-bindings
   git checkout <branch>
   git submodule update --init --recursive
   ```

2. Point your app at it by path:

   ```toml
   # your app's Cargo.toml
   [dependencies]
   waku-bindings = { path = "../logos-delivery-rust-bindings/waku-bindings" }
   ```

### Build prerequisites

`waku-bindings` builds the underlying Nim library from source the first time,
so the toolchain has to be present:

- **Nim** and **nimble** (installed by the vendored `make` targets, or bring
  your own on `PATH`)
- **make** and a C/C++ toolchain
- A **Rust** toolchain (RLN is built from source)

The first build compiles the whole logos-delivery tree and is slow; subsequent
builds are incremental.

## What you can call

Everything is driven through `LogosDeliveryCtx` (re-exported at the crate root).
The crate docs group the available operations — node lifecycle, messaging,
reliable channels, and the typed event listeners — under **Operations**; run
`cargo doc --open -p waku-bindings` for the full, linked method list. A minimal
end-to-end example lives in [`examples/basic`](examples/basic).

## About Waku

[Waku](https://waku.org/) is a family of robust and censorship-resistant communication protocols enabling privacy-focused messaging for Web3 applications.

Private. Secure. Runs anywhere.

Read the [Waku docs](https://docs.waku.org/)
