# gltf-meshdoc

Converts between glTF 2.0 files and the meshdoc state. The crates.io `gltf`
crate defines the file model. This crate carries a parsed file into
meshdoc's in-memory `MeshMain` and back.

## File conversion

The state is a `GltfMeshMain`, a `MeshMain<GltfExt>` carrying the glTF state
with no native meshdoc home as its ext. A `GltfFile` is a parsed document as
its parts: the JSON root, the binary buffer, and the loose files the root
references by relative URI.

- `from_gltf_file` / `to_gltf_file`: between a `GltfFile` and a
  `GltfMeshMain`. Images, textures, materials, meshes, and nodes become
  meshdoc's images, textures, materials, objects, and hierarchy nodes.
  Every scene's nodes and every parentless node become the roots. The
  loose files the document references become meshdoc's files. An image at
  a relative URI reads from its file. The rest of the file goes to the
  ext. A loaded file writes back exactly through its ext. `GltfWriteOptions`
  picks where the images go: as loaded, embedded, or loose beside the
  document.
- `to_gltf_mesh_main`: a bare `MeshMain<()>` to a `GltfMeshMain` with a
  synthesized ext, one default scene listing the roots and a default entry
  per entity. glTF holds everything meshdoc does, so the document is not
  reshaped. `take_ext` on the state takes the ext back off.

Axes convert between glTF's Y-up and meshdoc's Z-up on both arrows.
Positions, normals, tangents, and node transforms rotate; nothing bakes into
positions. Units are meters on both sides.

## The `vxl` extras

What meshdoc models and glTF does not rides under the `vxl` key of an
entity's `extras`. A material's and an object's properties write to
`extras.vxl.values`, one entry per property: a number, a string, a bool, a
list of those, a list of rows, a texture info `{"index", "texCoord"}`, or a
file `{"uri"}` referencing one of the document's files. A primitive's name
writes to `extras.vxl.name`. The reader takes these back out. Whatever else
the `extras` hold stays in the ext. glTF keys these by name and has no file
listing, so files, properties, and further vertex attributes load in name
order.

A further vertex attribute keeps its scalar type: floats write a `FLOAT`
accessor, and 8- and 16-bit unsigned integers write `UNSIGNED_BYTE` and
`UNSIGNED_SHORT` accessors. On read, a normalized integer accessor and the
signed and 32-bit integer types read as floats.

## Bytes conversion

The `codec` module, behind the default `codec` feature, goes straight to and
from bytes:

- `codec::from_gltf_bytes`: `.gltf` or `.glb` bytes into a `GltfMeshMain`.
  The container is detected from the leading bytes. The caller passes the
  loose files it read beside the primary.
- `codec::gltf_loose_uris`: the relative URIs a primary references, so a
  caller knows which files to read beside it.
- `codec::to_gltf_bytes`: a `GltfMeshMain` to `.gltf` bytes. The geometry
  buffer becomes a data URI or the loose file it was loaded from. The images
  go where the options say.
- `codec::to_glb_bytes`: a `GltfMeshMain` to `.glb` bytes with the geometry
  buffer as the `BIN` chunk. The crate frames the container itself.

Each takes the dependencies for data URIs: `DecodeBase64` to load and
`EncodeBase64` to write. `DependenciesImpl` supplies both over `base64`
behind the `impl` feature. The crate carries no image library; images stay
encoded.

## The ext

`GltfExt` holds the glTF state with no native meshdoc home: the asset block,
the declared extensions, the scenes and the default scene, the cameras, the
skins, the animations, and one entry per node, mesh, primitive, material,
texture, and image, keyed by the entity's id, each with its `extras`, its
unmodeled extensions, and what meshdoc lacks: a node's camera, skin, and
weights; a primitive's skinning streams and morph targets; a texture's name
and sampler identity; whether an embedded image came from a buffer view or
a data URI. Skinning and animation data stay in glTF's Y-up axes.

The ext follows the state through meshdoc's `MeshExt` hooks, which see the
`MeshState`. An entity retained after the load gets the entry the
synthesizer would have built. A released node leaves every scene, skin, and
animation that referenced it. A gc rekeys the entries. A root in no scene
joins the default scene on write, so a node retained after the load is
reachable.

The `serde` feature, on by default, derives serde for the ext types. Ids
serialize as bare numbers.

## Not carried

Primitives of points or lines have no triangles and error. Vertex color sets
beyond `COLOR_0` error. Sparse accessors are not read. Texture-info and
material-slot `extras` and extensions, buffer and accessor names, and
animation samplers no channel plays are dropped. The listing order of
files, properties, and further vertex attributes is not kept.
