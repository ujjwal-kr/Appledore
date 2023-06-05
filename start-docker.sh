#!/bin/bash

if [ "$1" = "stop" ]; then
    docker stop appledore
    docker rm appledore
    exit 0
fi

cargo build --target x86_64-unknown-linux-musl --release
docker build -t appledore .
docker run -d -p 6379:6379 --name appledore appledore
