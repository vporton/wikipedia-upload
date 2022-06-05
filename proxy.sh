#!/bin/sh

docker run --rm /root/proxy/target/release/proxy -p 8080 -u http://localhost:1633 \
    -A "Content-Encoding: br" -R "Accept-Ranges" -R "Content-Length" -R "Decompressed-Content-Length"
