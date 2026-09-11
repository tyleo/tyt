# voxconv

Reads and writes voxel file formats through the voxcore state. Each format's
`-voxcore` bridge crate owns its conversion. This crate picks the bridge for
a format and moves a document's files through it.

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

These functions move a document through a bridge as a bare `VoxMain<()>`:

- `read_document_files` / `write_document_files`: between a path on disk and
  a document's files, through the caller's `ReadFile`, `ListDir`, and
  `WriteFile`
- `read` / `write`: between a document's files and a bare `VoxMain`. A read
  drops the format's ext. A write synthesizes the file from the scene
- `load` / `save`: `read_document_files` then `read`, and `write` then
  `write_document_files`
- `check_document_files`: a `VoxCheck` for whether the files decode, then one
  per spec check the format defines
- `voxj_version_from_bytes`: the version field of a Voxel Json document

Each takes a `Dependencies` that returns the codec dependencies of each
enabled format. A format feature enables its bridge's `codec` feature.
`DependenciesImpl`, behind the `impl` feature, binds std's filesystem and
each format's codec impl.

## The Ext

A format's ext is the state its bridge keeps so a loaded file writes back
exactly. The `ext` feature, on by default, opens the `ext` module, where the
typed pairs carry that state boxed in a `VoxconvVoxMain`, a
`VoxMain<Box<dyn VoxconvExt>>`. `VoxconvExt` is any `VoxExt` that can
downcast and clone:

- `read_with_ext` / `write_with_ext`: a read boxes the ext the bridge loads.
  A write downcasts the box to the format's ext. Failing that, the write
  encodes the box to its Voxel Json `ext` block and takes the format's entry
  from it. A box with no entry for the format writes the synthesized file
  the bare pair writes.
- `load_with_ext` / `save_with_ext`: the same from and to a path

The block's keys live here because voxconv is where the formats meet Voxel Json.
A bridge knows nothing of its key. A Voxel Json document's `ext` block decodes
into a `CompositeVoxExt`, one boxed ext per entry in the block's order. An entry
under an enabled format's key becomes that format's ext, and any other entry
stays an `InertVoxExt`. A mutation such as an object selection keeps a format's
ext aligned because the hooks forward to every entry. A write back to that
format takes its ext from the composite, and a write to Voxel Json encodes the
composite back to the block. `composite_vox_ext_from_voxj_vox_ext` and
`voxj_vox_ext_from_ext` do that mapping for a caller too. The `ext` feature
enables each bridge's `serde` feature for that transcode.
