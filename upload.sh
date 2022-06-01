#!/bin/bash

if $# != 1; then
  echo "Usage: $0 <ZIMURL>"
fi

TMPDIR=$(mktemp -d)
trap "test \"$TMPDIR\" != '' && rm -rf \"$TMPDIR\"" EXIT

wget -O $TMPDIR/input.zim "$1"

docker build -t zim-tools -f Dockerfile.zim-tools .
zimtools=$(docker run -d --name zimdump zim-tools --mount "type=volume,src=$TMPDIR,dst=/tmp/workdir")
docker exec $zimtools /usr/local/bin/zimdump dump --dir=/tmp/workdir/out --redirect $TMPDIR/input.zim
