### Description

This is a chat example where multiple participants can talk within the same room.

### What it does

1. Instantiates one Waku node
2. Starts the Waku node
3. Registers the node to waku events (messages, connection change, topic health, etc.)
4. Subscribes de node to the game_topic
5. Retrieves previous chat messages at the beginning

### How to run
From within the `examples/toy-chat/` folder, run the following to start a chat using the given nick name.

e.g.:
```code
cargo run "Alice"
```


