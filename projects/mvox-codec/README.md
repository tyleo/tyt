# mvox-codec

Reads and writes MagicaVoxel `.vox` files.

`from_vox_file_bytes` parses the bytes of a `.vox` file into an `mvox::MVoxFile`, and `to_vox_file_bytes` serializes one back, losslessly: every modeled chunk round-trips every field, and a chunk this crate does not model is preserved verbatim. Parsing is bounds-checked throughout, so truncated or malformed input is rejected with an error rather than masked. See the `mvox` crate for the data types and the format references.
