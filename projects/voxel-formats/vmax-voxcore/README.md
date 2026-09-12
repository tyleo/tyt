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
  write. A loaded document writes back exactly through its ext.
- `to_vmax_vox_main`: a bare `VoxMain<()>` to a `VMaxVoxMain` with a
  synthesized ext, which writes as a document synthesized from the scene.
  The hierarchy becomes a tree first. A node placed along several paths is
  cloned per extra path. A node no root reaches is released. `take_ext` on
  the state takes the ext back off.
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
provenance aligned by index with the state's listings. The ext follows the
listings through voxcore's `VoxExt` hooks. A node, object, palette, or
material released or reordered after the load still writes back with the
surviving provenance. A node retained after the load writes like a
synthesized one. voxconv carries the ext through a Voxel Json document's
`ext` block.

The `serde` feature, on by default, derives serde for the ext types.
