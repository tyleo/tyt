# voxj-voxcore

Converts between Voxel Json documents and the voxcore state. The `voxj` crate
defines the document types; this crate carries a parsed document into
voxcore's in-memory `VoxMain` and back.

## Document conversion

- `from_voxj_file` / `to_voxj_file`: between a parsed `VoxjFile` and a
  `VoxMain`, decoding and re-encoding each object's position and sample
  blocks.
- `VoxjFileBuilder`: the configurable writer, with control over the block
  encodings, the ext block, and when the edit state records each object's
  editor build volume (`EditStateMode`).

## Bytes conversion

The `codec` module, behind the default `codec` feature, pulls in `voxj-codec`
and goes straight to and from file bytes:

- `codec::from_voxj_bytes`: `.voxj` or `.voxjz` bytes into a `VoxMain`,
  the container form detected from the leading bytes.
- `codec::to_voxj_bytes` / `codec::to_voxjz_bytes`: a state to compact `.voxj`
  JSON or a `.voxjz` zip archive, choosing the smallest block encodings per
  object; the `*_with` variants fix the encodings instead.

## The ext block

The loaders and writers are generic over the state's ext slot through
voxcore's `VoxExtSlot`. Loading types the document's `ext` block into the
slot. Writing persists the block the slot encodes unless the builder drops
it. The unit slot carries no ext. A `VoxjVoxMain` carries the block verbatim
as a voxcore value tree, whichever format owns it.
