# mvox-voxcore

Converts between MagicaVoxel `.vox` files and the voxcore state. The `mvox`
crate defines the file model. This crate carries a decoded file into
voxcore's in-memory `VoxMain` and back.

## File conversion

The state is a `MVoxVoxMain`, a `VoxMain<MVoxExt>` carrying the MagicaVoxel
state with no native voxcore home as its ext.

- `from_mvox_file` / `to_mvox_file`: between a decoded `MVoxFile` and a
  `MVoxVoxMain`. Models become objects. The 256-color palette and the
  materials become one shared palette of value pools. The scene graph
  becomes the hierarchy nodes, one per scene node. A loaded file writes back
  exactly through its ext.
- `to_mvox_vox_main`: a bare `VoxMain<()>` to a `MVoxVoxMain` with a
  synthesized ext, which writes as a file synthesized from the scene. The
  state takes MagicaVoxel's shape first. Every palette merges into one
  256-color table gathering every used color. The hierarchy rebuilds with
  one node per scene node under one synthetic root. Names ride the transform
  nodes. Rotation and scale drop. A grid over 256 voxels per axis or more
  than 255 distinct colors is an error. `take_ext` on the state takes the
  ext back off.

## Bytes conversion

The `codec` module, behind the default `codec` feature, goes straight to and
from `.vox` bytes over `mvox-codec`:

- `codec::from_mvox_bytes`: `.vox` bytes into a `MVoxVoxMain`.
- `codec::to_mvox_bytes`: a `MVoxVoxMain` to `.vox` bytes.

## The ext

`MVoxExt` holds the MagicaVoxel state with no native voxcore home: the
format version, the layers, cameras, render settings, palette notes, and
index map, each material's exact optional fields, and per scene-node
provenance aligned by index with the hierarchy nodes. The ext follows the
state's listings through voxcore's `VoxExt` hooks. After a node or object is
released or reordered, the file still writes back with the surviving
provenance. A released material shifts the recorded ids above it. The writer
errors on a node retained after the load because it has no scene node.
voxconv carries the ext through a Voxel Json document's `ext` block.

The `serde` feature, on by default, derives serde for the ext types.
