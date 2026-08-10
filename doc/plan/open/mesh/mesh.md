# `vxl mesh`

_Part of the [mesh plan](README.md)._

```
vxl mesh <input> [output] [options]
```

`vxl mesh` triangulates one object's voxels into a glTF mesh. It bakes the
object's palette materials into values. The values ride along as textures,
material fields, and files beside the mesh. The default output path is the input
stem with the mesh extension. The format comes from `--to`, else the output
extension, else `.glb`.

```
vxl mesh turret.voxj                         # turret.glb, geometry only
vxl mesh turret.voxj --output-profile pbr    # + embedded albedo, orm, and emissive maps
```

`mesh` writes one object as pure geometry. No hierarchy-node transform applies.
The common case is pulling a leaf object out without placement. `--select` and
`--select-index` choose the object. The selection must resolve to exactly one
object, so a multi-object document needs a selector. See
[Object selectors](../vxl-commands/reference/conventions.md#object-selectors).

## Options

1. `--to` `gltf` | `glb`: the target mesh format, glTF text (`.gltf`) or binary
   (`.glb`). When omitted, the format comes from the output extension, else
   `.glb`.
2. `--from <format>`: the source voxel format. When omitted, the format comes
   from the input extension.
3. `--voxel-size <meters>` (default `1.0`): the real-world edge length of one
   voxel in meters. The voxel grid is unitless and glTF is meter-native, so this
   flag gives a voxel its physical size. `1.0` opens at one meter per voxel, and
   `0.01` opens at one centimeter. The size applies as a uniform scale to vertex
   positions only.
4. `--method` `greedy` | `culled` | `naive` (default `greedy`): the meshing
   strategy. `greedy` merges coplanar, same-material faces into the fewest
   quads. `culled` emits one unmerged quad per solid-empty boundary face.
   `naive` emits all six faces of every solid voxel, hidden interior faces
   included. Choose `culled` or `naive` only when you need stable per-voxel
   topology.
5. `--texture-shape` `fit` | `line` | `pot` | `square` | `<n>` (default `pot`):
   the atlas canvas, counted in cells. A cell is one texel, and in a
   [corner texture](#the-corner-atlas) a 2x2 texel block. `fit` is the
   near-square packing. `line` is a single row of cells. `pot` is the smallest
   square power of two. `square` is the smallest square. `<n>` is an exact
   `n`x`n` canvas of cells, and a canvas too small errors. Unused cells are
   transparent black, and the mesh never samples them.
6. `--material-count <count>` (default `0`): how many materials the mesh
   carries, numbered from `0`. Every material flag names one material by index.
   An index at or above the count errors rather than growing the count, so a run
   that writes a material declares it first; see
   [Primitives and materials](#primitives-and-materials).
7. `--material-name <material-index> <name>`: names a material. The name lands
   as the glTF `material.name`. Optional; a material without the flag carries no
   name.
8. `--primitive <material-index | none> <src-expr>`: declares a primitive. The
   first argument is the material the primitive draws with, and `none` is no
   material at all. The expression is the select that routes the primitive's
   faces. Primitives number from `0` in flag order. The select is a
   [bool](value-language.md#booleans) read at the
   [face domain](value-language.md#domains), lower domains climbing in, and the
   primitive takes every face whose entry is true. The selects partition the
   faces: a face no select takes errors, and a face two selects take errors too.
   Without the flag the mesh has one primitive, index 0, holding every face and
   addressable like any other. That implicit primitive draws with material 0
   when the mesh carries materials, and with no material when it carries none.
   The first `--primitive` replaces it, so the declared primitives are exactly
   the mesh's; see [Primitives and materials](#primitives-and-materials).
9. `--primitive-name <primitive-index> <name>`: names a primitive. glTF
   primitives carry no name field, so the name rides the primitive's `extras`.
   Optional.
10. `--uv` `row` | `face` | `corner`: declares the mesh's UV streams in
    `TEXCOORD` order, one stream per flag. The list is the whole contract: each
    texture bakes at the lowest listed domain at or above its value's domain, so
    `--uv face` alone bakes row maps per face. A texture whose domain sits above
    every listed entry errors, since stepping down is never implicit. When
    omitted, the list derives from use, each texture at its value's own domain.
    An unused entry is legal, and its stream emits for an outside sampler. See
    [UV streams](#uv-streams).
11. `--compute-occlusion <dst-name>`: computes occlusion from the voxel geometry
    and binds it under the name. The value is a per-corner vec1 in `0..1`, one
    entry per face corner, and `1` is fully open. Every expression can read it
    the way it reads a palette property. Without the flag nothing computes and
    the name does not exist; see
    [Computed occlusion](value-language.md#computed-occlusion).
12. `--value <dst-name> <src-expr>`: defines a value the writers and slots can
    name. Every property of the effective palette is a name. Values evaluate in
    flag order, and redefinition is let-style: a later `--value` overrides an
    earlier one, and later expressions see the new value. The expression grammar
    is the [value language](value-language.md).
13. `--write-file-json-value <dst-file> <dst-name> <src-expr> <linear | srgb>`:
    writes a value to a JSON file under the name as its key. The token names the
    transfer the written numbers take. A bool value writes `true`/`false` under
    `linear` alone. Repeatable, and repeats on one path merge, so the file is
    always an object; see [JSON files](value-language.md#json-files).
14. `--write-file-png-value <dst-file> <src-expr> <linear | srgb>`: writes a
    [row, face, or corner](value-language.md#domains) array to an 8-bit PNG
    beside the mesh, one texel per entry, a corner array in the
    [corner atlas](#the-corner-atlas)'s block layout. The image is sized to its
    value: vec1 through vec4 write grey, grey-alpha, RGB, and RGBA, and
    components map to channels by position. Grey-alpha is PNG's only two-channel
    form, so a vec2's second component lands in the alpha channel. Pad with
    `rgb(u, v, 0)` where a viewer should read opaque color. The token names the
    encoding. `srgb` applies the sRGB transfer, for an image a viewer reads as
    color. `linear` applies none, for the data channels glTF wants linear. A
    component outside `0..1` errors. The file also declares its transfer in its
    own chunks; see the [notes](value-language.md#notes).
15. `--write-material-extra-image-file <material-index> <dst-name> <src-file>`:
    sets a custom `extras.vxl.values.<name>` entry on the indexed material to an
    image reference. The entry holds a texture index, and the texture points at
    the named file by relative path; see
    [Material slots](value-language.md#material-slots).
16. `--write-material-extra-image-value <material-index> <dst-name> <src-expr> <linear | srgb>`:
    writes an array value as an embedded image. The custom
    `extras.vxl.values.<name>` entry on the indexed material holds its texture
    index. A plain value errors, because an image needs texels; see
    [Material slots](value-language.md#material-slots).
17. `--write-material-extra-json-file <material-index> <dst-name> <src-file>`:
    sets a custom `extras.vxl.values.<name>` entry on the indexed material to a
    `{"uri"}` pointer at the named JSON file by relative path; see
    [Material slots](value-language.md#material-slots).
18. `--write-material-extra-json-value <material-index> <dst-name> <src-expr> <linear | srgb>`:
    writes a value's numbers into a custom `extras.vxl.values.<name>` entry on
    the indexed material. A plain value writes as its numbers, and an array
    writes as rows; see [Material slots](value-language.md#material-slots).
19. `--write-material-slot-file <material-index> <dst-property> <src-file>`:
    sets a texture property of the indexed material to reference an existing
    file by relative path. The file can come from `--write-file-png-value` or
    from anywhere else; see [Material slots](value-language.md#material-slots).
20. `--write-material-slot-value <material-index> <dst-property> <src-expr>`:
    sets one property of the indexed material. A plain value becomes a material
    field. An array value embeds as an image, in the glb binary chunk or as a
    data URI in a `.gltf`; see
    [Material slots](value-language.md#material-slots).
21. `--write-mesh-extra-image-file <dst-name> <src-file>`: sets a mesh
    `extras.vxl.values.<name>` entry to an image reference. The entry holds a
    texture index, and the texture points at the named file by relative path;
    see [Palettes](#palettes).
22. `--write-mesh-extra-image-value <dst-name> <src-expr> <linear | srgb>`:
    writes an array value as an embedded image. The mesh
    `extras.vxl.values.<name>` entry holds its texture index. A plain value
    errors; see [Palettes](#palettes).
23. `--write-mesh-extra-json-file <dst-name> <src-file>`: sets a mesh
    `extras.vxl.values.<name>` entry to a `{"uri"}` pointer at the named JSON
    file by relative path; see [Palettes](#palettes).
24. `--write-mesh-extra-json-value <dst-name> <src-expr> <linear | srgb>`:
    writes a value's numbers into a mesh `extras.vxl.values.<name>` entry. A
    plain value writes as its numbers. An array writes as rows, one row per
    flattened material; see [Palettes](#palettes).
25. `--write-primitive-builtin-value <primitive-index> <dst-attribute> <src-expr>`:
    writes a value to an attribute glTF defines, `COLOR_0`, on the indexed
    primitive. The corners take the value by
    [domain](value-language.md#domains), and a corner value lands exactly. The
    defined vocabulary fixes the encoding, so the flag carries no token. An
    underscore name errors; the custom flag is its home; see
    [Vertex attributes](value-language.md#vertex-attributes).
26. `--write-primitive-custom-value <primitive-index> <dst-name> <src-expr> <linear | srgb>`:
    writes a value to a custom vertex attribute on the indexed primitive. The
    name is typed with the underscore glTF requires of application-specific
    attributes. `_MY_COLOR` lands exactly as written, and a bare name errors;
    see [Vertex attributes](value-language.md#vertex-attributes).
27. `--write-primitive-index <primitive-index> <dst-name> <u8 | u16>`: writes
    the per-vertex palette index as its own custom attribute on the indexed
    primitive. The name is underscore-typed, and the width is `u8` or `u16`:
    `u8` holds 256 rows, and `u16` holds 65536. A palette the width cannot index
    errors. The numbers are the effective palette's own rows under any select.
    The flag is independent of every other flag. Alone it is the bare index, and
    beside extras rows it is the join key a shader reads them by; see
    [Palettes](#palettes).
28. `--write-primitive-normal <primitive-index> <true | false>` (default
    `true`): whether the indexed primitive writes `NORMAL`, the mesher's
    computed normal, beside `POSITION`. glTF leaves the attribute optional, and
    a viewer derives flat normals from the triangles. A voxel face is flat, so a
    conforming viewer draws the same pixels either way. `false` drops the
    stream, bytes a data primitive never reads.
29. `--value-profile <profile>`: applies a profile of values as if each were a
    `--value` at the flag's own position. The built-ins ship in the binary, and
    the rest come from `.vxlconfig`. Repeatable; see the
    [profile language](profile-language.md).
30. `--output-profile <profile>`: applies an output profile, a run's whole
    surface: the geometry options, materials, primitives, files, and extras. The
    profile expands to the flags it spells, and its `values` list applies its
    value profiles first. At most once per run. An explicit flag replaces the
    element it collides with; see the [profile language](profile-language.md).
31. `--file-stem <file-stem>`: replaces `{file-stem}` in profile file templates.
    The default is the output mesh's own stem, after the output path resolves.
    An output of `turret.glb`, named or derived from the input, fills
    `{file-stem}-mse.png` as `turret-mse.png` with no flag at all. Renaming the
    mesh renames every templated file with it.
32. `--select <glob>`: chooses the object by hierarchy path, a node path
    selecting its subtree. Repeatable; unions with `--select-index`. See
    [Object selectors](../vxl-commands/reference/conventions.md#object-selectors).
33. `--select-index <index>`: chooses the object by position, an integer or an
    `a-b` range. Repeatable; unions with `--select`.

A writer's arguments read destination first, then source, then the token when
one exists. The order is an assignment: the location before what fills it,
`--value`'s own order with the encoding trailing. A material or primitive index
is part of the destination, so it rides ahead of the rest: which object, then
what on it. A writer's name ends in its source kind. `-value` takes an
expression and writes its value. `-file` takes an existing file. `-index` takes
the palette row number. `-normal` takes the mesher's computed normal; source and
destination are both fixed, so the flag carries only whether the write happens.
A `<src-expr>` is any expression of the [value language](value-language.md), a
defined name the simplest. A `<src-file>` names an existing file.

## Primitives and materials

A glTF mesh holds primitives. A primitive is one draw: its own vertex data, its
own triangle list, and at most one material. Two materials on one mesh means two
primitives, each holding the faces it draws. By default a run carries no
materials and one primitive, index 0, holding every face and addressable like
any other. `--material-count` sets how many materials exist. Each
`--primitive <material-index | none> <src-expr>` declares a primitive: the
material it draws with and the select that routes its faces. The first flag
replaces the implicit primitive. The implicit primitive draws with material 0
when the mesh carries materials, and with no material when it carries none.
Everything is 0-indexed. A flag naming an index at or above a count errors
rather than growing the count.

A primitive can carry no material. `none` in the material position declares one
beside any count, so a data primitive rides beside drawn ones. glTF leaves
`primitive.material` optional. A viewer draws such a primitive with the spec's
default material, every field at its default, the pixels an empty material
produces. `COLOR_0` multiplies into base color, so a mesh of vertex values alone
still shows its colors. At count `0` the glTF carries no `materials` array,
because the spec forbids an empty one. A declared material no primitive draws is
legal and emits unused.

The select routes the faces. It names a [bool](value-language.md#booleans) value
read at the [face domain](value-language.md#domains), lower domains climbing the
ladder, and the primitive takes every face whose entry is true. A row bool
routes whole rows: every face takes its row's answer. A face bool routes faces
one by one, reaching below the palette to what only the mesh knows. A plain bool
takes every face or none, and `true` is the whole-mesh select on any material.
The selects partition the faces. A face no select takes would be a silent drop,
so it errors. A face two selects take is two flags claiming one destination, the
error the rest of the flag surface already throws. The partition covers the
faces the mesher emits. A row-bool partition of the used rows holds under every
`--method`, while a face-bool gap can open under one method and not another. The
complement select is whole under all of them, and a `false` select takes
nothing, legal wherever the rest cover the mesh. Selects only route; they never
change what geometry exists.

The selects split the model. Every face draws once with its row's material:

```
# split: solid rows with material 0, glowing rows with material 1
vxl mesh turret.voxj
    --value glowing "emissiveStrength > 0"
    --value solid "!glowing"
    --material-count 2
    --primitive 0 solid
    --primitive 1 glowing
```

```jsonc
{
  "asset": { "version": "2.0" },
  "buffers": [ /* ... */ ],
  "bufferViews": [ /* ... */ ],
  "accessors": [ /* ... */ ],
  "materials": [ {}, {} ],
  "meshes": [ { "primitives": [
    { "attributes": { "POSITION": 0, "NORMAL": 1 }, "indices": 2, "material": 0 },
    { "attributes": { "POSITION": 3, "NORMAL": 4 }, "indices": 5, "material": 1 }
  ] } ],
  "nodes": [ { "mesh": 0 } ],
  "scenes": [ { "nodes": [ 0 ] } ],
  "scene": 0
}
```

A face select splits inside a row. Occlusion lives on the mesh, not the palette,
so a crevice mask sends one row's seam faces to their own material:

```
# dirt in the creases: material 1 takes the occluded faces
vxl mesh statue.vox
    --compute-occlusion computedOcclusion
    --value crevice "faceAvg(computedOcclusion) < 0.7"
    --value open "!crevice"
    --material-count 2
    --primitive 0 open
    --primitive 1 crevice
```

A styled catch-all is an explicit complement. The last primitive selects what
the others do not, `--value rest "!(metal || glass)"`, so the partition stays
whole and every face still names its material.

`--material-name` lands as the glTF `material.name`. glTF primitives carry no
name field, so `--primitive-name` rides the primitive's `extras`.

## The palette atlas

Every row map of one bake shares a single layout: one texel per distinct
flattened material. The object's layers merge per property name by the format's
layer-override resolution. Each property reads through the last layer whose
palette supplies its name. A voxel's texel is therefore keyed by the tuple of
materials it samples in those winning layers, deduplicated in first-seen raster
order. A single-layer object reduces to one texel per material its voxels use.
Each map fills the same layout from its own value, so

```
vxl mesh turret.voxj --to gltf --output-profile pbr
```

writes `turret.gltf` with embedded albedo, orm, and emissive maps. Every map is
the same size, with the same flattened material at the same texel. Every face's
UVs sit at its texel center, read with a nearest-neighbor sampler and clamped
wrapping, so a face samples exactly its texel. The atlas depends on the
materials the object uses, so it is per-mesh, not shared across meshes.

Nothing auto-defaults. A profile spells the glTF spec defaults through the
`defaults` mixin, and a hand-written `--value` reads `default()` for a property
no layer supplies; see the
[profile language](profile-language.md#built-in-profiles). Once maps bake,
greedy meshing merges only faces that share a flattened material, since a merged
quad samples one texel. Pure geometry merges on shape alone.

## The unwrap atlas

The unwrap atlas is a per-mesh UV unwrap with a texel per face, the layout of
every texture that bakes at the face domain. It serves values that vary across a
surface: the language's [face domain](value-language.md#domains).
[Computed occlusion](value-language.md#computed-occlusion), reduced from its
corners, is the first face value. The layout packs the face cells into the
canvas and generates the face stream's UVs. `--uv face` alone lays every texture
out per face, row values climbing in; see [UV streams](#uv-streams).

## The corner atlas

The corner atlas is a per-mesh unwrap with a 2x2 texel block per face, one texel
per corner, the layout of every texture that bakes at the corner domain. The
block's texels sit in the face's corner order, and the face's UVs sit at the
four texel centers, so bilinear interpolation between the centers blends only
the block's own texels and reproduces the per-corner gradient. No gutter padding
is needed, since the UVs never leave the block. A corner texture samples linear
without mipmaps where the other layouts sample nearest, so minification cannot
bleed across blocks. A merged greedy face still carries one block, so corner
occlusion disagreeing inside a span splits the quad; see
[computed occlusion](value-language.md#computed-occlusion).

The layout exists for corner values written whole, and computed occlusion is the
first:

```
vxl mesh statue.vox
    --compute-occlusion computedOcclusion
    --material-count 1
    --write-material-slot-value 0 occlusionTexture computedOcclusion
```

The standard `occlusionTexture` slot then shades smooth creases in a stock
viewer, no custom shader involved. The
[vertex attributes](value-language.md#vertex-attributes) stay the textureless
route for a shader of your own.

## UV streams

A sampled texture is texels plus the coordinates faces read them by, and the two
must agree. Each texture-capable domain therefore has its own arrangement. A row
texture holds one texel per palette row, and every face of the row reads the
same texel. A face texture holds one texel per face, each face its own. A corner
texture holds a 2x2 block per face, each corner its own texel. One mesh can
carry several kinds at once, one face then reading a different spot in each, so
the mesh carries one UV stream per layout in use, glTF's numbered `TEXCOORD_<n>`
attributes. A row or face texture samples nearest; a corner texture samples
linear, the interpolation its point.

The stream list derives from use when nothing spells it: each texture bakes at
its value's own domain, and the streams in use emit in ladder order, `[row]`
through `[row, face, corner]`, empty when nothing writes a texture.
`--uv <row | face | corner>`, repeatable, spells the list instead, one stream
per flag in `TEXCOORD` order, and a profile's `uvs` key is the same list in
config, any `--uv` replacing all of it. The spelled list is the whole contract:
each texture bakes at the lowest listed domain at or above its value's domain,
climbing in, so `--uv face` alone bakes the row maps per face, one stream for a
consumer that reads one UV set. A texture whose domain sits above every listed
entry errors, since stepping down is never implicit; `faceAvg` spells the step.
The order pins the numbers: an engine that wants its face maps at a fixed slot
spells `--uv face --uv row`. An entry no texture bakes at is legal and emits its
stream anyway, the coordinates an externally-baked texture, an engine lightmap,
samples by.

Every texture's `texCoord` then derives: the domain it bakes at looks up in the
list, and its position is the number. No flag hand-wires a slot:

```jsonc
// vxl mesh turret.voxj --output-profile albedo
//     --compute-occlusion computedOcclusion
//     --value ao "faceAvg(computedOcclusion)"
//     --write-material-slot-value 0 occlusionTexture ao
// row and face both write, so the streams derive [row, face]
{
  "asset": { "version": "2.0" },
  "buffers": [ /* ... */ ],
  "bufferViews": [ /* ... */ ],
  "accessors": [ /* ... */ ],
  "images": [ /* ... */ ],
  "samplers": [ /* ... */ ],
  "textures": [ { "sampler": 0, "source": 0 }, { "sampler": 0, "source": 1 } ],
  "materials": [ {
    "pbrMetallicRoughness": {
      "baseColorTexture": { "index": 0, "texCoord": 0 }   // the row stream
    },
    "occlusionTexture": { "index": 1, "texCoord": 1 }     // the face stream
  } ],
  "meshes": [ { "primitives": [ {
    "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2, "TEXCOORD_1": 3 },
    "indices": 4,
    "material": 0
  } ] } ],
  "nodes": [ { "mesh": 0 } ],
  "scenes": [ { "nodes": [ 0 ] } ],
  "scene": 0
}
```

## Palettes

The mesh extras flags serve a runtime that resolves materials itself: a game
that swaps team colors or ramps a glow per damage state without re-exporting the
mesh. The four `--write-mesh-extra-*` flags write named entries under the mesh's
`extras.vxl.values`, the same grid the material extras take. `json-value` puts a
value's numbers in the entry, a plain value as its numbers and an array as rows,
one per flattened material in [shape order](value-language.md#shapes).
`json-file` points the entry at an existing JSON file. `image-value` embeds an
array as a PNG and stores its texture index. `image-file` references an existing
image the same way. The entry shapes cannot be confused: numbers and rows are
themselves, an image is an `{"index"}`, and a file pointer is a `{"uri"}`. The
same name twice, in any two forms, is two flags claiming one destination, the
usual error.

The palette pattern is rows beside their join key.
`--write-mesh-extra-json-value` writes the rows. `--write-primitive-index`
writes the attribute they are read by, one integer per vertex naming the
flattened material its face samples. The attribute is yours to spell: the name,
typed with its underscore like any custom attribute, and the width, `u8` holding
256 rows and `u16` holding 65536. A palette the width cannot index errors rather
than truncates. Every array value runs over the one effective palette, so every
entry of rows shares the one index. A shader reads `values.albedo[_PALETTE]`
against as many entries as the line writes, and each is plain data the runtime
can replace at will:

```jsonc
// vxl mesh turret.voxj --output-profile pbr
//     --write-mesh-extra-json-value albedo albedo linear
//     --write-mesh-extra-json-value emissive emissive linear
//     --write-primitive-index 0 _PALETTE u8
{
  "asset": { "version": "2.0" },
  "extensionsUsed": [ "KHR_materials_emissive_strength" ],
  "buffers": [ /* ... */ ],
  "bufferViews": [ /* ... */ ],
  "accessors": [ /* ... */ ],
  "images": [ /* ... */ ],
  "samplers": [ /* ... */ ],
  "textures": [ /* ... */ ],
  "materials": [ /* ... */ ],
  "meshes": [ {
    "primitives": [ {
      "attributes": { "POSITION": 0, "NORMAL": 1, "_PALETTE": 2 },
      "indices": 3,
      "material": 0
    } ],
    "extras": { "vxl": { "values": {
      "albedo":   [ [1, 0, 0, 1], [1, 0, 0, 1], [0, 0, 1, 1] ],
      "emissive": [ [0, 0, 0],    [0, 0, 0],    [4, 3, 0]    ]
    } } }
  } ],
  "nodes": [ { "mesh": 0 } ],
  "scenes": [ { "nodes": [ 0 ] } ],
  "scene": 0
}
```

A vec1 value's rows are numbers, and a vecN value's rows are arrays of N, the
`--write-file-json-value` shapes. The token is that writer's token for the same
reason: the rows are numbers your own runtime reads, nothing fixes their
encoding, so the flag declares it.

The flags stay independent, so each half of the pattern stands alone.
`--write-primitive-index` by itself is the bare index, an attribute with no
rows, for a runtime that ships its own tables keyed to the effective palette's
row order. Rows by themselves are legal too, data a build step reads in material
order with no per-vertex join. Nothing checks the pairing, so a runtime that
needs both spells both. The index is a custom attribute like any other, so a
`--write-primitive-custom-value` spelling its name is two flags claiming one
destination.

A storage choice is a flag combination. Embedded rows are
`--write-mesh-extra-json-value`. A sidecar is `--write-file-json-value` with a
`--write-mesh-extra-json-file` pointer at it. Both at once are the rows beside
the json:

```jsonc
// vxl mesh turret.voxj --output-profile pbr
//     --write-file-json-value turret-values.json albedo albedo linear
//     --write-mesh-extra-json-file albedo turret-values.json
//     --write-primitive-index 0 _PALETTE u8
{
  "asset": { "version": "2.0" },
  "extensionsUsed": [ "KHR_materials_emissive_strength" ],
  "buffers": [ /* ... */ ],
  "bufferViews": [ /* ... */ ],
  "accessors": [ /* ... */ ],
  "images": [ /* ... */ ],
  "samplers": [ /* ... */ ],
  "textures": [ /* ... */ ],
  "materials": [ /* ... */ ],
  "meshes": [ {
    "primitives": [ {
      "attributes": { "POSITION": 0, "NORMAL": 1, "_PALETTE": 2 },
      "indices": 3,
      "material": 0
    } ],
    "extras": { "vxl": { "values": {
      "albedo": { "uri": "turret-values.json" }
    } } }
  } ],
  "nodes": [ { "mesh": 0 } ],
  "scenes": [ { "nodes": [ 0 ] } ],
  "scene": 0
}
```

A mesh entry contests nothing a slot writes.
`--write-material-slot-file 0 baseColorTexture skin.png` fills the material
while `--write-mesh-extra-json-value baseColor baseColorFactor linear` writes
the rows: two destinations serving two readers. A stock viewer renders the slots
and never reads the extras. A runtime that reads the extras draws its own
pixels. The mesh therefore carries the baked look and the swappable data side by
side.

Layers end at the flatten. A runtime grouping is authored data instead. An int
property on the palette entries groups materials, flattens like any property,
and writes like any value. The engine then swaps one compact palette of its own
keyed by the group:

```jsonc
// in the voxel file's palette: colorId groups materials by color
{ "baseColorFactor": [1, 0, 0, 1], "roughnessFactor": 0.9, "colorId": 0 },
{ "baseColorFactor": [1, 0, 0, 1], "roughnessFactor": 0.1, "colorId": 0 },
{ "baseColorFactor": [0, 0, 1, 1], "roughnessFactor": 0.1, "colorId": 1 }
```

```
vxl mesh turret.voxj --output-profile pbr
    --write-mesh-extra-json-value colorId colorId linear
    --write-primitive-index 0 _PALETTE u8
```

The shader reads `myColors[values.colorId[_PALETTE]]`, and swapping the
two-entry `myColors` recolors every material in the group, dull and shiny alike.
