# vmax-voxcore

Converts between Voxel Max packages and the voxcore state. The `vmax` crate
defines the package model. This crate carries a parsed package into voxcore's
in-memory `VoxMain` and back.

## Document conversion

The state is a `VMaxVoxMain`, a `VoxMain<VMaxExt>` carrying the Voxel Max
state with no native voxcore home as its ext.

- `from_vmax_file` / `to_vmax_file`: between a parsed `VMaxFile` and a
  `VMaxVoxMain`. Geometry, palettes, and hierarchy become voxcore entities,
  turned from Voxel Max's Z-up axes onto voxcore's Y-up axes. The ext keeps
  the rest, so a round trip keeps world-space geometry, hierarchy, palette
  provenance, external meshes, and the package files the scene does not
  model. Each object writes in the smallest editor cube from 32 to 512 that
  holds its grid. The writer reads palettes in the layout the loader builds.
  `baseColor` and `emissiveColor` hold one value per color cell, and every
  other property holds one value per material slot. A second layer, a palette
  off that layout, or a grid wider than 512 errors. Both take the `vmax`
  crate's base64 dependencies because the ext keeps every unmodeled file as
  base64 text.
- `to_vmax_vox_main`: gives a bare `VoxMain<()>` a synthesized ext and
  reshapes its scene to fit Voxel Max:
  1. The hierarchy becomes a tree. A node placed along several paths is
     cloned, and a node no root reaches is released
  2. Voxel Max objects are leaves, so a node placing both objects and child
     nodes moves its objects onto a child node
  3. A palette needing more than Voxel Max's 8 material slots splits into one
     palette per set of materials an object samples
  4. Each palette's material properties are laid out one value per slot

  `take_ext` on the `VMaxVoxMain` takes the ext back off.
- `VMaxWriteOptions`: `VMaxColorFormat` picks where palette colors are
  stored, PNG by default. `SceneCameraSource` picks the scene camera, the
  ext's by default. An object with no kept camera gets one framed on its
  content.

## Package conversion

The `codec` module, behind the default `codec` feature, goes straight to and
from a package's files over `vmax-codec`:

- `codec::from_vmax_package`: a package's files into a `VMaxVoxMain`, read
  through the caller's list and resolve closures.
- `codec::to_vmax_package`: a `VMaxVoxMain` to a package's files, written
  through the caller's write closure.

Each takes the codec's dependencies: `DecompressLzfse`, `DecodeVMaxPlist`,
`DecodePng`, and `DecodeVMaxSceneJson` to load, and their encode
counterparts to write, plus `vmax`'s `EncodeBase64` to load and
`DecodeBase64` to write. `vmax_codec::DependenciesImpl` supplies them all.
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

A release drops the entry. An external mesh's files go with the last node
placing it. `gc` rekeys every entry to its compacted id. An exact material
list follows its material pools' surviving values, so a pruned palette keeps
the exact materials its slots still draw. The writer errors on an entity with
no entry, on a node with two parents, and on a root that is also a child,
because Voxel Max holds a tree. voxconv carries the ext through a Voxel Json
document's `ext` block.

The `serde` feature, on by default, derives serde for the ext types. A map
keyed by id serializes under the bare ids.
