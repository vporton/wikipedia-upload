#!/bin/sh

docker run --rm proxy /root/proxy/target/release/proxy --port 8080 http://localhost:1633 \
    -A "Content-Encoding: br" -R "Accept-Ranges" -R "Content-Length" -R "Decompressed-Content-Length"