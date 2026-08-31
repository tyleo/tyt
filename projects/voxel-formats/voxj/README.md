# voxj

Core Rust types for the Voxel Json (`.voxj` / `.voxjz`) format: a compact,
human-readable JSON representation of voxel models covering geometry, materials,
and scene hierarchy.

The types are the data model, with optional `serde` support behind the `serde`
feature.

The default `objects` feature adds the `objects` module for working with an
object's opaque position and sample blocks:

- `decode_voxj_object`: flatten one object into per-voxel positions and one
  sample channel per layer.
- `encode_voxj_object`: re-encode with fixed block encodings.
- `voxj_palette_material_counts`: the material count per referenced palette,
  the widths `packed-base64` needs.

The default `optimize` feature adds `encode_voxj_object_optimized` to the
`objects` module: pin either block or leave it unset to search its candidate
encodings, keeping the smallest deflated pairing.

The default `validation` feature adds the `validation` module, which checks a
parsed document against the format rules: `validate_voxj_file` fails on the
first finding and `check_voxj_file` runs every check and reports each result.
The geometry checks decode each object, so the feature builds on `objects`.

See [`docs/voxel-json-file-format.md`](docs/voxel-json-file-format.md) for the
format specification.
