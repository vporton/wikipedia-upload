#!/bin/sh

docker run --net=host --rm proxy /root/proxy/target/release/proxy --port 8080 http://localhost:1633
