# mini-redis

[![GitHub Actions workflow badge][github-actions-badge]][github-actions-url]

[github-actions-badge]: https://github.com/nukopy/mini-redis/actions/workflows/ci.yml/badge.svg?branch=main
[github-actions-url]: https://github.com/nukopy/mini-redis/actions/workflows/ci.yml?query=branch:main

Implementation of mini-redis on Tokio tutorial

## Environments

- Rust 1.69.0

## Commands

### How to Use Mini-Redis Server

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

### Dev

- Run my-mini-redis server

```sh
cargo run
```

- Run my-mini-redis server on watch mode

```sh
cargo watch -x run
```

- Run client code

```sh
cargo run --example test-mini-redis
cargo run --example test-mini-redis-concurrent
```

## References

- [(Zenn) Tokio チュートリアル (日本語訳)](https://zenn.dev/magurotuna/books/tokio-tutorial-ja)
- Official Resources
  - [Tokio Official Page](https://tokio.rs/)
  - [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
  - [github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio)
  - [github.com/tokio-rs/mini-redis](https://github.com/tokio-rs/mini-redis)

```

```
