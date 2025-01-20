### Description

This is a tic-tac-toe example that aims to show how to deal with
a Waku node in an app with UI. The example is very naïve and it
assumes only two tic-tac-toe instances are running globally. Therefore, the game messages might collide with other plays.

The game board is shown within a Rust eframe.

### What it does

1. Instantiates one Waku node
2. Starts the Waku node
3. Registers the node to waku events (messages, connection change, topic health, etc.)
4. Subscribes de node to the game_topic

### How to run
From within the `examples/tic-tac-toe/` foder run:
```code
cargo run
```

Another player can start their instance in either another
terminal or another machine.

