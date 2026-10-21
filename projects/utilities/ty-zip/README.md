# ty-zip

Frames a deflate stream as a single-member zip archive and reads it back. The
JSON document formats use it for their zipped forms, such as `.voxjz` and
`.sdfjz`.

- `wrap_deflated_member`: the archive holding one member under a name, given
  the member and its raw deflate stream. Identical input yields a
  byte-identical archive. A member past 4 GiB switches the archive to zip64.
- `deflated_member`: the deflate stream of an archive's first member.

The caller compresses and decompresses the member, so the crate carries no
compression library.
