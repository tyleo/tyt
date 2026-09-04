# qbcl-codec

Reads and writes the [Qubicle](https://getqubicle.com/) voxel editor's binary file formats: Qubicle Binary (`.qb`), Qubicle Binary Tree (`.qbt`), and Qubicle Construction Library (`.qbcl`). Each format lives in its own module behind a Cargo feature, all enabled by default: `qb`, `qbt`, and `qbcl`.

- `qb::from_qb_file_bytes` parses bytes into a `qbcl::qb::QbFile`, and `qb::to_qb_file_bytes` writes one back. Run-length-encoded and raw voxel data both decode into the same dense grid.
- `qbt::from_qbt_file_bytes` parses bytes into a `qbcl::qbt::QbtFile`, and `qbt::to_qbt_file_bytes` writes one back. Matrix voxel data is zlib-compressed.
- `qbcl::from_qbcl_file_bytes` parses bytes into a `qbcl::qbcl::QbclFile`, and `qbcl::to_qbcl_file_bytes` writes one back. Each matrix grid is run-length encoded then zlib-compressed. The thumbnail is decoded from its on-disk `BGRA` pixels.

The decode is model-lossless, not byte-exact. A re-encoded file decodes to the same model, but its zlib streams and RLE run boundaries are encoder choices that need not match the original bytes.

Reads are bounds-checked, so truncated or malformed input is rejected.

## Dependencies

The `.qbt` and `.qbcl` readers and writers transcode each matrix's zlib stream through the `DecompressZlib` and `CompressZlib` traits, taking `dependencies` first. Implement the traits, or use `DependenciesImpl` behind the default `impl` feature, which covers both over `flate2`. `flate2` is the only dependency besides `qbcl`. The `.qb` functions need no dependencies, so a lean `qb` build depends on `qbcl` alone.

See `qbcl` for the data types and the `validation` module.
