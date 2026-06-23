# voxj

Core types for the Voxel Json (`.voxj` / `.voxjz`) format: a compact, human-readable JSON representation of voxel models covering geometry, materials, and scene hierarchy. This crate is the Rust data model with optional `serde` support; reading and writing `.voxj` / `.voxjz` bytes lives in the companion `voxj-codec` crate.

See [`docs/voxel-json-file-format.md`](docs/voxel-json-file-format.md) for the full format specification.
