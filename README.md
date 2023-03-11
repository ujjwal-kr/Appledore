[![progress-banner](https://app.codecrafters.io/progress/redis/9d77ff4d-47c9-42ef-b43b-819fee3d0980)](https://app.codecrafters.io/users/ujjwal-kr)

# Redis clone in rust

A toy Redis clone that's capable of handling
basic commands like `PING`, `SET` and `GET`, etc. Built to get familiar about
concurrent programming, implementing the redis protocol, and more. Created as a part of ["Build Your Own Redis" Challenge](https://codecrafters.io/challenges/redis).

![image](https://user-images.githubusercontent.com/38783809/221806792-74f4f4e2-c3b9-401e-bfe8-d80c70f7cf74.png)

## Supported Commands
- PING
- ECHO
- SET
- GET
- DEL
- LPUSH
- RPUSH
- LRANGE

Read the [Redis protocol specification](https://redis.io/docs/reference/protocol-spec/) here.

## Building

- Make sure to have redis installed. Run `sudo apt install redis` to install redis-server and redis-cli.
- Run `cargo run` to start the server. You can use the official [redis CLI](https://redis.io/docs/ui/cli/) to interact with it.

## TODO
- More array stuff    -- Done
- Persistent storage  -- Planning
- Organize code       -- Okay for now

A work in progress by [@ujjwal-kr](https://github.com/ujjwal-kr).
