# voxj-voxcore

Converts between Voxel Json documents and the voxcore state. The `voxj` crate
defines the document types; this crate carries a parsed document into
voxcore's in-memory `VoxMain` and back.

## Document conversion

- `from_voxj_file` / `to_voxj_file`: between a parsed `VoxjFile` and a
  `VoxMain`, decoding and re-encoding each object's position and sample
  blocks.
- `VoxjWriteOptions`: the writer's options. `Default` searches each object's
  block encodings for the lowest cost, keeps the ext block, and records the
  edit state only when an object carries margin around its live voxels.
  `EditStateMode` picks when the edit state records each object's editor
  build volume.

Each takes the caller's voxj dependencies: `DecodeBase64` to load,
`EncodeBase64` and `CostVoxjObject` to write. `voxj::DependenciesImpl`
supplies all three.

## Bytes conversion

The `codec` module, behind the default `codec` feature, goes straight to and
from file bytes over `voxj-codec`:

- `codec::from_voxj_bytes`: `.voxj` or `.voxjz` bytes into a `VoxMain`, with
  the container form detected from the leading bytes.
- `codec::to_voxj_bytes` / `codec::to_voxj_pretty_bytes` /
  `codec::to_voxjz_bytes`: a state to compact `.voxj` JSON, pretty-printed
  `.voxj` JSON, or a `.voxjz` zip archive. Each takes `VoxjWriteOptions`.

Each also takes the codec's dependencies: `DecodeVoxjJson` and `Inflate` to
load, `EncodeVoxjJson` and `Deflate` to write. `voxj_codec::DependenciesImpl`
supplies those and voxj's.

## The ext block

The loaders are generic over the state's ext through voxcore's
`VoxExtBlockCodec` and the writers through `VoxExt`. Loading decodes the
document's `ext` block into the ext. Writing persists the block the ext
encodes unless the options drop it or the block is empty. A `()` ext carries
nothing. A `VoxjVoxMain` carries the block verbatim as a voxcore value tree,
whichever format owns it.
