# voxj-codec

Reads and writes Voxel Json `.voxj` / `.voxjz` documents over the data types
the `voxj` crate defines.

## Load and save

Load and save leave each object's position and sample blocks encoded.

- `from_voxj_file_bytes` / `to_voxj_file_bytes`: the uncompressed `.voxj` JSON
  form.
- `from_voxjz_file_bytes` / `to_voxjz_file_bytes`: the zip-packaged `.voxjz`
  form.
- `from_voxj_or_voxjz_file_bytes`: either form, detected by its leading bytes.
- `to_voxj_pretty_file_bytes`: pretty-printed `.voxj` JSON.

Each transcodes the JSON through the `DecodeVoxjJson` and `EncodeVoxjJson`
traits and the `.voxjz` member through `Deflate` and `Inflate`. Implement the
traits, or use `DependenciesImpl` behind the default `impl` feature: JSON over
`serde_json` with exact float parsing, and deflate over `flate2`.
`DependenciesImpl` also implements the `voxj` crate's traits, so one value
serves both crates. Those two crates are the only dependencies besides `voxj`.

The `voxj` crate's `objects` module decodes and re-encodes the blocks, and
its `validation` module checks a parsed document against the format rules.

See the `voxj` crate for the format specification.
