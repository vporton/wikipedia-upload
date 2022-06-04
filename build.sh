#!/bin/sh

set -e

docker build -t zim-tools -f Dockerfile.zim-tools .
docker build -t preparer -f Dockerfile.preparer .
