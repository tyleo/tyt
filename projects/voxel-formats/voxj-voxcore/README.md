# voxj-voxcore

Converts between Voxel Json documents and the voxcore state. The `voxj` crate
defines the document types; this crate carries a parsed document into
voxcore's in-memory `VoxMain` and back.

## Document conversion

The state is a `VoxjVoxMain`, a `VoxMain<VoxjVoxExt>` carrying the document's
`ext` block as it was parsed.

- `from_voxj_file` / `to_voxj_file`: between a parsed `VoxjFile` and a
  `VoxjVoxMain`, decoding and re-encoding each object's position and sample
  blocks. A document with no block loads an empty ext, and an empty ext
  writes no block.
- `to_voxj_vox_main`: a bare `VoxMain<()>` to a `VoxjVoxMain` with an empty
  block. `take_ext` on the state takes the block back off.
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

- `codec::from_voxj_bytes`: `.voxj` or `.voxjz` bytes into a `VoxjVoxMain`,
  with the container form detected from the leading bytes.
- `codec::to_voxj_bytes` / `codec::to_voxj_pretty_bytes` /
  `codec::to_voxjz_bytes`: a `VoxjVoxMain` to compact `.voxj` JSON,
  pretty-printed `.voxj` JSON, or a `.voxjz` zip archive. Each takes
  `VoxjWriteOptions`.

Each also takes the codec's dependencies: `DecodeVoxjJson` and `Inflate` to
load, `EncodeVoxjJson` and `Deflate` to write. `voxj_codec::DependenciesImpl`
supplies those and voxj's.

## The ext block

`VoxjVoxExt` holds a document's `ext` block as it was parsed, one slot per
key. The block is parsed, not understood. It follows no hook and goes stale
under a mutation that moves a listing. voxconv decodes the slots into the
exts that follow.
