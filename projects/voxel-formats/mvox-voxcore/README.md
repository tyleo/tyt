# mvox-voxcore

Converts between MagicaVoxel `.vox` files and the voxcore state. The `mvox`
crate defines the file model. This crate carries a decoded file into
voxcore's in-memory `VoxMain` and back.

## File conversion

- `from_mvox_file` / `to_mvox_file`: between a decoded `MVoxFile` and a
  `MVoxVoxMain`. Models become objects, the 256-color palette and the
  materials become one shared palette of value pools, and the scene graph
  becomes the hierarchy nodes.

## Bytes conversion

The `codec` module, behind the default `codec` feature, goes straight to and
from `.vox` bytes over `mvox-codec`:

- `codec::from_mvox_bytes`: `.vox` bytes into a `MVoxVoxMain`.
- `codec::to_mvox_bytes`: a state to `.vox` bytes.

## The ext

The MagicaVoxel state with no native voxcore home rides in the `MVoxExt`.
The loader stores it as the state's ext, so a file loaded from MagicaVoxel
writes back exactly. A state without an ext, such as a state loaded from
another format, has its file synthesized from the bare scene. The `ext`
feature, on by default, keys the ext into a document's `ext` block under the
`mvox` key through voxcore's `VoxExtEntryCodec`. A Voxel Json document carries
the ext in that block.
