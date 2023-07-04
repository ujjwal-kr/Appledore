#!/bin/bash

if [ "$1" = "stop" ]; then
    docker stop appledore
    docker rm appledore
    exit 0
fi

docker build -t appledore .
docker run -d -p 6379:6379 --name appledore appledore
