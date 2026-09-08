# voxconv

Reads and writes voxel file formats through the voxcore state. Each format's
`-voxcore` bridge crate owns its conversion. This crate picks the bridge for
a format, moves a document's files through it, and moves the format's ext
into whatever ext type the caller's state carries.

## Formats

`ReadFormat` lists the formats a document can be read as, one variant per
enabled format feature: Goxel (`.gox`), MagicaVoxel (`.vox`), the three
Qubicle formats (`.qb`, `.qbt`, `.qbcl`), Voxel Max (`.vmax`), and Voxel Json
(`.voxj`, `.voxjz`). `WriteFormat` lists the write targets. Voxel Max carries
`vmax::VMaxWriteOptions` for the color format and scene camera. Voxel Json
carries `voxj::VoxjWriteOptions` for the block encodings, ext block, and edit
state. A `voxj::VoxjSerialization` beside the options picks the container.
This crate re-exports both options structs from the bridges.

## Files

A document is a list of `VoxDocumentFile`. Each holds a document-relative
path and its bytes. A single-file format has one entry with an empty path. A
`.vmax` package has one entry per file. Entries under `QuickLook/` keep the
prefix.

## Conversion

These functions move a document through a bridge:

- `read_document_files` / `write_document_files`: between a path on disk and
  a document's files, through the caller's `ReadFile`, `ListDir`, and
  `WriteFile`
- `read` / `write`: between a document's files and a `VoxMain`, with the
  format's ext moved through its block form into or out of the state's ext
- `load` / `save`: `read_document_files` then `read`, and `write` then
  `write_document_files`
- `check_document_files`: a `VoxCheck` for whether the files decode, then one
  per spec check the format defines
- `voxj_version_from_bytes`: the version field of a Voxel Json document

Each takes a `Dependencies` that returns the codec dependencies of each
enabled format. A format feature enables its bridge's `codec` and `ext`
features. `DependenciesImpl`, behind the `impl` feature, binds std's
filesystem and each format's codec impl.
