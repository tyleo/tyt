# mvox-codec

Reads and writes [MagicaVoxel](https://ephtracy.github.io/) `.vox` files.

`from_vox_file_bytes` parses bytes into an `mvox::MVoxFile`; `to_vox_file_bytes` writes one back to an equivalent file. Reads are bounds-checked, so truncated or malformed input is rejected, not masked.

`validate_vox_file` optionally checks a file for cross-reference and bounds faults. It is not run on decode.

See `mvox` for the data types.
