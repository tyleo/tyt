# qbcl-codec

Reads and writes the [Qubicle](https://getqubicle.com/) voxel editor's binary file formats: Qubicle Binary (`.qb`), Qubicle Binary Tree (`.qbt`), and Qubicle Construction Library (`.qbcl`). Each format lives in its own module behind a Cargo feature, all enabled by default: `qb`, `qbt`, and `qbcl`.

- `qb::from_qb_file_bytes` parses bytes into a `qbcl::qb::QbFile`; `qb::to_qb_file_bytes` writes one back. Run-length-encoded and raw voxel data both decode into the same dense grid.
- `qbt::from_qbt_file_bytes` parses bytes into a `qbcl::qbt::QbtFile`; `qbt::to_qbt_file_bytes` writes one back. Matrix voxel data is zlib-compressed.
- `qbcl::from_qbcl_file_bytes` parses bytes into a `qbcl::qbcl::QbclFile`; `qbcl::to_qbcl_file_bytes` writes one back. Each matrix grid is run-length encoded then zlib-compressed; the thumbnail is decoded from its on-disk `BGRA` pixels.

The decode is model-lossless, not byte-exact: a re-encoded file decodes to the same model, but its zlib streams and RLE run boundaries are encoder choices and need not match the original bytes.

Reads are bounds-checked, so truncated or malformed input is rejected, not masked.

`qb::validate_qb_file`, `qbt::validate_qbt_file`, and `qbcl::validate_qbcl_file` optionally check a file's grids (and, for `.qbcl`, its thumbnail) against their declared sizes. They are not run on decode.

See `qbcl` for the data types.
