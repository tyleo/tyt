# vmax-voxcore

Converts between Voxel Max packages and the voxcore state. The `vmax` crate
defines the package model. This crate carries a parsed package into voxcore's
in-memory `VoxMain` and back.

## Document conversion

The state is a `VMaxVoxMain`, a `VoxMain<VMaxExt>` carrying the Voxel Max
state with no native voxcore home as its ext.

- `from_vmax_file` / `to_vmax_file`: between a parsed `VMaxFile` and a
  `VMaxVoxMain`. Geometry, palettes, and hierarchy become native voxcore
  entities. Each object's snapshots are decoded on the fly and re-encoded on
  write. A loaded document writes back exactly through its ext. The writer
  reads each object's one palette in Voxel Max's layout, the one the loader
  builds. `baseColor` and `emissiveColor` hold one value per color cell.
  Every other property holds one value per material slot. A material pairs
  one cell with one slot. The writer converts nothing. An object with a
  second layer, or a palette off the layout, errors where it departs.
- `to_vmax_vox_main`: a bare `VoxMain<()>` to a `VMaxVoxMain` with a
  synthesized ext, which writes as a document synthesized from the scene.
  The hierarchy becomes a tree first. A node placed along several paths is
  cloned per extra path. A node no root reaches is released. `take_ext` on
  the `VMaxVoxMain` takes the ext back off.
- `VMaxWriteOptions`: the writer's options. `Default` stores palette colors
  as PNG and keeps the ext's camera. `VMaxColorFormat` picks where each
  palette's colors are stored. `SceneCameraSource` picks the scene camera the
  document opens with.

## Package conversion

The `codec` module, behind the default `codec` feature, goes straight to and
from a package's files over `vmax-codec`:

- `codec::from_vmax_package`: a package's files into a `VMaxVoxMain`, read
  through the caller's list and resolve closures.
- `codec::to_vmax_package`: a `VMaxVoxMain` to a package's files, written
  through the caller's write closure.

Each takes the codec's dependencies: `DecompressLzfse`, `DecodeVMaxPlist`,
`DecodePng`, and `DecodeVMaxSceneJson` to load, and their encode
counterparts to write. `vmax_codec::DependenciesImpl` supplies them all.
This crate's `impl` feature turns it on.

## The ext

`VMaxExt` holds the Voxel Max state with no native voxcore home: the scene
around its objects and groups, and per-node, per-palette, and per-object
provenance keyed by the entity's id. Every live entity has an entry. The ext
holds only what the scene cannot derive. A node's name, position, scale, and
parent, an object's content box, and a group's bounds are read from the
scene on write. A node's preserved axis-angle writes back while it still
decodes to the node's rotation. A node rotated after the load writes its
live rotation.

The ext follows the state through voxcore's `VoxExt` hooks, each of which
sees the `VoxState`. A node, object, or palette retained after the load
takes the entry `to_vmax_vox_main` would synthesize for it as it is retained:
a fresh UUID, a fresh index triplet, and the default anchors or editor
session.

A release drops the entry. `gc` rekeys every entry to its compacted id. An
exact material list follows its material pools' surviving values, so a pruned
palette keeps the exact materials its slots still draw. The writer errors on
an entity with no entry, on a node with two parents, and on a root that is
also a child, because Voxel Max holds a tree. voxconv carries the ext through
a Voxel Json document's `ext` block.

The `serde` feature, on by default, derives serde for the ext types. A map
keyed by id serializes under the bare ids.
