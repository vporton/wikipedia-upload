#!/bin/sh

if test $# != 2; then
  echo "Usage: upload-from-zim-url.sh (-u|-f) <URL>"
  echo
  echo "It uploads URL or file."
  exit 1
fi

# Without multiplying by 10 upload stalls.
./tool.py -F static -H -M "$1" -S -B -m 400 $1 $2 upload -s $((10*30*24*3600)) -I index.html -E error.html
