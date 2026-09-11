# vmax-voxcore

Converts between Voxel Max packages and the voxcore state. The `vmax` crate
defines the package model. This crate carries a parsed package into voxcore's
in-memory `VoxMain` and back.

## Document conversion

- `from_vmax_file` / `to_vmax_file`: between a parsed `VMaxFile` and a bare
  `VoxMain<()>`. Geometry, palettes, and hierarchy become native voxcore
  entities. Each object's snapshots are decoded on the fly and re-encoded on
  write. The writer synthesizes the document from the scene.
- `VMaxWriteOptions`: the writer's options. `Default` stores palette colors
  as PNG and keeps the path's camera. `VMaxColorFormat` picks where each
  palette's colors are stored. `SceneCameraSource` picks the scene camera the
  document opens with.

## Package conversion

The `codec` module, behind the default `codec` feature, goes straight to and
from a package's files over `vmax-codec`:

- `codec::from_vmax_package`: a package's files into a bare `VoxMain<()>`,
  read through the caller's list and resolve closures.
- `codec::to_vmax_package`: a state to a package's files, written through the
  caller's write closure.

Each takes the codec's dependencies: `DecompressLzfse`, `DecodeVMaxPlist`,
`DecodePng`, and `DecodeVMaxSceneJson` to load, and their encode
counterparts to write. `vmax_codec::DependenciesImpl` supplies them all.
This crate's `impl` feature turns it on.

## The ext

`VMaxExt` holds the Voxel Max state with no native voxcore home. The bare
converters drop it on load and synthesize it on write. The `ext` module's
typed path keeps it:

- `ext::from_vmax_file_with_ext` loads a document into a `VMaxVoxMain`, a
  `VoxMain<VMaxExt>` carrying the ext. `ext::to_vmax_file_with_ext`
  writes it back exactly.
- `codec::from_vmax_package_with_ext` and `codec::to_vmax_package_with_ext`
  do the same for a package's files.

The ext follows the state's listings through voxcore's `VoxExt` hooks. A
node, object, palette, or material released or reordered after the load
still writes back with the surviving provenance. A node retained after the
load writes like a synthesized one. voxconv carries the ext through a Voxel
Json document's `ext` block.

The `serde` feature, on by default, derives serde for the ext types.
