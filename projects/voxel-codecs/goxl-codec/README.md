# goxl-codec

Reads and writes [Goxel](https://goxel.xyz/) `.gox` files.

`from_gox_file_bytes` parses bytes into a `goxl::GoxFile`; `to_gox_file_bytes` writes one back to an equivalent file. Reads are bounds-checked, so truncated or malformed input is rejected, not masked.

See `goxl` for the data types.

Not yet implemented; this crate is currently an empty placeholder.
