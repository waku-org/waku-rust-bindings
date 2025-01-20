### Description

This is a very simplistic example where two waku nodes are instantiated within the same Rust app.

### What it does

1. Instantiates two Waku nodes
2. Each node registers an event callback (waku message event, connection change event, etc.)
3. Each node starts
4. Each node perform relay subscription
5. "node1" publishes a waku message
6. Both nodes are stopped

### How to run
From within the `examples/basic/` foder run:
```code
cargo run
```

