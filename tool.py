#!/bin/env python3

import logging
from shutil import copyfile
from os.path import abspath
import sys, os, subprocess, requests
from time import sleep
from tempfile import TemporaryDirectory
import argparse

parser = argparse.ArgumentParser(description="Extract ZIM archive and/or upload files to Swarm")
parser.add_argument("-l", "--log-level", dest="log_level", help="log level (DEBUG, INFO, WARNING, ERROR, CRITICAL or a number)")
parser.add_argument("-S", "--search", dest="search_index", help="create search index in search/", action=argparse.BooleanOptionalAction)
parser.add_argument("-B", "--brotli", dest="brotli", help="compress files with Brotli (inplace)", action=argparse.BooleanOptionalAction)
parser.add_argument("-m", "--max-search-results", dest="max_search_results", help="maximum number of search results stored",
                    default=500)
subparsers = parser.add_subparsers(dest='command', help="the operation to perform")
group = parser.add_mutually_exclusive_group(required=True)
group.add_argument("-f", "--zim-file", dest="zim_file", help="ZIM file for extraction")
group.add_argument("-u", "--zim-url", dest="zim_url", help="ZIM URL to download")
group.add_argument("-i", "--input-dir", dest="input_dir", help="input directory for upload to Swarm", metavar="DIR")
parser_extract = subparsers.add_parser('extract', help="extract from ZIM")
parser_extract.add_argument("-o", "--output-dir", dest="output_dir", help="output directory", metavar="DIR")
parser_upload = subparsers.add_parser('upload', help="upload to Swarm (after extraction if ZIM file specified)")
group = parser_upload.add_mutually_exclusive_group(required=True)
group.add_argument("-s", "--keepalive-seconds", dest="keepalive_seconds",
                   help="keep swarm alive for at least about this", metavar="SECONDS")
group.add_argument("-b", "--batch-id", dest="batch_id", help="use batch ID to upload")
group.add_argument("-I", "--index-doc", dest="index_document", help="index document name")
group.add_argument("-E", "--error-doc", dest="error_document", help="error document name")

args = parser.parse_args()

if args.command == 'extract' and args.input_dir is not None:
    sys.stderr.write("Incompatible options: cannot extract from directory.")
    os.exit(1)

logging.basicConfig(level=args.log_level or logging.INFO)

def extract_zim(output_dir):
    with TemporaryDirectory() as input_dir:
        if args.zim_url:
            # TODO: Don't place input.zim in current directory.
            os.system(f"wget -O {input_dir}/input.zim \"{zim_url}\"")
        else:
            copyfile(args.zim_file, f"{input_dir}/input.zim")

        os.mkdir(output_dir)

        os.system(f"docker build -t zim-tools -f Dockerfile.zim-tools .")
        logging.info(f"Starting zimdump extraction to {output_dir}...")
        os.system(
            f"docker run -u{os.getuid()} --name zimdump -v \"{abspath(input_dir)}:/in\" -v \"{abspath(output_dir)}:/out\" zim-tools " \
                f"/usr/local/bin/zimdump dump --dir=/out /in/input.zim")
        # TODO: Fix https://github.com/openzim/zim-tools/issues/303 and make Bee understand redirects, then add `--redirect` here:
        # os.system(f"docker exec zimtools /usr/local/bin/zimdump dump --dir=/tmp/workdir/out {output_dir}/input.zim")
        os.system(f"docker rm -f zimdump")
        os.system(f"rm -rf {output_dir}/X")  # Remove useless search indexes.

        # TODO: Files for other sites (not Wikipedia).
        os.system(f"cp index.html error.html {output_dir}")
        # os.system(f"cd {output_dir} && wget https://cdn.jsdelivr.net/npm/bootstrap@5.0.2/dist/css/bootstrap.min.css")

        os.system(f"docker build -t preparer -f Dockerfile.preparer .")
        if args.search_index:
            logging.info("Creating search index...")
            os.system(f"docker run -u{os.getuid()} --name preparer -v \"{abspath(output_dir)}:/volume\" preparer" \
                f" /root/preparer/target/release/indexer -m {args.max_search_results} /volume/A /volume/search")
        if args.brotli:
            os.system(f"docker run -u{os.getuid()} --name preparer -v \"{abspath(output_dir)}:/volume\" preparer" \
                f" /root/preparer/target/release/brotler /volume")
        os.system(f"docker rm -f preparer")

def extract_and_upload():
    with TemporaryDirectory() as tmpdir:
        extract_zim(tmpdir)
        upload(tmpdir)

def upload(directory):
    if args.batch_id is None:
        res = requests.get("http://localhost:1635/chainstate")
        price_in_bzz_per_block_second = int(res.json()["currentPrice"])
        total_size_in_blocks = int( subprocess.check_output(f"du -s -B 4096 {directory} | awk '{{print $1}}'", shell=True) )
        cost_in_bzz = total_size_in_blocks * price_in_bzz_per_block_second * int(args.keepalive_seconds) / 5
        depth = 20
        amount = int(cost_in_bzz / (2**depth)) + 1
        url = f"http://localhost:1635/stamps/{amount}/{depth}"
        res = requests.post(url)
        batch_id = res.json()["batchID"]

    logging.info("Creating an upload tag...")
    res = requests.post("http://localhost:1633/tags")
    tag = res.json()["uid"]
    logging.info(f"Upload tag: {tag}")

    logging.info("Starting TAR upload...")
    while True:  # loop until batch ID is available
        tar = subprocess.Popen(f"tar -C {directory} -cf - .", shell=True, stdout=subprocess.PIPE)
        headers = {
            "Content-Type": "application/x-tar",
            "Swarm-Collection": "true",
            "Swarm-Postage-Batch-Id": batch_id,
            "Swarm-Tag": str(tag),
        }
        if args.index_document is not None:
            headers["Swarm-Index-Document"] = args.index_document
        if args.error_document is not None:
            headers["Swarm-Error-Document"] = args.error_document
        res = requests.post("http://localhost:1633/bzz", data=tar.stdout, headers=headers)
        if res.status_code != 400:
            with open("uploads.log", 'a') as uploads_log:
                file_identificator = (args.zim_file if args.zim_file else args.zim_url) \
                    if args.zim_file or args.zim_url else args.input_dir
                uploaded_reference = res.json()['reference']
                log_line = f"{file_identificator} {uploaded_reference}\n"
                uploads_log.write(log_line)
                sys.stdout.write(log_line)
            break
        else:
            logging.debug(res.json()["message"])
        sleep(1.0)

    while True:
        res = requests.get(f"http://localhost:1633/tags/{tag}")
        total = res.json()['total']
        processed = res.json()['processed']
        synced = res.json()['synced']
        logging.info(f"fotal={total} processed={processed} synced={synced}")
        if synced == total:
            break

if args.command == 'extract':
    extract_zim(args.output_dir)
elif args.input_dir:
    upload(args.input_dir)
else:
    extract_and_upload()
