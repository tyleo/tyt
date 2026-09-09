# qbcl-voxcore

Converts between Qubicle files and the voxcore state. The `qbcl` crate
defines the file models for the three Qubicle formats: Qubicle Binary
`.qb`, Qubicle Binary Tree `.qbt`, and Qubicle Construction Library
`.qbcl`. This crate carries a decoded file into voxcore's in-memory
`VoxMain` and back.

## File conversion

- `from_qb_file` / `to_qb_file`: between a decoded `QbFile` and a bare
  `VoxMain<()>`. Each matrix becomes an object placed by a hierarchy node.
  The objects share one `baseColor` palette. The writer synthesizes the file
  from the scene. Each object placement becomes one matrix at its world
  translation. Grouping, rotation, scale, and alpha drop.
- `from_qbt_file` / `to_qbt_file`: between a decoded `QbtFile` and a bare
  `VoxMain<()>`. Matrix and compound grids become objects sharing one
  palette. The scene tree becomes the hierarchy. The writer synthesizes the
  file from the scene under one root model. A group's translation folds into
  its descendant matrices. Group names, rotation, scale, and alpha drop.
- `from_qbcl_file` / `to_qbcl_file`: between a decoded `QbclFile` and a bare
  `VoxMain<()>`. The loader and the writer work as the `.qbt` pair does.
  Group names are kept.

## Bytes conversion

The `codec` module, behind the default `codec` feature, goes straight
between a bare `VoxMain<()>` and file bytes over `qbcl-codec`:

- `codec::from_qb_bytes` / `codec::to_qb_bytes` for `.qb` bytes.
- `codec::from_qbt_bytes` / `codec::to_qbt_bytes` for `.qbt` bytes.
- `codec::from_qbcl_bytes` / `codec::to_qbcl_bytes` for `.qbcl` bytes.

The `.qbt` and `.qbcl` pairs take the codec's dependencies: `DecompressZlib`
to load and `CompressZlib` to write. `qbcl_codec::DependenciesImpl` supplies
both. This crate's `impl` feature turns on the codec's.

## The ext

`QbExt`, `QbtExt`, and `QbclExt` hold the Qubicle state with no native
voxcore home. The bare converters drop it on load and synthesize the file on
write. The `ext` feature, on by default, opens the `ext` module, where the
typed path keeps the ext:

- `ext::from_qb_file_with_ext` loads a file into a `QbVoxMain`, a
  `VoxMain<Option<QbExt>>` carrying the ext. `ext::to_qb_file_with_ext`
  writes the state back exactly. `QbtVoxMain` and `QbclVoxMain` pair the
  same way with their `_with_ext` loaders and writers.
- `codec::from_qb_bytes_with_ext` and `codec::to_qb_bytes_with_ext` do the
  same for `.qb` bytes. The `qbt` and `qbcl` pairs sit beside them.

`QbExt` and `QbtExt` follow the state's listings through voxcore's `VoxExt`
hooks. After a release or a reorder, the file still writes back with the
surviving provenance. An object retained after the load is written as a
synthesized matrix. A node retained after the load is written as a
synthesized node. Each ext enters a document's `ext` block as its `qb`,
`qbt`, or `qbcl` entry through voxcore's `VoxExtEntryCodec`. A Voxel Json
document carries the ext in that block.
