# goxl-voxcore

Converts between Goxel `.gox` files and the voxcore state. The `goxl` crate
defines the file model. This crate carries a parsed file into voxcore's
in-memory `VoxMain` and back.

## File conversion

- `from_goxl_file` / `to_goxl_file`: between a parsed `GoxlFile` and a
  `GoxelVoxMain`. The shared `BL16` voxel blocks become objects sharing one
  `baseColor` palette. The `LAYR` layers become the hierarchy nodes that
  place them.

## Bytes conversion

The `codec` module, behind the default `codec` feature, goes straight to and
from `.gox` bytes over `goxl-codec`:

- `codec::from_goxl_bytes`: `.gox` bytes into a `GoxelVoxMain`.
- `codec::to_goxl_bytes`: a state to `.gox` bytes.

Each takes the codec's dependencies: `DecodePng` to load and `EncodePng` to
write. `goxl_codec::DependenciesImpl` supplies both. This crate's `impl`
feature turns it on.

## The ext

The Goxel state with no native voxcore home rides in the `GoxelExt` the
loader stashes on the state's ext slot, so a file loaded from Goxel writes
back exactly. A state without an ext, such as a state loaded from another
format, has its file synthesized from the bare scene. The `ext` feature, on
by default, keys the ext into a document's `ext` block under the `goxel` key
through voxcore's `VoxExtCodec`. A Voxel Json document carries the ext in
that block.
