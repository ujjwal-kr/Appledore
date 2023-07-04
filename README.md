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
Make sure to have redis installed. You may need [redis CLI](https://redis.io/docs/ui/cli/) to interact with appledore. 

### Dev mode
- Run `cargo run` to start the server.

### Prod
- Run the `./start-docker.sh` script.
- `./start-docker stop` to stop the container.

## TODO
- HASH
- JSON features

A work in progress by [@ujjwal-kr](https://github.com/ujjwal-kr).
