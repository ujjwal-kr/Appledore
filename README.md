# Appledore

High performant and lightweight redis compatible caching layer for use in the hackarmour search engine.

## Advantages:

- Only 12mb container size.
- Written in rust.
- Very minimal.
- Shipping utils from the redis-stack service which uses some weird opensource licencing. [WIP]

![image](https://user-images.githubusercontent.com/38783809/221806792-74f4f4e2-c3b9-401e-bfe8-d80c70f7cf74.png)

### Supported Commands
- PING
- ECHO
- SET
- GET
- DEL
- LPUSH
- RPUSH
- LRANGE
- LLEN
- LPOP
- LSET
- LINDEX
- LREM
- HSET

Read the [Redis protocol specification](https://redis.io/docs/reference/protocol-spec/) here.

## Building

- Make sure to have redis installed. Run `sudo apt install redis` to install redis-cli.
- Run `cargo run` to start the server. You can use the official [redis CLI](https://redis.io/docs/ui/cli/) to interact with it.
- To run using docker, first add the musl target using `rustup target add x86_64-unknown-linux-musl` and then run  `./start-docker.sh`.
- Stop the container using `./start-docker.sh stop`.

## TODO
- HASH
- JSON features

A work in progress by [@ujjwal-kr](https://github.com/ujjwal-kr).
