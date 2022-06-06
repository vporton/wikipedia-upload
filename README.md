# Wikipedia & ZIM uploader

FIXME: It is nondeterministic because of files modification times.

## Installation

Prerequistes:
* sh (tested with Bash)
* tar
* Python 3
* Sed
* a working installation of Bee

Do the following (in any order):

Start a Bee node on localhost. Debug port must be enabled (unless for upload
you specify existing batch ID by `--batch-id`). You can use my `Dockerfile.bee`.
Don't forget to fund the Bee Swarm wallet with both xDai and BZZ.
Don't forget to connect `/root/.bee` to your volume directory, otherwise the
files (including the funded wallet will disappear).

If you choose to use my Bee container, run (with addition to correct volume):
```sh
docker build -t bee -f Dockerfile.bee .
docker run -d --name bee bee
```

Build Docker images by
```sh
./build.sh
```

## Command line

```
$ ./tool.py -h
usage: tool.py [-h] [-l LOG_LEVEL] [-F ADD_FILES] [-H | --enhance-files | --no-enhance-files] [-M TEXT] [-S | --search | --no-search] [-B | --brotli | --no-brotli] [-m MAX_SEARCH_RESULTS]
               (-f ZIM_FILE | -u ZIM_URL | -i DIR)
               {extract,upload} ...

Extract ZIM archive and/or upload files to Swarm

positional arguments:
  {extract,upload}      the operation to perform
    extract             extract from ZIM
    upload              upload to Swarm (after extraction if ZIM file specified)

options:
  -h, --help            show this help message and exit
  -l LOG_LEVEL, --log-level LOG_LEVEL
                        log level (DEBUG, INFO, WARNING, ERROR, CRITICAL or a number)
  -F ADD_FILES, --add-files ADD_FILES
                        files to add
  -H, --enhance-files, --no-enhance-files
                        add a comment to bottom
  -M TEXT, --enhance-files-more TEXT
                        add the specified text in bottom comment
  -S, --search, --no-search
                        create search index in search/
  -B, --brotli, --no-brotli
                        compress files with Brotli (inplace)
  -m MAX_SEARCH_RESULTS, --max-search-results MAX_SEARCH_RESULTS
                        maximum number of search results stored
  -f ZIM_FILE, --zim-file ZIM_FILE
                        ZIM file for extraction
  -u ZIM_URL, --zim-url ZIM_URL
                        ZIM URL to download and extract
  -i DIR, --input-dir DIR
                        input directory for upload to Swarm
$ ./tool.py extract -h
usage: tool.py extract [-h] [-o DIR]

options:
  -h, --help            show this help message and exit
  -o DIR, --output-dir DIR
                        output directory
$ ./tool.py upload -h
usage: tool.py upload [-h] (-s SECONDS | -b BATCH_ID) [-L UPLOADS_LOG] [-I INDEX_DOCUMENT] [-E ERROR_DOCUMENT]

options:
  -h, --help            show this help message and exit
  -s SECONDS, --keepalive-seconds SECONDS
                        keep swarm alive for at least about this
  -b BATCH_ID, --batch-id BATCH_ID
                        use batch ID to upload (creates one if not present in command line)
  -L UPLOADS_LOG, --uploads-log-file UPLOADS_LOG
                        uploads log file (default uploads.log, specify empty string for no uploads log)
  -I INDEX_DOCUMENT, --index-doc INDEX_DOCUMENT
                        index document name
  -E ERROR_DOCUMENT, --error-doc ERROR_DOCUMENT
                        error document name
```

In `extract` mode it does not upload to Swarm, but just creates a directory for testing.

Some "difficult to understand" options:

`-F` or `--add-files` allows to specify additional files to upload. By default `static/`
subdirectory (not including hidden files) of current directory is added.
To specify index and error files use `-I` and `-E`.

`-H`, `--enhance-files` appends to every HTML file in `A/` a comment with name and hash
of the ZIM file. If can add additional information to this comment by `--enhance-files-more`.

`-m`, `--max-search-results` See _Usage_.

TODO: modify mtimes to be the same as of .zim file.

## Usage

Open the uploaded hash (also stored in `uploads.log` in the current directory or
location specified by `-L` option) to view.

It has two buttons "Open article" and "Search". Search performs set intersection of
indexes for typed words (searching subwords is not supported). Each index contains all
pages with this exact word but no more pages that the value specified by `-m`, `--max-search-results`.

### View files

If files are not Brotli-compressed, just open the BZZ URL.

If the files are Brotli-compressed, you can view them only through a special proxy:
run it as
```
./proxy.sh
```
to appear on port 8080.

If you need SSL, chain Bee and two proxies like this:
```
Apache/Nginx <-- ./proxy.sh <-- Bee
```

## Technicals

It is a system of Python and sh scripts and two Docker containers:

`preparer` contains two commands developed by me:

- `indexer` that creates a word index
- `brotler` that compress every file except of symlinks in a directory recursively

`zim-tools` contains the third-party package [ZIM tools](https://github.com/openzim/zim-tools).

The actions sequence (some elements are excluded dependently on command line options):

- download ZIM
- extract ZIM by `zimdump` from ZIM tools Docker container
- remove the directory `X` containing indexes for dynamic sites
- add additional files (`index.html` and `error.html`)
- create a lighweight search index by `indexer` executable from `preparer` Docker container
- enhance files (add the source ZIM location and its SHA-256 sum and optional user data)
- compress files inplace by `brotler` executable from `preparer` Docker container
- calculate the amount of postmarks we need, using `/chainstate` _debug_ API of Bee
- create Bee's batch ID and tag
- upload the directory through Bee's tar interface
- the previous operation repeates until success (because batch ID is available only after a delay)
- add uploaded reference to `uploads.log` file
- wait till synced == total

## Bugs

It every time creates a new manifest, because files have different mtimes.

## Old ideas

I considered to modify `.tar` so that
symlinks would be uploaded with `Content-Type: text/x-redirect; charset=utf-8`.
Then to modify Bee (Create `/bzz/x-redirect` endpoint) to serve this content type as a redirect.
But it is not worth as Wikipedia HTML redirect files anyway fit into one block.
Also zim-tools `zimdump` has a [bug](https://github.com/openzim/zim-tools/issues/303).

At first, it looks like we should use direct upload (`Swarm-Deferred-Upload: false`),
but if we want to upload 1000 files in one block, we would need open 1000 connections.
So, instead just do 1000 `bzz-raw` uploads with a tag and wait till `synced`=`total`.
Alternatively, we can assign a tag to every `bzz-raw` upload, but it seems unnecessary.
(I suppose that uploading many chunks to local node is a reliable operation and the
need to reupload is seldom.)

Check code: https://github.com/ethersphere/swarm-cli/blob/ec496a220bffd024f9e3896abd96a834032b7200/src/command/upload.ts#L272

Debug port must be open unless you specify existing `--batch-id`.
