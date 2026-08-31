# voxj-voxcore

Converts between Voxel Json documents and the voxcore state. The `voxj` crate
defines the document types; this crate carries a parsed document into
voxcore's in-memory `VoxMain` and back.

## Document conversion

- `from_voxj_file` / `to_voxj_file`: between a parsed `VoxjFile` and a
  `VoxjVoxMain`, decoding and re-encoding each object's position and sample
  blocks.
- `VoxjFileBuilder`: the configurable writer, with control over the block
  encodings, the ext block, and when the edit state records each object's
  editor build volume (`EditStateMode`).

## Bytes conversion

The `codec` module, behind the default `codec` feature, pulls in `voxj-codec`
and goes straight to and from file bytes:

- `codec::from_voxj_bytes`: `.voxj` or `.voxjz` bytes into a `VoxjVoxMain`,
  the container form detected from the leading bytes.
- `codec::to_voxj_bytes` / `codec::to_voxjz_bytes`: a state to compact `.voxj`
  JSON or a `.voxjz` zip archive, choosing the smallest block encodings per
  object; the `*_with` variants fix the encodings instead.

## The ext block

`VoxjVoxMain` is the state with the document's `ext` block in its ext slot as
a voxcore value tree. Loading keeps whatever block the document carries,
whichever format owns it. Writing persists the carried block back; the
builder can drop it. Typing the block is the caller's business:
`VoxMain::map_ext` carries it in and out of the slot.
