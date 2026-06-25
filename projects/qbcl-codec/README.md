# qbcl-codec

Reads and writes the [Qubicle](https://getqubicle.com/) voxel editor's binary file formats: Qubicle Binary (`.qb`) and Qubicle Binary Tree (`.qbt`). Each format lives in its own module behind a Cargo feature, both enabled by default: `qb` and `qbt`.

- `qb::from_qb_file_bytes` parses bytes into a `qbcl::qb::QbFile`; `qb::to_qb_file_bytes` writes one back. Run-length-encoded and raw voxel data both decode into the same dense grid.
- `qbt::from_qbt_file_bytes` parses bytes into a `qbcl::qbt::QbtFile`; `qbt::to_qbt_file_bytes` writes one back. Matrix voxel data is zlib-compressed.

Reads are bounds-checked, so truncated or malformed input is rejected, not masked.

`qb::validate_qb_file` and `qbt::validate_qbt_file` optionally check a file's matrix grids against their declared sizes. They are not run on decode.

See `qbcl` for the data types.
