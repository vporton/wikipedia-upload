#!/bin/env python3

import json, sys, os, subprocess, requests
from tempfile import TemporaryDirectory
from argparse import ArgumentParser

parser = ArgumentParser(description="Upload files to Swarm")
parser.add_argument("-f", "--zim-file", dest="zim_file", help="ZIM file")
parser.add_argument("-u", "--zim-url", dest="zim_file", help="ZIM URL to download")
parser.add_argument("-s", "--keepalive-seconds", dest="keepalive_seconds",
                    help="keep swarm alive for at least about this", metavar="int")
parser.add_argument("-b", "--batch-id", dest="batch_id", help="use batchID to upload")
parser.add_argument("-S", "--dont-scan", dest="dont_scan", help="use batchID to upload", metavar="bool")

args = parser.parse_args()

if args.keepalive_seconds is not None and args.batch_id is not None:
    sys.stderr.write("Can't specify both --keepalive-seconds and --batch-id")
    os.exit(1)

if args.keepalive_seconds is None and args.batch_id is None:
    sys.stderr.write("Need to specify --keepalive-seconds or --batch-id")
    os.exit(1)

if args.zim_file is not None and args.zim_url is not None:
    sys.stderr.write("Can't specify both --zim-file and --zim-url")
    os.exit(1)

if args.zim_file is None and args.zim_url is None:
    sys.stderr.write("Need to specify --zim-file or --zim-url")
    os.exit(1)

zim_url = args.zimurl
keepalive_seconds = args.keepalive_seconds
zim_file = args.zim_file

with TemporaryDirectory as tmpdir:
    if zim_url:
        os.system(f"wget -O {tmpdir}/input.zim \"{zim_url}\"")
        zim_file = f"{tmpdir}/input.zim"

    os.system(f"docker build -t zim-tools -f Dockerfile.zim-tools .")
    zimtools = subprocess.check_output(f"docker run -d --name zimdump zim-tools --mount \"type=volume,src={tmpdir},dst=/tmp/workdir\"")
    print("Starting zimdump extraction...")
    os.system(f"docker exec zimtools /usr/local/bin/zimdump dump --dir=/tmp/workdir/out --redirect {tmpdir}/input.zim")
    os.system(f"docker rm {zimtools}")
    os.system(f"rm -rf {tmpdir}/out/X")  # Remove useless search indexes.

    os.system(f"docker build -t brotler -f Dockerfile.brotler .")
    brotler = subprocess.check_output(f"docker run -d --name brotler brotler --mount \"type=volume,src={tmpdir},dst=/tmp/workdir\"")
    print("Starting brotler...")
    os.system(f"docker exec brotler /root/brotler/target/release/brotler /tmp/workdir/out")
    os.system(f"docker rm {brotler}")

    if args.batch_id is None:
        price_in_bzz_per_byte_second = FIXME
        total_size_in_bytes = int( subprocess.check_output(f"du -sb {tmpdir}/out | awk '{print \$1}'") )
        cost_in_bzz = price_in_bzz_per_byte_second * total_size_in_bytes * keepalive_seconds
        depth = 20
        amount = cost_in_bzz * (10**16) / (2**depth)
        res = requests.post(f"http://localhost:1635/stamps/{amount}/{depth}")
        batch_id = res.json()["batchID"]  # FIXME: batch ID may be usable not immediately

    res = requests.post("http://localhost:1633/tags")
    tag = res.json()["uid"]

    print("Starting TAR upload...")
    process = subprocess.Popen(f"tar -C {tmpdir}/out -cf .", stdout=subprocess.PIPE)
    res = requests.post("http://localhost:1633/bzz", data=tar, headers={
        "Content-Type": "application/x-tar",
        "Swarm-Index-Document": "index.html",
        "Swarm-Error-Document": "error.html",
        "Swarm-Collection": "true",
        "Swarm-Postage-Batch-Id": batch_id,
        "Swarm-Tag": tag,
    })

    while True:
        res = requests.get(f"http://localhost:1633/tags/{tag}")
        total = res.json()['total']
        processed = res.json()['processed']
        synced = res.json()['synced']
        print(f"fotal={total} processed={processed} synced={synced}")
        if synced == total:
            break
