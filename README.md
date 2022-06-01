Algorithm on trigger:

1. Download the ZIM.

1. Extract ("zimdump dump") it by Docker image with zim-tools.

1. Compress files with Gzip (without adding `.gz` extension).

1. Start Docker with Bee.

1. Create a DB mapping number and bzzhash (calculated by Go) of file into its name.
   Store the number of next file to be uploaded.

1. Upload to Swarm one-by-one file with all not yet uploaded files (bzz-raw, no manifest).
   Calculate manifest accordingly https://ethereum.stackexchange.com/questions/83977/generate-directory-hash-without-upload-using-swarm-api
   and upload it.
   Symlinks will be uploaded with `Content-Type: symlink; charset=utf-8`.

1. Create `/bzz/links/brotli` endpoint.
