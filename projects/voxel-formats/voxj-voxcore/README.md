# voxj-voxcore

Converts between Voxel Json documents and the voxcore state. The `voxj` crate
defines the document types; this crate carries a parsed document into
voxcore's in-memory `VoxMain` and back.

## Document conversion

- `from_voxj_file` / `to_voxj_file`: between a parsed `VoxjFile` and a
  `VoxMain`, decoding and re-encoding each object's position and sample
  blocks. The writer searches each object's block encodings for the pairing
  with the lowest cost.
- `VoxjFileBuilder`: the configurable writer, with control over the block
  encodings, the ext block, and when the edit state records each object's
  editor build volume (`EditStateMode`).

Each takes the caller's voxj dependencies: `DecodeBase64` to load,
`EncodeBase64` and `CostVoxjObject` to write. `voxj::DependenciesImpl`
supplies all three.

## Bytes conversion

The `codec` module, behind the default `codec` feature, goes straight to and
from file bytes over `voxj-codec`:

- `codec::from_voxj_bytes`: `.voxj` or `.voxjz` bytes into a `VoxMain`, with
  the container form detected from the leading bytes.
- `codec::to_voxj_bytes` / `codec::to_voxjz_bytes`: a state to compact `.voxj`
  JSON or a `.voxjz` zip archive, choosing each object's block encodings by
  the lowest cost. The `*_with` variants fix the encodings.

Each also takes the codec's dependencies: `DecodeVoxjJson` and `Inflate` to
load, `EncodeVoxjJson` and `Deflate` to write. `voxj_codec::DependenciesImpl`
supplies those and voxj's.

## The ext block

The loaders and writers are generic over the state's ext slot through
voxcore's `VoxExtSlot`. Loading types the document's `ext` block into the
slot. Writing persists the block the slot encodes unless the builder drops
it. The unit slot carries no ext. A `VoxjVoxMain` carries the block verbatim
as a voxcore value tree, whichever format owns it.
