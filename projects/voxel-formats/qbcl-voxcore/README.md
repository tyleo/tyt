# qbcl-voxcore

Converts between Qubicle files and the voxcore state. The `qbcl` crate
defines the file models for the three Qubicle formats: Qubicle Binary
(`.qb`), Qubicle Binary Tree (`.qbt`), and Qubicle Construction Library
(`.qbcl`). This crate carries a decoded file into voxcore's in-memory
`VoxMain` and back.

## File conversion

- `from_qb_file` / `to_qb_file`: between a decoded `QbFile` and a
  `QbVoxMain`. Each matrix becomes an object placed by a hierarchy
  node, all sharing one `baseColor` palette.
- `from_qbt_file` / `to_qbt_file`: between a decoded `QbtFile` and a
  `QbtVoxMain`. Matrix and compound grids become objects sharing one
  palette, and the scene tree becomes the hierarchy.
- `from_qbcl_file` / `to_qbcl_file`: between a decoded `QbclFile` and a
  `QbclVoxMain`, the same way.

## Bytes conversion

The `codec` module, behind the default `codec` feature, goes straight to and
from file bytes over `qbcl-codec`: `codec::from_qb_bytes` /
`codec::to_qb_bytes`, `codec::from_qbt_bytes` / `codec::to_qbt_bytes`, and
`codec::from_qbcl_bytes` / `codec::to_qbcl_bytes`. The `.qbt` and `.qbcl`
conversions take the codec's dependencies, `DecompressZlib` to load and
`CompressZlib` to write. `qbcl_codec::DependenciesImpl` supplies both. This
crate's `impl` feature turns on the codec's.

## The ext

The Qubicle state with no native voxcore home rides in the format's ext:
`QbExt`, `QbtExt`, or `QbclExt`. The loader stashes it on
the state's ext slot, so a loaded file writes back exactly. The `.qb` and
`.qbt` writers require it. The `.qbcl` writer synthesizes a file from the bare
scene when the ext is absent, such as for a state loaded from another format.
The `ext` feature, on by default, keys each ext into a document's `ext` block
under its `qb`, `qbt`, or `qbcl` key through voxcore's
`VoxExtCodec`. A Voxel Json document carries the ext in that block.
