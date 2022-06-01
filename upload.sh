#!/bin/bash

if $# != 2; then
  echo "Usage: $0 <ZIMURL> <KEEPALIVE-SECONDS>"
fi

zim_url="$1"
keepalive_seconds="$2"

TMPDIR=$(mktemp -d)
trap "test \"$TMPDIR\" != '' && rm -rf \"$TMPDIR\"" EXIT

wget -O $TMPDIR/input.zim "$zim_url"

docker build -t zim-tools -f Dockerfile.zim-tools .
zimtools=$(docker run -d --name zimdump zim-tools --mount "type=volume,src=$TMPDIR,dst=/tmp/workdir")
docker exec $zimtools /usr/local/bin/zimdump dump --dir=/tmp/workdir/out --redirect $TMPDIR/input.zim

price_in_bzz_per_byte_second=FIXME
total_size_in_bytes=$(du -sb $TMPDIR/out | awk '{print $1}')
cost_in_bzz=$(($price_in_bzz_per_byte * $total_size_in_bytes * $keepalive_seconds))
depth=20
amount=$(($priceInBZZ * (10**16) / (2**$depth))) # FIXME: silent overflow possible
batchId=$(curl -s -XPOST http://localhost:1635/stamps/$amount/$depth | python3 -c 'import json, sys; print(json.load(sys.stdin)["batchID"])')
