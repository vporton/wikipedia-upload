#!/bin/env python3

from cProfile import run
import json
import logging
import re
from shlex import quote
from shutil import copyfile
from os.path import abspath
import sys, os, subprocess, requests
from time import sleep
from tempfile import TemporaryDirectory
import argparse

parser = argparse.ArgumentParser(description="Extract ZIM archive and/or upload files to Swarm")
parser.add_argument("-l", "--log-level", dest="log_level", help="log level (DEBUG, INFO, WARNING, ERROR, CRITICAL or a number)")
parser.add_argument("-F", "--add-files", dest="add_files", help="files to add")
parser.add_argument("-H", "--enhance-files", dest="enhance_files", help="add a comment to bottom", action=argparse.BooleanOptionalAction)
parser.add_argument("-M", "--enhance-files-more", dest="enhance_files_more", help="add the specified text in bottom comment", metavar="TEXT", nargs='?')
parser.add_argument("-S", "--search", dest="search_index", help="create search index in search/", action=argparse.BooleanOptionalAction)
parser.add_argument("-B", "--brotli", dest="brotli", help="compress files with Brotli (inplace)", action=argparse.BooleanOptionalAction)
parser.add_argument("-m", "--max-search-results", dest="max_search_results", help="maximum number of search results stored",
                    default=500)
subparsers = parser.add_subparsers(dest='command', help="the operation to perform")
group = parser.add_mutually_exclusive_group(required=True)
group.add_argument("-f", "--zim-file", dest="zim_file", help="ZIM file for extraction", nargs='?')
group.add_argument("-u", "--zim-url", dest="zim_url", help="ZIM URL to download and extract", nargs='?')
group.add_argument("-i", "--input-dir", dest="input_dir", help="input directory for upload to Swarm", metavar="DIR")
parser_extract = subparsers.add_parser('extract', help="extract from ZIM")
parser_extract.add_argument("-o", "--output-dir", dest="output_dir", help="output directory", metavar="DIR")
parser_upload = subparsers.add_parser('upload', help="upload to Swarm (after extraction if ZIM file specified)")
group = parser_upload.add_mutually_exclusive_group(required=True)
group.add_argument("-s", "--keepalive-seconds", dest="keepalive_seconds",
                   help="keep swarm alive for at least about this", metavar="SECONDS")
group.add_argument("-b", "--batch-id", dest="batch_id", help="use batch ID to upload (creates one if not present in command line)")
parser_upload.add_argument("-L", "--uploads-log-file", dest="uploads_log",
                           help="uploads log file (default uploads.log, specify empty string for no uploads log)", default="uploads.log")
parser_upload.add_argument("-I", "--index-doc", dest="index_document", help="index document name")
parser_upload.add_argument("-E", "--error-doc", dest="error_document", help="error document name")
parser_upload.add_argument("-D", "--feed-topic", dest="feed_topic", help="Swarm feed topic (update it if specified)")

args = parser.parse_args()

if args.command == 'extract' and args.input_dir is not None:
    sys.stderr.write("Incompatible options: cannot extract from directory.")
    os.exit(1)

if args.enhance_files is not None and args.input_dir is not None:
    sys.stderr.write("Incompatible options: enhance a directory.")
    os.exit(1)

logging.basicConfig(level=args.log_level or logging.INFO)
logger = logging.getLogger("tool.py")

class CommandError(Exception):
    """Error running shell command"""

    def __init__(self, command):
        super().__init__(f"Error running shell command: {command}")

def run_command(command):
    if os.system(command) != 0:
        raise CommandError(command)

def extract_zim(output_dir):
    with TemporaryDirectory() as input_dir:
        if args.zim_url:
            # TODO: Don't place input.zim in current directory.
            run_command(f"wget --tries=100 -O {input_dir}/input.zim \"{args.zim_url}\"")
        else:
            copyfile(args.zim_file, f"{input_dir}/input.zim")

        try:
            os.mkdir(output_dir)
        except FileExistsError:
            pass

        logger.info(f"Starting zimdump extraction to {output_dir}...")
        run_command(
            f"docker run --rm -u{os.getuid()} -v \"{abspath(input_dir)}:/in\" -v \"{abspath(output_dir)}:/out\" zim-tools " \
                f"/usr/local/bin/zimdump dump --dir=/out /in/input.zim")
        # TODO: Fix https://github.com/openzim/zim-tools/issues/303 and make Bee understand redirects, then add `--redirect` here:
        run_command(f"rm -rf {output_dir}/X")  # Remove useless search indexes.

        logger.info("Correcting files with /...")
        needle = r'<meta http-equiv="refresh" content="0;url=../'
        repl   = r'<meta http-equiv="refresh" content="0;url='
        root = f"{output_dir}/_exceptions"
        RE = re.compile(r".*?A%2f([^/]*)$")
        for (_dirpath, _dirnames, filenames) in os.walk(root):
            for name in filenames:
                source = os.path.join(root, name)
                dest = f"{output_dir}/A/" + re.sub(RE, r'\1', source).replace('/', '%2f')  # Be careful, it is used in rm -rf
                logger.info(f"{source} -> {dest}")
                with open(source) as source_file:
                    text = source_file.read().replace(needle, repl)
                    try:
                        os.system(f"rm -rf {dest}")
                    except NameError:
                        pass
                    with open(dest, 'w') as dest_file:
                        dest_file.write(text)
                os.unlink(source)

        if args.add_files:
            logger.info("Adding additional files...")
            run_command(f"cp -r {args.add_files}/* {output_dir}")
        
        if args.search_index:
            logger.info("Creating search index...")
            run_command(f"docker run --rm -e RUST_LOG={args.log_level} -u{os.getuid()} -v \"{abspath(output_dir)}:/volume\" preparer" \
                f" /root/preparer/target/release/indexer -m {args.max_search_results} /volume/A /volume/search")

        if args.enhance_files:
            source = args.zim_url if args.zim_url else args.zim_file
            run_command(f"sum=`sha256sum {input_dir}/input.zim | awk '{{print $1}}'`; find {output_dir}/A -type f | "\
                f"while read f; do {{ echo; echo \"<!--\"; echo {source} SHA256=$sum; echo {args.enhance_files_more}; echo \"-->\"; }} >> \"$f\"; done")

        if args.brotli:
            logger.info("Compressing files (inplace)...")
            run_command(f"docker run --rm -e RUST_LOG={args.log_level} -u{os.getuid()} -v \"{abspath(output_dir)}:/volume\" preparer" \
                f" /root/preparer/target/release/brotler /volume")

        # Resetting mtimes makes it deterministic.
        logger.info("Resetting files mtime...")
        run_command(f"docker run --rm -e RUST_LOG={args.log_level} -u{os.getuid()} -v \"{abspath(input_dir)}:/in\" -v \"{abspath(output_dir)}:/out\" preparer " \
                f"/root/preparer/target/release/copy_mtime /in/input.zim /out")


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
        depth = 28  # terabyte data
        amount = int(cost_in_bzz / (2**depth)) + 1
        url = f"http://localhost:1635/stamps/{amount}/{depth}"
        print(url)
        res = requests.post(url)
        try:
            batch_id = res.json()["batchID"]
        except KeyError:
            print(res.json()["message"])
            sys.exit(1)

    logger.info("Creating an upload tag...")
    res = requests.post("http://localhost:1633/tags")
    tag = res.json()["uid"]
    logger.info(f"Upload tag: {tag}")

    logger.info("Starting TAR upload...")
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
        try:
            res = requests.post("http://localhost:1633/bzz", data=tar.stdout, headers=headers)
        except requests.exceptions.ConnectionError:  # tar disconnected  # TODO: Differentiate different errors.
            logger.info('tar disconnected - repeating')
        else:
            if 200 <= res.status_code < 300:
                uploaded_reference = res.json()['reference']
                break
            else:
                logger.info(res.json()["message"])
        sleep(1.0)

    while True:
        res = requests.get(f"http://localhost:1633/tags/{tag}")
        total = res.json()['total']
        processed = res.json()['processed']
        synced = res.json()['synced']
        logger.info(f"total={total} processed={processed} synced={synced}")
        if synced >= total:
            break
        sleep(1.0)

    file_identificator = (args.zim_file if args.zim_file else args.zim_url) \
        if args.zim_file or args.zim_url else args.input_dir
    if args.feed_topic:
        log_json = {
            'file': file_identificator,
            'reference': uploaded_reference,
            'batchID': batch_id,
            'tag': tag
        }
        log_json_serialized = quote(json.dumps(log_json))

        url = f"http://localhost:1635/stamps/1/17"
        print(f"stamp for the feed: {url}")
        res = requests.post(url)
        try:
            batch_id2 = res.json()["batchID"]
        except KeyError:
            print(res.json()["message"])
            sys.exit(1)

        print("Signing the feed:")
        run_command(f"docker run --rm --network=host -u{os.getuid()} sign-feed " \
            f"sh -c node /root/signFeed/signfeed-cli.js {quote(args.feed_topic)} {quote(log_json_serialized)}")
    log_line = f"{file_identificator} reference={uploaded_reference} batchID={batch_id2} tag={tag} feed_topic={quote(args.feed_topic)}\n"
    sys.stdout.write(log_line)
    if args.uploads_log:
        with open(args.uploads_log, 'a') as f:
            f.write(log_line)

if args.command == 'extract':
    extract_zim(args.output_dir)
elif args.input_dir:
    upload(args.input_dir)
else:
    extract_and_upload()
