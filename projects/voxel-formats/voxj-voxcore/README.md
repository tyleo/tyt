# voxj-voxcore

Converts between Voxel Json documents and the voxcore state. The `voxj` crate
defines the document types; this crate carries a parsed document into
voxcore's in-memory `VoxMain` and back.

## Document conversion

- `from_voxj_file` / `to_voxj_file`: between a parsed `VoxjFile` and a bare
  `VoxMain`, decoding and re-encoding each object's position and sample
  blocks. The loader drops the `ext` block and the writer writes none.
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

- `codec::from_voxj_bytes`: `.voxj` or `.voxjz` bytes into a bare `VoxMain`,
  with the container form detected from the leading bytes.
- `codec::to_voxj_bytes` / `codec::to_voxj_pretty_bytes` /
  `codec::to_voxjz_bytes`: a bare state to compact `.voxj` JSON,
  pretty-printed `.voxj` JSON, or a `.voxjz` zip archive. Each takes
  `VoxjWriteOptions`.
- The `_with_ext` twin of each carries the `ext` block.

Each also takes the codec's dependencies: `DecodeVoxjJson` and `Inflate` to
load, `EncodeVoxjJson` and `Deflate` to write. `voxj_codec::DependenciesImpl`
supplies those and voxj's.

## The ext block

`VoxjVoxExt` holds a document's `ext` block as it was parsed, one slot per key.
The `ext` module's typed path keeps it: `ext::from_voxj_file_with_ext` loads a
`VoxjVoxMain`, a `VoxMain<VoxjVoxExt>`, and `ext::to_voxj_file_with_ext` writes
it back. A document with no block loads an empty ext, and an empty ext writes no
block. The block is parsed, not understood. It follows no hook and goes stale
under a mutation that moves a listing. voxconv decodes the slots into the exts
that follow.
