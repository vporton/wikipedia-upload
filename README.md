Algorithm on trigger:

1. Download the ZIM.

1. Extract ("zimdump dump") it by Docker image with zim-tools.

1. Start Docker with Bee.

1. Modify `.tar` upload for:
   Symlinks will be uploaded with `Content-Type: text/x-redirect; charset=utf-8`.

1. Output to terminal `synced`!=`total` until they equal.

1. Create `/bzz/x-redirect` endpoint supporting `text/x-redirect`.

1. Add `Content-Encoding: br` by a Nginx container.

At first, it looks like we should use direct upload (`Swarm-Deferred-Upload: false`),
but if we want to upload 1000 files in one block, we would need open 1000 connections.
So, instead just do 1000 `bzz-raw` uploads with a tag and wait till `synced`=`total`.
Alternatively, we can assign a tag to every `bzz-raw` upload, but it seems unnecessary.
(I suppose that uploading many chunks to local node is a reliable operation and the
need to reupload is seldom.)

============================

**Old** algorithm on trigger:

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
