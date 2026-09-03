# vmax-voxcore

Converts between Voxel Max packages and the voxcore state. The `vmax` crate
defines the package model. This crate carries a parsed package into voxcore's
in-memory `VoxMain` and back.

## Document conversion

- `from_vmax_file` / `to_vmax_file`: between a parsed `VMaxFile` and a
  `VoxelMaxVoxMain`. Geometry, palettes, and hierarchy become native voxcore
  entities. Each object's snapshots are decoded on the fly and re-encoded on
  write.
- `VmaxFileBuilder`: the configurable writer. `VoxelMaxColorFormat` picks
  where each palette's colors are stored and `SceneCameraSource` the scene
  camera the document opens with.

## Package conversion

The `codec` module, behind the default `codec` feature, goes straight to and
from a package's files over `vmax-codec`:

- `codec::from_vmax_package`: a package's files into a `VoxelMaxVoxMain`,
  read through the caller's list and resolve closures.
- `codec::to_vmax_package`: a state to a package's files, written through the
  caller's write closure.

## The ext

The Voxel Max state with no native voxcore home rides in the `VoxelMaxExt`
the loader stashes on the state's ext slot, so a document loaded from a
package writes back exactly. A state without one, such as one loaded from
another format, has its document synthesized from the bare scene. The `ext`
feature, on by default, keys the ext into a document's `ext` block under the
`voxel-max` key through voxcore's `VoxExtCodec`. A Voxel Json document carries
the ext in that block.
