# voxj

Core Rust types for the Voxel Json (`.voxj` / `.voxjz`) format: a compact,
human-readable JSON representation of voxel models covering geometry, materials,
and scene hierarchy.

The types are the data model, with optional `serde` support behind the `serde`
feature.

See [`docs/voxel-json-file-format.md`](docs/voxel-json-file-format.md) for the
format specification.
