# tokio-tutorial

## Environments

- Rust 1.69.0

## Commands

### Mini-Redis Server

- Installation

```sh
cargo install mini-redis
```

- Run

```sh
mini-redis-server
```

- Read / Write

```sh
# Read
mini-redis-cli get key1
# (nil)

# Write
mini-redis-cli set key1 value1

# Read
mini-redis-cli get key1
# "value1"
```

## References

- [(Zenn) Tokio チュートリアル (日本語訳)](https://zenn.dev/magurotuna/books/tokio-tutorial-ja)
- Official Resources
  - [Tokio Official Page](https://tokio.rs/)
  - [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
  - [github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio)
  - [github.com/tokio-rs/mini-redis](https://github.com/tokio-rs/mini-redis)
