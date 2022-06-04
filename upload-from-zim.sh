#!/bin/sh

if test $# != 2; then
  echo "Usage: upload-from-zim-url.sh (-u|-f) <URL>"
  echo
  echo "It uploads URL or file."
  exit 1
fi

./tool.py -F static -H -M "$1" -S -B -m 400 $1 $2 upload -s $((30*24*3600)) -I index.html -I error.html
