Algorithm on trigger:

1. Download the ZIM.

1. Extract ("zimdump dump") it by Docker image with zim-tools.

1. Brotli all files.

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

TODO: Google Custom Search?

TODO: https://github.com/ethersphere/swarm-cli/blob/ec496a220bffd024f9e3896abd96a834032b7200/src/command/upload.ts#L272

Python logs

Tell about sudo and password.

TODO: Don't copy {index,error.html} by default.

Debug port must be open unless you specify existing `--batch-id`.
