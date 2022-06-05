#!/bin/sh

docker run --rm /root/proxy/target/release/proxy -A "Content-Encoding: br" -R Accept-Ranges -R Content-Length -R Decompressed-Content-Length