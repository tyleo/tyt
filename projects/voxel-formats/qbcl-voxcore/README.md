# qbcl-voxcore

Converts between Qubicle files and the voxcore state. The `qbcl` crate
defines the file models for the three Qubicle formats: Qubicle Binary
`.qb`, Qubicle Binary Tree `.qbt`, and Qubicle Construction Library
`.qbcl`. This crate carries a decoded file into voxcore's in-memory
`VoxMain` and back.

## File conversion

Each format has a `VoxMain` carrying the format's state with no native
voxcore home as its ext. `QbVoxMain` is a `VoxMain<QbExt>`, and `QbtVoxMain`
and `QbclVoxMain` follow the same pattern.

- `from_qb_file` / `to_qb_file`: between a decoded `QbFile` and a
  `QbVoxMain`. Each matrix becomes an object placed by a hierarchy node.
  The objects share one `baseColor` palette. A loaded file writes back
  exactly through its ext.
- `from_qbt_file` / `to_qbt_file`: between a decoded `QbtFile` and a
  `QbtVoxMain`. Matrix and compound grids become objects sharing one
  palette. The scene tree becomes the hierarchy.
- `from_qbcl_file` / `to_qbcl_file`: between a decoded `QbclFile` and a
  `QbclVoxMain`. The loader and the writer work as the `.qbt` pair does.
- `to_qb_vox_main`, `to_qbt_vox_main`, and `to_qbcl_vox_main`: a bare
  `VoxMain<()>` to the format's state with a synthesized ext, which writes
  as a file synthesized from the scene. `take_ext` on the state takes the
  ext back off.

The `.qb` conversion flattens the scene first. Each object placement becomes
one root placing one object at its world translation. An object placed
several times is duplicated per extra placement. Grouping, rotation, scale,
and alpha drop. The `.qbt` and `.qbcl` conversions fold the scene under one
root model instead. A node placed along several paths is cloned per extra
path. Each node takes its world position. An object no node places gets a
node of its own at the origin. A `.qbt` group loses its name because a
model carries none. Rotation, scale, and alpha drop.

## Bytes conversion

The `codec` module, behind the default `codec` feature, goes straight
between a format's state and file bytes over `qbcl-codec`:

- `codec::from_qb_bytes` / `codec::to_qb_bytes` for `.qb` bytes.
- `codec::from_qbt_bytes` / `codec::to_qbt_bytes` for `.qbt` bytes.
- `codec::from_qbcl_bytes` / `codec::to_qbcl_bytes` for `.qbcl` bytes.

The `.qbt` and `.qbcl` pairs take the codec's dependencies: `DecompressZlib`
to load and `CompressZlib` to write. `qbcl_codec::DependenciesImpl` supplies
both. This crate's `impl` feature turns on the codec's.

## The ext

`QbExt`, `QbtExt`, and `QbclExt` hold the Qubicle state with no native
voxcore home: the header, plus per-matrix or per-node provenance aligned by
index with the state's listings. The exts follow the listings through
voxcore's `VoxExt` hooks. After a release or a reorder, the file still
writes back with the surviving provenance. An object retained after the load
is written as a synthesized matrix. A node retained after the load is
written as a synthesized node. voxconv carries each ext through a Voxel Json
document's `ext` block.

The `serde` feature, on by default, derives serde for the ext types.
