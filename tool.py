#!/bin/env python3

from shutil import copyfile
from os.path import abspath
import sys, os, subprocess, requests
from tempfile import TemporaryDirectory
import argparse

parser = argparse.ArgumentParser(description="Extract ZIM archive and/or upload files to Swarm")
subparsers = parser.add_subparsers(dest='command', help="the operation to perform")
group = parser.add_mutually_exclusive_group(required=True)
group.add_argument("-f", "--zim-file", dest="zim_file", help="ZIM file for extraction")
group.add_argument("-u", "--zim-url", dest="zim_url", help="ZIM URL to download")
group.add_argument("-i", "--input-dir", dest="input_dir", help="input directory for upload to Swarm", metavar="DIR")
parser.add_argument("-B", "--brotli", dest="brotli", help="compress files with Brotli (inplace)", action=argparse.BooleanOptionalAction)
parser_extract = subparsers.add_parser('extract', help="extract from ZIM")
parser_extract.add_argument("-o", "--output-dir", dest="output_dir", help="output directory", metavar="DIR")
parser_upload = subparsers.add_parser('upload', help="upload to Swarm (after extraction if ZIM file specified)")
group = parser_upload.add_mutually_exclusive_group(required=True)
group.add_argument("-s", "--keepalive-seconds", dest="keepalive_seconds",
                   help="keep swarm alive for at least about this", metavar="SECONDS")
group.add_argument("-b", "--batch-id", dest="batch_id", help="use batch ID to upload")

args = parser.parse_args()

if args.command == 'extract' and args.input_dir is not None:
    sys.stderr.write("Incompatible options: cannot extract from directory.")
    os.exit(1)

no_upload = args.output_dir is not None

def extract_zim(output_dir):
    with TemporaryDirectory() as input_dir:
        if args.zim_url:
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

def extract_and_upload():
    with TemporaryDirectory() as tmpdir:
        extract_zim(tmpdir)
        upload(tmpdir)

def upload(directory):
    if args.brotli:
        os.system(f"docker build -t brotler -f Dockerfile.brotler .")
        print("Starting brotler...")
        os.system(f"docker run --name brotler -v \"{abspath(output_dir)}:/volume\" brotler" \
            f" /root/brotler/target/release/brotler /volume")
        os.system(f"docker rm -f brotler")
        os.system(f"sudo chown -R `id -u`:`id -g` {output_dir}")  # hack

    if args.batch_id is None:
        price_in_bzz_per_byte_second = FIXME
        total_size_in_bytes = int( subprocess.check_output(f"du -sb {directory} | awk '{{print \$1}}'") )
        cost_in_bzz = price_in_bzz_per_byte_second * total_size_in_bytes * args.keepalive_seconds
        depth = 20
        amount = cost_in_bzz * (10**16) / (2**depth)
        res = requests.post(f"http://localhost:1635/stamps/{amount}/{depth}")
        batch_id = res.json()["batchID"]  # FIXME: batch ID may be usable not immediately

    res = requests.post("http://localhost:1633/tags")
    tag = res.json()["uid"]

    print("Starting TAR upload...")
    process = subprocess.Popen(f"tar -C {directory} -cf .", stdout=subprocess.PIPE)
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

if args.command == 'extract':
    extract_zim(args.output_dir)
elif args.input_dir:
    upload(args.input_dir)
else:
    extract_and_upload()
