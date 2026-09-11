# mvox-voxcore

Converts between MagicaVoxel `.vox` files and the voxcore state. The `mvox`
crate defines the file model. This crate carries a decoded file into
voxcore's in-memory `VoxMain` and back.

## File conversion

- `from_mvox_file` / `to_mvox_file`: between a decoded `MVoxFile` and a bare
  `VoxMain<()>`. Models become objects. The 256-color palette and the
  materials become one shared palette of value pools. The scene graph
  becomes the hierarchy nodes. The writer synthesizes the file from the
  scene. Each object becomes one model, one global palette gathers every
  used color, and the scene graph mirrors the hierarchy. Rotation and scale
  drop.

## Bytes conversion

The `codec` module, behind the default `codec` feature, goes straight to and
from `.vox` bytes over `mvox-codec`:

- `codec::from_mvox_bytes`: `.vox` bytes into a bare `VoxMain<()>`.
- `codec::to_mvox_bytes`: a state to `.vox` bytes.

## The ext

`MVoxExt` holds the MagicaVoxel state with no native voxcore home. The bare
converters drop it on load and synthesize the file on write. The `ext`
module's typed path keeps the ext:

- `ext::from_mvox_file_with_ext` loads a file into a `MVoxVoxMain`, a
  `VoxMain<MVoxExt>` carrying the ext. `ext::to_mvox_file_with_ext`
  writes the state back exactly.
- `codec::from_mvox_bytes_with_ext` and `codec::to_mvox_bytes_with_ext` do
  the same for `.vox` bytes.

The ext follows the state's listings through voxcore's `VoxExt` hooks. After
a node or object is released or reordered, the file still writes back with
the surviving provenance. A released material shifts the recorded ids above
it. The writer errors on a node retained after the load because it has no
scene node. voxconv carries the ext through a Voxel Json document's `ext`
block.

The `serde` feature, on by default, derives serde for the ext types.
