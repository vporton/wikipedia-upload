#!/bin/env python3

from shutil import copyfile
from os.path import abspath
import sys, os, subprocess, requests
from tempfile import TemporaryDirectory
import argparse

parser = argparse.ArgumentParser(description="Upload files to Swarm")
parser.add_argument("-f", "--zim-file", dest="zim_file", help="ZIM file")
parser.add_argument("-u", "--zim-url", dest="zim_url", help="ZIM URL to download")
parser.add_argument("-s", "--keepalive-seconds", dest="keepalive_seconds",
                    help="keep swarm alive for at least about this", metavar="int")
parser.add_argument("-b", "--batch-id", dest="batch_id", help="use batchID to upload")
parser.add_argument("-B", "--no-brotli", dest="no_brotli", help="don't compress files", action=argparse.BooleanOptionalAction)
parser.add_argument("-d", "--output-dir", dest="output_dir", help="output to directory instead of uploading", metavar="str")

args = parser.parse_args()

no_upload = args.output_dir is not None

if args.keepalive_seconds is not None and args.batch_id is not None:
    sys.stderr.write("Can't specify both --keepalive-seconds and --batch-id")
    os.exit(1)

if not no_upload and args.keepalive_seconds is None and args.batch_id is None:
    sys.stderr.write("Need to specify --keepalive-seconds or --batch-id")
    os.exit(1)

if args.zim_file is not None and args.zim_url is not None:
    sys.stderr.write("Can't specify both --zim-file and --zim-url")
    os.exit(1)

if args.zim_file is None and args.zim_url is None:
    sys.stderr.write("Need to specify --zim-file or --zim-url")
    os.exit(1)

zim_url = args.zim_url
keepalive_seconds = args.keepalive_seconds
zim_file = args.zim_file

def prepare_files(output_dir):
    with TemporaryDirectory() as input_dir:
        if zim_url:
            # TODO: Don't place input.zim in current directory.
            os.system(f"wget -O {input_dir}/input.zim \"{zim_url}\"")
        else:
            copyfile(args.zim_file, f"{input_dir}/input.zim")

        os.mkdir(output_dir)

        os.system(f"docker build -t zim-tools -f Dockerfile.zim-tools .")
        print("Starting zimdump extraction...")
        print(f"OUTPUT={output_dir}")
        os.system(
            f"docker run --name zimdump -v \"{abspath(input_dir)}:/in\" -v \"{abspath(output_dir)}:/out\" zim-tools " \
                f"/usr/local/bin/zimdump dump --dir=/out /in/input.zim")
        # TODO: Fix https://github.com/openzim/zim-tools/issues/303 and make Bee understand redirects, then add `--redirect` here:
        # os.system(f"docker exec zimtools /usr/local/bin/zimdump dump --dir=/tmp/workdir/out {output_dir}/input.zim")
        os.system(f"docker rm -f zimdump")
        os.system(f"sudo chown -R `id -u`:`id -g` {output_dir}")  # hack
        os.system(f"rm -rf {output_dir}/X")  # Remove useless search indexes.

        # TODO: Files for other sites (not Wikipedia).
        os.system(f"cp index.html error.html {output_dir}")
        # os.system(f"cd {output_dir} && wget https://cdn.jsdelivr.net/npm/bootstrap@5.0.2/dist/css/bootstrap.min.css")

        if not args.no_brotli:
            os.system(f"docker build -t brotler -f Dockerfile.brotler .")
            print("Starting brotler...")
            subprocess.check_output(
                f"docker run --name brotler brotler -v \"{abspath(output_dir)}:/volume\""
                    f" /root/brotler/target/release/brotler /volume",
                shell=True)
            os.system(f"docker rm -f brotler")
            os.system(f"sudo chown -R `id -u`:`id -g` {output_dir}")  # hack

def full_upload():
    with TemporaryDirectory() as tmpdir:
        prepare_files(f"{tmpdir}/out")

        if args.batch_id is None:
            price_in_bzz_per_byte_second = FIXME
            total_size_in_bytes = int( subprocess.check_output(f"du -sb {tmpdir}/out | awk '{{print \$1}}'") )
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

if no_upload:
    prepare_files(args.output_dir)
else:
    full_upload()
