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
image, preview, materials, cameras, light, unknown chunks, and one layer
entry per hierarchy node, keyed by the node's id, each with its stamp
placements. The ext stores only what the scene cannot derive. The writer
takes a layer's name from its node.

The ext follows the state through voxcore's `VoxExt` hooks, which see the
`VoxState`. A node retained after the load gets a complete entry on the spot,
stamping its objects at its translation, the entry the synthesizer would
have built. A released node drops its entry. A released object leaves every
entry's placements. A gc rekeys the entries. Releasing a layer that another
layer clones is refused, because the clone's `base-id` would dangle. Release
the clones first. voxconv carries the ext through a Voxel Json document's
`ext` block.

The `serde` feature, on by default, derives serde for the ext types. Ids
serialize as bare numbers.
