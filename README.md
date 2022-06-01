Algorithm on trigger:

1. Download the ZIM.

1. Extract ("zimdump dump") it by Docker image with zim-tools.

1. Start Docker with Bee.

1. Create a DB mapping number of file into its bzzhash (calculated by Go) and name.
   Store also for each file whether it was uploaded and minimal number of not yet uploaded file.

1. Upload to Swarm one-by-one file encoding by brotli all not yet uploaded files (bzz-raw, no manifest).
   Calculate manifest accordingly https://ethereum.stackexchange.com/questions/83977/generate-directory-hash-without-upload-using-swarm-api
   and upload it.
   Symlinks will be uploaded with `Content-Type: text/x-redirect; charset=utf-8`.

1. Create `/bzz/x-redirect` endpoint supporting `text/x-redirect`.

1. Add `Content-Encoding: br` by a Nginx container.
