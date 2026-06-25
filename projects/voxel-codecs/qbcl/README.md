# qbcl

A Rust data model for the [Qubicle](https://getqubicle.com/) voxel editor's binary file formats. A file decodes into typed structs, and files round-trip unchanged. The types live here; reading and writing bytes lives in `qbcl-codec`.

Each format lives in its own module behind a Cargo feature, both enabled by default:

- `qb`: Qubicle Binary (`.qb`), a list of named, dense voxel matrices.
- `qbt`: Qubicle Binary Tree (`.qbt`), a scene tree of matrix, model, and compound nodes with a shared color map.

Disable default features and enable just the one you need to drop the other format (and, for `qbt`, its zlib dependency in the codec).
