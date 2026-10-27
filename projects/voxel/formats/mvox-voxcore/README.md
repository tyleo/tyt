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
index map, each material's exact optional fields keyed by material, and per
scene-node provenance keyed by hierarchy node. An entry holds only what the
scene cannot derive. A scene node's name and child links come from the
voxcore node at write. A shape's model indices come from the object listing.
A node renamed or an object moved after the load writes as it stands. The
kind, the scene-node id, the layer, the exact frames, and the per-frame
model list stay in the entry.

The ext follows the state through voxcore's `VoxExt` hooks, which see the
`VoxState`. A node retained after the load takes a synthesized entry on the
spot, a shape when it places objects, a transform when it has one child
node, a group otherwise, under a fresh scene-node id. A released node or
material drops its entry. Releasing the palette drops every material entry
and the index map. `gc` rekeys every entry. An index-map byte for a
released material takes an index the compaction freed, which keeps the map
a permutation. The writer errors on an ext out of step with the
state, on a node whose children do not fit its kind, on a transform node
moved after the load without its frames, on a second palette, and on a
material past the 256 color slots. An object release is refused while a
shape entry still draws the object. voxconv carries the ext through a Voxel
Json document's `ext` block.

The `serde` feature, on by default, derives serde for the ext types. Ids
serialize as bare integers.
