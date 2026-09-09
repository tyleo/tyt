# goxl-voxcore

Converts between Goxel `.gox` files and the voxcore state. The `goxl` crate
defines the file model. This crate carries a parsed file into voxcore's
in-memory `VoxMain` and back.

## File conversion

- `from_goxl_file` / `to_goxl_file`: between a parsed `GoxlFile` and a bare
  `VoxMain<()>`. The shared `BL16` voxel blocks become objects sharing one
  `baseColor` palette. The `LAYR` layers become the hierarchy nodes that
  place them. The writer synthesizes the file from the scene. Each object
  placement becomes one layer at its world translation. Grouping, rotation,
  and scale drop.

## Bytes conversion

The `codec` module, behind the default `codec` feature, goes straight to and
from `.gox` bytes over `goxl-codec`:

- `codec::from_goxl_bytes`: `.gox` bytes into a bare `VoxMain<()>`.
- `codec::to_goxl_bytes`: a state to `.gox` bytes.

Each takes the codec's dependencies: `DecodePng` to load and `EncodePng` to
write. `goxl_codec::DependenciesImpl` supplies both. This crate's `impl`
feature turns it on.

## The ext

`GoxlExt` holds the Goxel state with no native voxcore home. The bare
converters drop it on load and synthesize the file on write. The `ext`
feature, on by default, opens the `ext` module, where the typed path keeps
the ext:

- `ext::from_goxl_file_with_ext` loads a file into a `GoxlVoxMain`, a
  `VoxMain<Option<GoxlExt>>` carrying the ext. `ext::to_goxl_file_with_ext`
  writes the state back exactly.
- `codec::from_goxl_bytes_with_ext` and `codec::to_goxl_bytes_with_ext` do
  the same for `.gox` bytes.

The ext follows the state's listings through voxcore's `VoxExt` hooks. After
a node or object is released or reordered, the file still writes back with
the surviving provenance. A node retained after the load is written as a
synthesized layer. The ext enters a document's `ext` block as the `goxl`
entry through voxcore's `VoxExtEntryCodec`. A Voxel Json document carries the
ext in that block.
