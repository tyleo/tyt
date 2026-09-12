# goxl-voxcore

Converts between Goxel `.gox` files and the voxcore state. The `goxl` crate
defines the file model. This crate carries a parsed file into voxcore's
in-memory `VoxMain` and back.

## File conversion

The state is a `GoxlVoxMain`, a `VoxMain<GoxlExt>` carrying the Goxel state
with no native voxcore home as its ext.

- `from_goxl_file` / `to_goxl_file`: between a parsed `GoxlFile` and a
  `GoxlVoxMain`. The shared `BL16` voxel blocks become objects sharing one
  `baseColor` palette. The `LAYR` layers become root hierarchy nodes with no
  transform, each placing the blocks it stamps. The stamp positions ride in
  the ext. A loaded file writes back exactly through its ext. The writer
  errors on an object larger than a block.
- `to_goxl_vox_main`: a bare `VoxMain<()>` to a `GoxlVoxMain` with a
  synthesized ext, which writes as a file synthesized from the scene. The
  scene takes the shape a loaded file has. Each object placement becomes one
  root node. The node holds the placement's tiles: `16 x 16 x 16` objects
  cut from the object on the world grid at its world translation. Its layer
  entry stamps the tiles there. The source nodes and objects are released.
  Grouping, rotation, and scale drop. `take_ext` on the state takes the ext
  back off.

## Bytes conversion

The `codec` module, behind the default `codec` feature, goes straight to and
from `.gox` bytes over `goxl-codec`:

- `codec::from_goxl_bytes`: `.gox` bytes into a `GoxlVoxMain`.
- `codec::to_goxl_bytes`: a `GoxlVoxMain` to `.gox` bytes.

Each takes the codec's dependencies: `DecodePng` to load and `EncodePng` to
write. `goxl_codec::DependenciesImpl` supplies both. This crate's `impl`
feature turns it on.

## The ext

`GoxlExt` holds the Goxel state with no native voxcore home: the header,
image, preview, materials, cameras, light, unknown chunks, and per-layer
provenance aligned by index with the hierarchy nodes, each with its stamp
placements. The ext follows the state's listings through voxcore's `VoxExt`
hooks. After a node or object is released or reordered, the file still
writes back with the surviving provenance. A node retained after the load is
written as a synthesized layer. voxconv carries the ext through a Voxel Json
document's `ext` block.

The `serde` feature, on by default, derives serde for the ext types.
