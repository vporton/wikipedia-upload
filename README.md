Algorithm on trigger:

1. Download the ZIM.

1. Extract ("zimdump dump") it by Docker image with zim-tools.

1. Compress files with Gzip (without adding `.gz` extension).

1. Start Docker with Bee.

1. Upload to Swarm created on-the-fly `.tar` with all files.
   (Does .tar uploading need enhancment to be memory-efficient?)
   If fails, repeat upload.
   Set `Content-Encoding:`.
   Consider uncompressed `.zip` instead of `.tar`.

1. Ensure that Bee serves correctly with `Content-Encoding:`.