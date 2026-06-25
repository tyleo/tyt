# goxl-codec

Reads and writes [Goxel](https://goxel.xyz/) `.gox` files.

`from_gox_file_bytes` parses bytes into a `goxl::GoxlFile`; `to_gox_file_bytes` writes one back to an equivalent file. Reads are bounds-checked, so truncated or malformed input is rejected, not masked. The shared `BL16` voxel blocks and the `PREV` preview are decoded from their PNGs into voxel/pixel arrays and re-encoded on write; PNG is lossless for the pixels, so a file round-trips through a decode/encode.

`validate_gox_file` optionally checks a file for shape and cross-reference faults. It is not run on decode.

See `goxl` for the data types.
