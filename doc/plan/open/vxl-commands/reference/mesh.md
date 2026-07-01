# `vxl mesh`

_Part of the [Vxl Command-Line Reference](../README.md)._

```
vxl mesh <input> [output] [options]
```

Triangulates voxels into a mesh and optionally writes the voxels' material into
the mesh, either as textures the mesh's UVs sample or as per-vertex attributes.
The default output path is the input stem with the mesh extension; the mesh
format is inferred from the output extension or set with `--to`.

`mesh` outputs one object as pure geometry: the object's voxel grid is meshed on
its own, with no hierarchy-node transform applied, since the common case is
pulling a leaf object out without placement. Pass `--select` or `--select-index`
to choose which object; see [Object selectors](conventions.md#object-selectors).
The selectors may resolve to several objects, but how to output more than one is
not settled, so for now `mesh` errors unless the selection is exactly one object,
including a multi-object document meshed with no selector. Assembling a placed
scene from the hierarchy, baking the node transforms and instancing in
[Hierarchy Nodes](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#hierarchy-nodes),
is a separate mode left for a later pass.

1. `--to` `gltf` | `glb`: target mesh format, glTF text (`.gltf`) or binary
   (`.glb`); glTF is the only mesh format written for now. Inferred from the
   output extension when omitted.
2. `--from <format>`: source voxel format. Inferred from the input extension
   when omitted.
3. `--meters-per-voxel <meters>` (default `1.0`): the real-world edge length of
   one voxel in meters, applied as a uniform scale to every output vertex. The
   voxel-json format is unitless, with one unit per voxel, so `--meters-per-voxel`
   is where that grid gains a physical size: `1.0` sizes a voxel at one meter,
   `0.01` at one
   centimeter, and `0.001` at one millimeter. glTF is meter-native, so the mesh
   writes `<meters>` per voxel and opens at that real size. Scale affects vertex
   positions only and leaves UVs, normals, and vertex colors unchanged.
4. `--method` `greedy` | `culled` | `naive` (default `greedy`): meshing
   strategy. `greedy` merges coplanar, same-material faces into the fewest
   quads and has the lowest triangle count. `culled` emits one quad per
   solid-empty boundary face without merging. `naive` emits all six faces of
   every solid voxel, including hidden interior faces, and has the highest
   triangle count. Choose `culled` or `naive` only when you need stable
   per-voxel topology for further per-face editing.
5. `--computed-occlusion-strength <0..1>` (default `1.0`): scales how much
   computed occlusion darkens, from `0` for none to `1` for the full effect.
   Applies to computed occlusion wherever it is written, the
   `--vertex computed-occlusion` attribute or the `computed-occlusion` map.
6. `--computed-occlusion-min-brightness <0..1>` (default `0.0`): floor on the
   brightness the deepest occlusion reaches, so crevices never darken below it.
   `0` lets occlusion reach black.
7. `--computed-occlusion-color-space` `linear` | `srgb` (default `linear`): the
   space the occlusion values are written in. `linear` is correct for glTF and
   other PBR data textures; `srgb` matches pipelines that multiply occlusion in
   sRGB space.
8. `--atlas` `palette` | `unwrap` (default `palette`): material-map atlas layout;
   see [Material and texture maps](#material-and-texture-maps).
9. `--select <glob>`: choose the object by hierarchy path, matched as
   `hierarchy show` matches node paths, so a node path selects its subtree.
   Repeatable; the result is the union of every `--select` and `--select-index`
   value. See [Object selectors](conventions.md#object-selectors). The selection
   must resolve to one object, as above.
10. `--select-index <index>`: choose the object by position, an integer or an
   `a-b` range. Repeatable; unions with `--select` as above. See
   [Object selectors](conventions.md#object-selectors).

## Material and texture maps

A material map is one image whose channels are filled from a material's
attributes, so maps that read attributes share one atlas and differ only in
which attributes they read. Attributes a cell omits fall back to their spec defaults from
[Attributes](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#attributes),
so a map never fails for a missing attribute. Wherever an attribute is read,
`smoothness` names the derived `1 - roughness`, so `smoothness` and `1-roughness`
are interchangeable, as are `1-smoothness` and `roughness`. `--atlas` sets how
the atlas is laid out and how the mesh's UVs index it:

1. `palette` (default): one texel per palette material, placed at its palette
   index. The atlas depends only on the palette set, not the geometry, so every
   mesh on those palettes gets a byte-identical texture and identical UVs, and
   meshes that share a palette share its maps. Many faces sample one texel. This
   is the compact, shareable form. An object that references several palette
   layers has its material in the merge of one cell per layer, so the atlas keys
   on the merged material: one texel per combination of layer cells, the product
   of the layer sizes, which stays a pure function of the palette set and so
   stays shareable. The product grows with the layers, so a many-layer object is
   better served by `unwrap` or by `--vertex palette-layers`; a single-layer
   object is just one texel per cell.
2. `unwrap`: each face takes its own texel from a per-mesh UV unwrap, so the
   atlas is unique to one mesh and larger. Use it for spatially varying data
   that one texel per material cannot hold, such as `computed-occlusion`
   baked from the voxel geometry.

`computed-occlusion` is occlusion computed from the voxel geometry, the same
quantity `--vertex computed-occlusion` writes to vertex colors (see
[Vertex attribute maps](#vertex-attribute-maps)). It
varies across a surface, so it cannot occupy a palette texel and always bakes
into an unwrap layout for its own image. Under `--atlas palette` the shared
palette maps keep the mesh's primary UV set and the occlusion image takes a
second, unwrap UV set, so one mesh carries both the compact shared material
maps and a per-mesh occlusion bake. Under `--atlas unwrap` every map already
shares the one unwrap set and occlusion is another image on it. The second UV
set needs a mesh format that stores more than one, which glTF does, so a glTF
target can pair palette material maps with `computed-occlusion`.

`--texture-storage` `embedded` | `external` | `both` chooses where the images
go. `external` writes each map as a separate `.png` beside the mesh, the
`{stem}-mse.png` paths below; `embedded` packs them into the mesh itself, a
`.glb` binary chunk or a base64 data URI in a `.gltf`; and `both` embeds the
copy the mesh references and also writes the loose `.png` files. The default
follows the target, `embedded` for a `.glb` and `external` for a `.gltf`,
matching how each form carries its other resources. Under `both` the mesh reads
the embedded copy, so the loose files are working copies to edit and re-bake
from, which is why a texture-heavy mesh is easier to iterate on as `external`.

Maps come from two flags, each repeatable once per output image: `--texture` for
the named presets and `--texture-map` for a custom packing.

`--texture <name> [path]` writes a preset map, repeatable, as in
`--texture albedo --texture orm`. The optional `path` overrides the default of
the mesh stem plus the name, the `{stem}-mse.png` style. The names are:

1. `albedo`: RGBA base color from `rgba`. Four channels.
2. `orm`: glTF occlusion-roughness-metallic packing, R = `occlusion`,
   G = `roughness`, B = `metallic`. Three channels.
3. `metallic-roughness`: glTF metallic-roughness packing, G = `roughness`,
   B = `metallic`, R = `0`. Three channels.
4. `metallic-smoothness`: Unity metallic-smoothness packing, R = `metallic`,
   A = `smoothness`, G and B = `0`. Four channels.
5. `mse`: the custom MSE packing, R = `metallic`, G = `smoothness`,
   B = `emissive`. Three channels. This is the voxel-native form of the MSE
   texture the material tooling builds from image maps.
6. `emissive`: grayscale `emissive` strength. One channel.
7. `occlusion`: grayscale `occlusion`. One channel.
8. `computed-occlusion`: grayscale occlusion computed from the voxel geometry
   rather than read from the `occlusion` attribute. One channel. Always bakes
   into an unwrap layout; see the atlas notes above.
9. `roughness`: grayscale `roughness`. One channel.
10. `smoothness`: grayscale `smoothness`. One channel.

`--texture-map <path> <channels>` writes a custom packing, also repeatable. The
`channels` argument is a comma-separated list of `R=<expr>`, `G=<expr>`,
`B=<expr>`, and optional `A=<expr>`, where `<expr>` is an attribute name,
`1-<attribute>` for an inverted attribute, one color component as
`<attribute>.r`, `.g`, `.b`, or `.a`, the constant `0` or `1`, or
`computed-occlusion` for the geometry-derived occlusion. The channel
count is the number of channels named; an omitted channel is `0`. For example
`--texture-map model-mse.png R=metallic,G=smoothness,B=emissive` reproduces
`--texture mse`, and swapping `G=roughness` writes roughness instead of
smoothness. A packing that names `computed-occlusion` always bakes into an
unwrap layout, as `--texture-map ao.png R=computed-occlusion` does.

A color attribute is read one component at a time. `rgba` is a color, so
`R=rgba.a` writes its straight alpha and `R=rgba.r,G=rgba.g,B=rgba.b,A=rgba.a`
splits the color across four channels; the components are the stored sRGB
values with straight alpha. Naming a color with no component, as in `R=rgba`,
is an error, and `--texture albedo` is the way to write the whole color. A
scalar attribute names no component, so `metallic.r` is an error.

`--define-attribute <name> <key> [type=scalar]` names a custom
attribute so `--texture-map` and `--vertex-map` can read it, repeatable, as in
`--define-attribute sss subsurface` and `--define-attribute tint tint color`.
The voxel-json format stores attributes generically, so a palette may
carry keys beyond the recommended set in
[Attributes](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#attributes),
and a binding gives one such key a name a packing can use. A binding reads the
key's merged value across the object's palette layers, the same merge the rest
of `mesh` walks, so a key several layers set resolves to the last layer that
sets it. Its parts are:

1. `name`: the name used in `--texture-map` and `--vertex-map`. It shadows a
   built-in attribute name on collision, so `--define-attribute roughness
   micro-rough` makes `R=roughness` read the merged `micro-rough` instead. The
   shadowing is scoped to the custom packings; the `--texture` and `--vertex`
   presets always read the spec attributes.
2. `key`: the voxel-json attribute key read from the merged material.
3. `type` (default `scalar`): `scalar` for a `0..1` number read as a bare
   `<name>`, or `color` for a `#RRGGBBAA` hex whose `r`, `g`, `b`, and `a`
   components a packing reads as `<name>.r` and so on.

For example, `--define-attribute tint tint color` then `--texture-map
paint.png R=tint.r,G=tint.g,B=tint.b,A=rgba.a` packs the custom `tint` color
into RGB and the base color's alpha into `A`.

## Vertex attribute maps

A material map reads a material's attributes the same way whether it lands in a
texture or on the mesh's vertices; only the carrier differs. The
[texture maps](#material-and-texture-maps) above write through `--texture` and
`--texture-map`, sampled by the mesh's UVs; `--vertex` and `--vertex-map` write
the same resolved values to glTF vertex attributes, one value per vertex, with no
texture and no UV set. The source grammar is shared: `--vertex-map` reads the
same `<channels>` expressions as `--texture-map`, and `--define-attribute` names
custom attributes for both. Vertex attributes need geometry, so unlike texture
maps they have no `material`-command equivalent.

The attribute name a value lands in decides whether a generic glTF viewer reads
it. glTF's metallic-roughness shading reads one per-vertex slot, `COLOR_0`, a
vec4 that multiplies the base color, so per-vertex color is portable but
per-vertex metallic, roughness, and the rest are not: they go in
application-specific attributes, whose names glTF requires to start with `_` and
that only a custom shader reads. A generic viewer stores and ignores them.

A vertex carries one value per face corner. Greedy meshing merges only coplanar,
same-material faces, so every corner of a merged quad shares one material and
per-vertex material stays constant across the quad with no extra splitting;
`culled` and `naive` already carry one value per face corner. `computed-occlusion`
is the exception: it takes each corner's value from the three voxels meeting
there, four discrete levels, so it varies within a quad and forces per-face
resolution. With `--method greedy` two faces merge only when their occlusion
matches along the shared edge, so flat runs still merge into large quads while
concave seams split, and when a quad's four corners are uneven the triangle
diagonal is chosen to keep the darker pair together so interpolation does not
seam.

`--vertex <name> [target]` writes a preset to vertices, repeatable, the vertex
twin of `--texture`. The optional `target` overrides the default attribute name.
The names reuse the texture presets and pack the same way:

1. `albedo`: RGBA base color from `rgba` into `COLOR_0`. Portable; a glTF viewer
   renders it as the per-vertex base color with no texture.
2. `computed-occlusion`: occlusion computed from the voxel geometry, multiplied
   into `COLOR_0` to darken the base color, tuned by the `--computed-occlusion-*`
   options above. Override the target, as `--vertex computed-occlusion _AO`, to
   write it as a standalone custom scalar instead of darkening the color.
3. `metallic`, `roughness`, `emissive`, `occlusion`, `smoothness`: one scalar
   into `_METALLIC`, `_ROUGHNESS`, and so on, the name a `_` plus the preset
   uppercased. Custom attributes a custom shader reads.
4. `orm`, `mse`, `metallic-roughness`, `metallic-smoothness`: the packed presets,
   into `_ORM`, `_MSE`, and so on, packed across the attribute's components
   exactly as the texture preset packs them across channels. Custom attributes.
5. `palette-index`: one index per vertex into a flattened table of the distinct
   merged materials this mesh uses, written as a scalar into `_PALETTEINDEX` with
   that table shipped as [palette data](#palette-data). A custom shader looks the
   material up by index rather than sampling a texture: the most compact carrier
   and the exact-value alternative to the palette atlas. The table is the used
   combinations, so it is per-mesh, not shared across meshes. Custom; not read by
   a generic viewer.
6. `palette-layers`: one index per referenced palette layer per vertex, written
   as scalars `_PALETTEINDEX0`, `_PALETTEINDEX1`, and so on, with each layer's
   palette shipped verbatim as [palette data](#palette-data). A custom shader merges the
   indexed cells the way the spec defines, later layers winning. Its data sums
   the layer sizes rather than multiplying them and depends only on the palette
   set, so it stays shareable across meshes and is the compact carrier for a
   many-layer object; it is also the only carrier that keeps a layer value the
   merge would otherwise drop. A single-layer object reduces to one
   `_PALETTEINDEX0` and the one palette. Custom; not read by a generic viewer.

`--vertex-map <target> <channels>` writes a custom packing to a named attribute,
repeatable, the vertex twin of `--texture-map`. `target` is `COLOR_0` or a custom
`_NAME`, and `channels` is the same comma-separated `R=<expr>,G=<expr>,...` list,
so `--vertex-map _ORM R=occlusion,G=roughness,B=metallic` packs ORM into a vec3
attribute and `--vertex-map COLOR_0 R=rgba.r,G=rgba.g,B=rgba.b,A=rgba.a` writes
the base color. The component count follows the channels named: one is a scalar,
two a vec2, three a vec3, four a vec4. A packing that names `computed-occlusion`
resolves it per corner as above.

The two carriers compose in one run: write base color to `COLOR_0` and bake PBR
into a shared palette atlas, or carry everything on vertices for a texture-free
mesh. `--atlas` sets only the texture layout and never affects vertex attributes.

## Palette data

`--vertex palette-index` and `--vertex palette-layers` write only an index per
vertex; the table that turns an index into a material is a small block of JSON.
`--palette-storage` `embedded` | `external` | `both` chooses where it goes:

1. `embedded`: in the glTF document's `extras`, under a `vxl` key, so the table
   travels inside the `.gltf` or `.glb` and a custom loader reads `extras.vxl`.
2. `external`: in a sidecar JSON file beside the mesh, the mesh stem plus
   `-palette.json`, so `model.gltf` pairs with `model-palette.json`. Easier to
   read and edit on its own.
3. `both`: write the `extras` block and the sidecar file; the embedded copy is
   the one a loader should trust, the file a loose working copy.

The default follows the target, `embedded` for a `.glb` and `external` for a
`.gltf`, the same split as `--texture-storage`. Either way a generic glTF viewer
ignores the data, since it is not a glTF material; these carriers need a custom
loader. The sidecar file's top-level object is exactly the value of `extras.vxl`,
so the two forms carry byte-identical content in different places. Examples below
are JSONC for readability; a real file is plain JSON.

The top-level `kind` says which carrier wrote it. A material is an object of
voxel-json
[attribute](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#attributes)
keys to values: a color is a `#RRGGBBAA` hex string, a scalar is a number, and an
omitted attribute takes its spec default, so a material lists only what it sets.
An unrecognized `version` is rejected.

### `kind: "palette-index"`

`_PALETTEINDEX` on each vertex indexes `materials`, one entry per distinct merged
material the mesh uses:

```jsonc
{
  "version": 1,
  "kind": "palette-index",
  "materials": [
    { "rgba": "#FF0000FF", "roughness": 0.9 }, // index 0
    { "rgba": "#FF0000FF", "roughness": 0.1 }, // index 1
    { "rgba": "#0000FFFF", "roughness": 0.1 }  // index 2
  ]
}
```

### `kind: "palette-layers"`

`_PALETTEINDEX0`, `_PALETTEINDEX1`, and so on index `layers[0]`, `layers[1]`, and
so on in the object's `paletteRefs` order. A custom shader merges the indexed
cells, a later layer overriding an earlier one on a shared key, the same merge
the voxel-json format defines:

```jsonc
{
  "version": 1,
  "kind": "palette-layers",
  "layers": [
    [ // layer 0 (base), indexed by _PALETTEINDEX0
      { "rgba": "#FF0000FF", "tint": "#880000FF" },
      { "rgba": "#0000FFFF", "tint": "#000088FF" }
    ],
    [ // layer 1 (finish), indexed by _PALETTEINDEX1
      { "roughness": 0.9, "tint": "#FFFFFFFF" },
      { "roughness": 0.1, "tint": "#FFFF00FF" }
    ]
  ]
}
```

For the three voxels A = (base 0, finish 0), B = (base 0, finish 1), and
C = (base 1, finish 1), `_PALETTEINDEX0` is `0, 0, 1` and `_PALETTEINDEX1` is
`0, 1, 1`.

### TypeScript Schema

```typescript
// The palette data the palette-index and palette-layers vertex carriers write,
// either in glTF `extras.vxl` or in a `<stem>-palette.json` sidecar; the two
// carry identical content.
type PaletteData = PaletteIndexData | PaletteLayersData;

interface PaletteIndexData {
  version: 1;
  kind: "palette-index";
  // _PALETTEINDEX on each vertex indexes this array; one entry per distinct
  // merged material the mesh uses.
  materials: Material[];
}

interface PaletteLayersData {
  version: 1;
  kind: "palette-layers";
  // _PALETTEINDEX0, _PALETTEINDEX1, ... index layers[0], layers[1], ... in the
  // object's paletteRefs order; a later layer overrides an earlier one on a
  // shared key.
  layers: Material[][];
}

// Attribute keys to values, following the voxel-json Attributes vocabulary: a
// color is a `#RRGGBBAA` hex string, a scalar is a number. An omitted attribute
// takes its spec default, so a material lists only what it sets.
type Material = { [attribute: string]: string | number };
```

## Future work

A later pass may add `--computed-occlusion-radius` and
`--computed-occlusion-falloff` for a sampled neighborhood model that gathers
occluders out to a distance and weights them by a falloff curve, giving smoother
and wider gradients. They do not apply to the current discrete corner method,
which has a fixed one-voxel reach, so they are left out until that model lands.
