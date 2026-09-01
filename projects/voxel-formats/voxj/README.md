# voxj

Core Rust types for the Voxel Json (`.voxj` / `.voxjz`) format: a compact,
human-readable JSON representation of voxel models covering geometry, materials,
and scene hierarchy.

The types are the data model, with optional `serde` support behind the `serde`
feature.

The `objects` module works with an object's opaque position and sample
blocks:

- `decode_voxj_object`: flatten one object into per-voxel positions and one
  sample channel per layer.
- `encode_voxj_object`: re-encode with fixed block encodings.
- `encode_voxj_object_optimized`: pin either block or leave it unset to
  search its candidate encodings, keeping the pairing with the lowest cost.
- `voxj_palette_material_counts`: the material count per referenced palette,
  the widths `packed-base64` needs.

The `validation` module checks a parsed document against the format rules:
`validate_voxj_file` fails on the first finding and `check_voxj_file` runs
every check and reports each result.

Both modules transcode the `*-base64` blocks through the `EncodeBase64` and
`DecodeBase64` traits. The encoding search costs its candidates through
`CostVoxjObject`. Implement the traits, or use `DependenciesImpl` behind the
default `impl` feature: base64 over the `base64` crate, and a cost that
deflates the object's JSON over `flate2` and `serde_json`. Those three
crates are the only dependencies besides `serde`.

See [`docs/voxel-json-file-format.md`](docs/voxel-json-file-format.md) for the
format specification.
