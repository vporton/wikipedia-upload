#!/bin/env python3

import os
import re
import sys
import requests


if len(sys.argv) != 4:
    print("A specialized script to upload Wikipedia to Swarm\n")
    print("Usage ./cron.py <SOURCE-DIR> <STATE-DIR> <SUBSTR>")
    print("\nHere <SOURCE-DIR> is this directory, <STATE-DIR> stores the state and log, <SUBSTR> is like `en_all_maxi`.")
    sys.exit(1)
source_dir, state_dir, sstr = sys.argv[1], sys.argv[2], sys.argv[3]

directory_url = "https://dumps.wikimedia.org/other/kiwix/zim/wikipedia/"

res = requests.get(directory_url)
matches = re.findall('href="(.*?)"', res.content.decode('utf-8'))
matches = [match for match in matches if re.match(f'^wikipedia_{sstr}_([0-9]{{4}})-([0-9]{{2}}).zim$', match)]
url = directory_url + max(matches)

try:
    with open(f"{state_dir}/last_upload.txt", "r") as f:
        old_url = f.read()
        if old_url == url:
            print(f"{url} already uploaded, exiting")
except FileNotFoundError:
    pass

class CommandError(Exception):
    """Error running shell command"""

    def __init__(self, command):
        super().__init__(f"Error running shell command: {command}")

def run_command(command):
    if os.system(command) != 0:
        raise CommandError(command)

# Without multiplying by 10 upload stalls.
run_command(f"{source_dir}/tool.py -F {source_dir}/static -H -M '' -S -B -m 400 -u {url} upload " \
    f"-s {10*30*24*3600} -I {source_dir}/index.html -E {source_dir}/error.html -L {state_dir}/uploads.log")

with open(f"{state_dir}/last_upload.txt", "w") as f:
    f.write(url)
