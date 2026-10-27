# qbcl

A Rust data model for the [Qubicle](https://getqubicle.com/) voxel editor's binary file formats. A file decodes into typed structs and round-trips unchanged. The types live here. `qbcl-codec` reads and writes the bytes.

Each format lives in its own module behind a Cargo feature, all enabled by default:

- `qb`: Qubicle Binary (`.qb`), a list of named, dense voxel matrices.
- `qbt`: Qubicle Binary Tree (`.qbt`), a scene tree of matrix, model, and compound nodes with a shared color map.
- `qbcl`: Qubicle Construction Library (`.qbcl`), Qubicle's native format: a header with a preview thumbnail and metadata, then a scene tree of matrix, model, and compound nodes.

The `validation` module's `validate_qb_file`, `validate_qbt_file`, and `validate_qbcl_file` check a decoded file's grids, and for `.qbcl` its thumbnail, against their declared sizes. The codec does not run them on decode.

Disable default features and enable just the ones you need to drop the rest.
