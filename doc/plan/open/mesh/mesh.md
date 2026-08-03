# `vxl mesh`

_Part of the [mesh plan](README.md)._

```
vxl mesh <input> [output] [options]
```

Triangulates one object's voxels into a glTF mesh, and bakes the object's
palette materials into values that ride the mesh as textures, material
fields, and files beside it. The default output path is the input stem
with the mesh extension. The format comes from `--to`, else the output
extension, else `.glb`.

```
vxl mesh turret.voxj                         # turret.glb, geometry only
vxl mesh turret.voxj --output-profile pbr    # + embedded albedo, orm, and emissive maps
```

`mesh` writes one object as pure geometry, with no hierarchy-node
transform applied. The common case is pulling a leaf object out without
placement. `--select` and `--select-index` choose the object; the
selection must resolve to exactly one, so a multi-object document needs a
selector. See
[Object selectors](../vxl-commands/reference/conventions.md#object-selectors).

## Options

1. `--to` `gltf` | `glb`: target mesh format, glTF text (`.gltf`) or
   binary (`.glb`). Inferred from the output extension when omitted,
   defaulting to `.glb`.
2. `--from <format>`: source voxel format. Inferred from the input
   extension when omitted.
3. `--voxel-size <meters>` (default `1.0`): the real-world edge length of
   one voxel in meters, the mesh twin of `voxelize`'s `--voxel-size`. The
   voxel grid is unitless and glTF is meter-native, so this is where a
   voxel gains a physical size: `1.0` opens at one meter per voxel,
   `0.01` at one centimeter. Applied as a uniform scale to vertex
   positions only.
4. `--method` `greedy` | `culled` | `naive` (default `greedy`): meshing
   strategy. `greedy` merges coplanar, same-material faces into the
   fewest quads. `culled` emits one quad per solid-empty boundary face,
   unmerged. `naive` emits all six faces of every solid voxel, hidden
   interior faces included. Choose `culled` or `naive` only when you need
   stable per-voxel topology.
5. `--atlas` `palette` | `unwrap` (default `palette`): material-map
   atlas layout. `palette` is one texel per flattened material the mesh
   uses, `unwrap` one texel per face. See
   [The palette atlas](#the-palette-atlas) and
   [The unwrap atlas](#the-unwrap-atlas).
6. `--texture-shape` `fit` | `line` | `pot` | `square` | `<n>` (default
   `pot`): the atlas canvas. `fit` is the near-square packing, `line` a
   single row of texels, `pot` the smallest square power of two,
   `square` the smallest square, and `<n>` an exact `n`x`n` canvas,
   rejected when too small. Unused cells are transparent black the mesh
   never samples.
7. `--material-count <count>` (default `1`): how many materials the
   mesh carries, numbered from `0`. Every material flag names one by
   index, and an index at or above the count errors rather than
   growing it; see
   [Primitives and materials](#primitives-and-materials).
8. `--material-name <material-index> <name>`: names a material, the
   glTF `material.name`. Optional; a material without the flag
   carries none.
9. `--primitive <material-index> <src-expr>`: declares a primitive,
   the material it draws with and the select that routes its faces,
   primitives numbering from `0` in flag order. The value is a
   [bool](value-language.md#booleans) read at the
   [face domain](value-language.md#domains), lower domains climbing
   in, and the primitive takes every face whose entry is true. The
   selects partition the faces, so a face no select takes errors,
   and so does one two selects take. Without the flag the mesh has
   one primitive, index 0, material 0, every face, addressable like
   any other, and the first `--primitive` replaces it, so the
   declared primitives are exactly the mesh's; see
   [Primitives and materials](#primitives-and-materials).
10. `--primitive-name <primitive-index> <name>`: names a primitive.
    glTF primitives carry no name field, so the name rides the
    primitive's `extras`. Optional.
11. `--uv` `row` | `face`: declares the mesh's UV streams in
    `TEXCOORD` order, one per flag. Omitted, the list derives from
    the domains the run's textures use; spelled, it replaces the
    derived list whole, a used domain missing from it erroring and
    an unused entry legal, its stream emitted for an outside
    sampler. See [UV streams](#uv-streams).
12. `--compute-occlusion <dst-name>`: computes occlusion from the
    voxel geometry and binds it under the name, a per-corner vec1
    in `0..1`, `1` fully open, one entry per face corner, available
    to every expression the way a palette property is. Without the
    flag nothing computes and the name does not exist; see
    [Computed occlusion](value-language.md#computed-occlusion).
13. `--value <dst-name> <src-expr>`: defines a value the writers and
    slots can name. Every property of the effective palette is a
    name, values evaluate in flag order, and redefinition is
    let-style, so a later `--value` overrides an earlier one and
    later expressions see the new value. The expression grammar is
    the [value language](value-language.md).
14. `--write-file-json-value <dst-file> <dst-name> <src-expr> <linear | srgb>`:
    writes a value to a JSON file under the name as its key,
    the token naming the transfer the written numbers take, a bool
    value writing `true`/`false` under `linear` alone. Repeatable,
    and repeating it on one path merges, so the file is always an
    object; see [JSON files](value-language.md#json-files).
15. `--write-file-png-value <dst-file> <src-expr> <linear | srgb>`:
    writes a [row or face](value-language.md#domains) array to an
    8-bit PNG beside the mesh, one texel per entry, a corner value
    stepping down through `faceAverage` first. The image
    is sized to its value: vec1 through vec4 write grey, grey-alpha,
    RGB, and RGBA, components mapping to channels by position.
    Grey-alpha is PNG's only two-channel form, so a vec2's second
    component lands in the alpha channel; pad with `rgb(u, v, 0)`
    where a viewer should read opaque color. The token names the
    encoding: `srgb` applies the sRGB transfer, for an image a viewer
    reads as color, and `linear` applies none, for the data channels
    glTF wants linear. A component outside `0..1` errors. The file
    also declares its transfer in its own chunks; see the
    [notes](value-language.md#notes).
16. `--write-material-extra-image-file <material-index> <dst-name> <src-file>`:
    sets a custom `extras.vxl.values.<name>` entry on the indexed
    material to an image reference, the entry holding a texture index
    and the texture pointing at the named file by relative path; see
    [Material slots](value-language.md#material-slots).
17. `--write-material-extra-image-value <material-index> <dst-name> <src-expr> <linear | srgb>`:
    writes an array value as an embedded image, the custom
    `extras.vxl.values.<name>` entry on the indexed material holding
    its texture index; a plain value errors, an image needing texels;
    see [Material slots](value-language.md#material-slots).
18. `--write-material-extra-json-file <material-index> <dst-name> <src-file>`:
    sets a custom `extras.vxl.values.<name>` entry on the indexed
    material to a `{"uri"}` pointer at the named JSON file by
    relative path; see
    [Material slots](value-language.md#material-slots).
19. `--write-material-extra-json-value <material-index> <dst-name> <src-expr> <linear | srgb>`:
    writes a value's numbers into a custom `extras.vxl.values.<name>`
    entry on the indexed material, a plain value as its numbers and
    an array as rows; see
    [Material slots](value-language.md#material-slots).
20. `--write-material-slot-file <material-index> <dst-property> <src-file>`:
    sets a texture property of the indexed material to reference an
    existing file by relative path, whether `--write-file-png-value`
    wrote it or not; see
    [Material slots](value-language.md#material-slots).
21. `--write-material-slot-value <material-index> <dst-property> <src-expr>`:
    sets one property of the indexed material. A plain value becomes
    a material field; an array value embeds, its image landing in the
    glb binary chunk or as a data URI in a `.gltf`; see
    [Material slots](value-language.md#material-slots).
22. `--write-mesh-extra-image-file <dst-name> <src-file>`: sets a mesh
    `extras.vxl.values.<name>` entry to an image reference, the
    mesh-side `--write-material-extra-image-file`; see
    [Palettes](#palettes).
23. `--write-mesh-extra-image-value <dst-name> <src-expr> <linear | srgb>`:
    writes an array value as an embedded image, the mesh
    `extras.vxl.values.<name>` entry holding its texture index; a
    plain value errors; see [Palettes](#palettes).
24. `--write-mesh-extra-json-file <dst-name> <src-file>`: sets a mesh
    `extras.vxl.values.<name>` entry to a `{"uri"}` pointer at the
    named JSON file by relative path; see [Palettes](#palettes).
25. `--write-mesh-extra-json-value <dst-name> <src-expr> <linear | srgb>`:
    writes a value's numbers into a mesh `extras.vxl.values.<name>`
    entry, a plain value as its numbers and an array as rows, one per
    flattened material; see [Palettes](#palettes).
26. `--write-primitive-builtin-value <primitive-index> <dst-attribute> <src-expr>`:
    writes a value to an attribute glTF defines, `COLOR_0`, on the
    indexed primitive, the corners taking it by
    [domain](value-language.md#domains), a corner value exactly;
    the defined vocabulary fixes the encoding, so the flag carries no
    token, and an underscore name errors, the custom flag its home;
    see [Vertex attributes](value-language.md#vertex-attributes).
27. `--write-primitive-custom-value <primitive-index> <dst-name> <src-expr> <linear | srgb>`:
    writes a value to a custom vertex attribute on the indexed
    primitive; the name is typed with the underscore glTF requires of
    application-specific attributes, `_MY_COLOR` landing exactly as
    written, and a bare name errors; see
    [Vertex attributes](value-language.md#vertex-attributes).
28. `--write-primitive-index <primitive-index> <dst-name> <u8 | u16>`:
    writes the per-vertex palette index as its own custom attribute
    on the indexed primitive, the name underscore-typed and the width
    `u8` or `u16`, `u8` holding 256 rows and `u16` 65536, a palette
    the width cannot index erroring. The numbers are the effective
    palette's own rows under any select. Independent of every other
    flag: alone it is the bare index, and beside extras rows it is
    the join key a shader reads them by; see [Palettes](#palettes).
29. `--value-profile <profile>`: applies a profile of values as if
    each were a `--value` at the flag's own position; the built-ins
    ship in the binary and the rest come from `.vxlconfig`.
    Repeatable; see the [profile language](profile-language.md).
30. `--output-profile <profile>`: applies an output profile, the
    run's whole surface, the geometry options, materials, primitives,
    files, and extras, expanded to the flags it spells, its `values`
    list applying its value profiles first. At most once per run; an
    explicit flag replaces the element it collides with; see the
    [profile language](profile-language.md).
31. `--file-stem <file-stem>`: replaces `{file-stem}` in profile file
    templates. The default is the output mesh's own stem, after the
    output path resolves, so an output of `turret.glb`, named or
    derived from the input, fills `{file-stem}-mse.png` as
    `turret-mse.png` with no flag at all, and renaming the mesh
    renames every templated file with it.
32. `--select <glob>`: choose the object by hierarchy path, matched
    the way `hierarchy show` matches node paths, so a node path
    selects its subtree. Repeatable; unions with `--select-index`. See
    [Object selectors](../vxl-commands/reference/conventions.md#object-selectors).
33. `--select-index <index>`: choose the object by position, an
    integer or an `a-b` range. Repeatable; unions with `--select`.

A writer's arguments read destination first, then the source, then
the token when one exists, the order `--value` binds in with the
encoding trailing: an assignment, the location before what fills it.
A material or primitive index is part of the destination, so it
rides ahead of the rest: which object, then what on it. A writer's
name ends in its source kind: `-value` takes an expression and
writes its value, `-file` an existing file, and `-index` the palette
row number. A `<src-expr>` is any expression of the
[value language](value-language.md), a defined name the simplest,
and a `<src-file>` names an existing file.

## Primitives and materials

A glTF mesh holds primitives, and a primitive is one draw: its own
vertex data, its own triangle list, and at most one material, so two
materials on one mesh means two primitives holding the faces each
draws. By default a run has one material and one primitive, index 0,
material 0, holding every face, addressable like any other, which
is the whole story until a flag says otherwise: `--material-count`
sets how many materials exist, and each
`--primitive <material-index> <src-expr>` declares a primitive,
the material it draws with and the select that routes its faces,
the first flag replacing the implicit primitive. Everything is
0-indexed, and a flag naming an index at or above a count errors
rather than growing it.

The select routes the faces. It names a
[bool](value-language.md#booleans) value read at the
[face domain](value-language.md#domains), lower domains climbing
the ladder, and the primitive takes every face whose entry is true.
A row bool routes whole rows, every face taking its row's answer, a
face bool routes faces one by one, reaching below the palette to
what only the mesh knows, and a plain bool takes every face or
none, `true` the whole-mesh select on any material. The selects
partition the faces: a face no select takes is a face with no
material, a silent default, and a face two selects take is two
flags claiming one destination, the errors the rest of the flag
surface already throws. The partition covers the faces the mesher
emits, so a row-bool partition of the used rows holds under every
`--method`, while a face-bool gap can open under one method and not
another; the complement select is whole under all of them, and a
`false` select takes nothing, legal wherever the rest cover the
mesh. Selects only route; they never change what geometry exists.

The selects split the model, every face drawn once with its row's
material:

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
"meshes": [ { "primitives": [
  { "attributes": { "POSITION": 0, "NORMAL": 1 }, "indices": 2, "material": 0 },
  { "attributes": { "POSITION": 3, "NORMAL": 4 }, "indices": 5, "material": 1 }
] } ]
```

A face select splits inside a row. Occlusion lives on the mesh, not
the palette, so a crevice mask sends one row's seam faces to their
own material:

```
# dirt in the creases: material 1 takes the occluded faces
vxl mesh statue.vox
    --compute-occlusion computedOcclusion
    --value crevice "faceAverage(computedOcclusion) < 0.7"
    --value open "!crevice"
    --material-count 2
    --primitive 0 open
    --primitive 1 crevice
```

A styled catch-all is an explicit complement: the last primitive
selects what the others do not, `--value rest "!(metal || glass)"`,
so the partition stays whole and every face still names its
material.

`--material-name` lands as the glTF `material.name`; glTF primitives
carry no name field, so `--primitive-name` rides the primitive's
`extras`.

## The palette atlas

All the maps of one bake share a single atlas layout: one texel per
distinct flattened material. The object's layers merge per property name
by the format's layer-override resolution, each property read through the
last layer whose palette supplies its name, so a voxel's texel is keyed
by the tuple of materials it samples in those winning layers,
deduplicated in first-seen raster order. A single-layer object reduces to
one texel per material its voxels use. Each map fills the same layout
from its own value, so

```
vxl mesh turret.voxj --to gltf --output-profile pbr
```

writes `turret.gltf` with embedded albedo, orm, and emissive maps, every
map the same size with the same flattened material at the same texel.
Every face's UVs sit at its texel center, read with a nearest-neighbor
sampler and clamped wrapping, so a face samples exactly its texel. The
atlas depends on the materials the object uses, so it is per-mesh, not
shared across meshes.

Nothing auto-defaults: a profile spells the glTF spec defaults through
the `defaults` mixin, and a hand-written `--value` reads `default()` for
a property no layer supplies; see the
[profile language](profile-language.md#built-in-profiles). Once maps are
baked, greedy meshing merges only faces that share a flattened material,
since a merged quad samples one texel; pure geometry merges on shape
alone.

## The unwrap atlas

`--atlas unwrap` is a per-mesh UV unwrap with a texel per face, the
layout for values that vary across a surface: the language's
[face domain](value-language.md#domains), with
[computed occlusion](value-language.md#computed-occlusion) reduced from
its corners the first face value. The layout packs the face rectangles
into the canvas and generates the UVs, and every texture is laid out
per face, row values climbing in, so the derived stream list is
`[face]` alone; see [UV streams](#uv-streams).

## UV streams

A sampled texture is texels plus the coordinates faces read them by, and
the two must agree, so each texture-capable domain has its own
arrangement: a row texture holds one texel per palette row, every face
of the row reading the same texel, and a face texture one texel per
face, each face its own. One mesh can carry both kinds at once, one face
then reading a different spot in each, so the mesh carries one UV stream
per domain in use, glTF's numbered `TEXCOORD_<n>` attributes.

The stream list derives from use when nothing spells it: `[row]` when
only row values write textures, `[face]` when only face values do,
`[row, face]` when both do, and empty when nothing does, no stream
emitted. `--uv <row | face>`, repeatable, spells the whole list instead,
one stream per flag in `TEXCOORD` order, and a profile's `uvs` key is
the same list in config, any `--uv` replacing all of it. The override
exists to pin the numbers: the derived list moves a stream when textures
come and go, and an engine that wants its face maps at a fixed slot
spells `--uv face --uv row`. A used domain missing from the spelled list
errors; an entry nothing uses is legal and emits its stream anyway, the
coordinates an externally-baked texture, an engine lightmap, samples by.

Every texture's `texCoord` then derives: its value's domain is looked up
in the list, and the position is the number, so no flag hand-wires a
slot:

```jsonc
// vxl mesh turret.voxj --output-profile albedo
//     --compute-occlusion computedOcclusion
//     --value ao "faceAverage(computedOcclusion)"
//     --write-material-slot-value 0 occlusionTexture ao
// row and face both write, so the streams derive [row, face]
"materials": [ {
  "pbrMetallicRoughness": {
    "baseColorTexture": { "index": 0, "texCoord": 0 }   // the row stream
  },
  "occlusionTexture": { "index": 1, "texCoord": 1 }     // the face stream
} ]
```

## Palettes

The mesh extras flags serve a runtime that resolves materials
itself: a game that swaps team colors or ramps a glow per damage
state without re-exporting the mesh. The four `--write-mesh-extra-*`
flags write named entries under the mesh's `extras.vxl.values`, the
same grid the material extras take: `json-value` puts a value's
numbers in the entry, a plain value as its numbers and an array as
rows, one per flattened material in
[shape order](value-language.md#shapes); `json-file` points the
entry at an existing JSON file; `image-value` embeds an array as a
PNG and stores its texture index; `image-file` references an
existing image the same way. The entry shapes cannot be confused:
numbers and rows are themselves, an image is an `{"index"}`, and a
file pointer is a `{"uri"}`. The same name twice, in any two forms,
is two flags claiming one destination, the usual error.

The palette pattern is rows beside their join key.
`--write-mesh-extra-json-value` writes the rows, and
`--write-primitive-index` writes the attribute they are read by,
one integer per vertex naming the flattened material its face
samples. The attribute is yours to spell: the name with its
underscore typed like any custom attribute, and the width, `u8`
holding 256 rows and `u16` holding 65536, a palette the width
cannot index erroring rather than truncating. Every array value
runs over the one effective palette, so every entry of rows shares
the one index: a shader reads `values.albedo[_PALETTE]` against as
many entries as the line writes, and each is plain data the runtime
can replace at will:

```jsonc
// vxl mesh turret.voxj --output-profile pbr
//     --write-mesh-extra-json-value albedo albedo linear
//     --write-mesh-extra-json-value emissive emissive linear
//     --write-primitive-index 0 _PALETTE u8
"meshes": [
  {
    "primitives": [ {
      "attributes": { "POSITION": 0, "NORMAL": 1, "_PALETTE": 2 },
      "material": 0
    } ],
    "extras": { "vxl": { "values": {
      "albedo":   [ [1, 0, 0, 1], [1, 0, 0, 1], [0, 0, 1, 1] ],
      "emissive": [ [0, 0, 0],    [0, 0, 0],    [4, 3, 0]    ]
    } } }
  }
]
```

A vec1 value's rows are numbers and a vecN value's are arrays of N,
the `--write-file-json-value` shapes, and the token is that writer's
for the same reason: the rows are numbers your own runtime reads,
nothing fixes their encoding, so the flag declares it.

The flags stay independent, so each half of the pattern stands
alone. `--write-primitive-index` by itself is the bare index, an
attribute with no rows, for a runtime that ships its own tables
keyed to the effective palette's row order; rows by themselves are
legal too, data a build step reads in material order with no
per-vertex join. Nothing checks the pairing, so a runtime that needs
both spells both. The index is a custom attribute like any other, so
a `--write-primitive-custom-value` spelling its name is two flags
claiming one destination.

A storage choice is a flag combination: embedded rows are
`--write-mesh-extra-json-value`, a sidecar is
`--write-file-json-value` with a `--write-mesh-extra-json-file`
pointer at it, and both at once are the rows beside the json, the
way an embedded slot rides beside its loose `--write-file-png-value`
copy:

```jsonc
// vxl mesh turret.voxj --output-profile pbr
//     --write-file-json-value turret-values.json albedo albedo linear
//     --write-mesh-extra-json-file albedo turret-values.json
//     --write-primitive-index 0 _PALETTE u8
"meshes": [
  {
    "primitives": [ {
      "attributes": { "POSITION": 0, "NORMAL": 1, "_PALETTE": 2 },
      "material": 0
    } ],
    "extras": { "vxl": { "values": {
      "albedo": { "uri": "turret-values.json" }
    } } }
  }
]
```

A mesh entry contests nothing a slot writes.
`--write-material-slot-file 0 baseColorTexture skin.png` fills the
material while
`--write-mesh-extra-json-value baseColor baseColorFactor linear`
writes the rows: two destinations serving two readers. A stock
viewer renders the slots and never reads the extras, and a runtime
that reads them draws its own pixels, so a mesh carries the baked
look and the swappable data side by side.

Layers end at the flatten. A runtime grouping is authored data
instead: an int property on the palette entries groups materials,
flattens like any property, and writes like any value, so the engine
swaps one compact palette of its own keyed by the group:

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

The shader reads `myColors[values.colorId[_PALETTE]]`, and
swapping the two-entry `myColors` recolors every material in the
group, dull and shiny alike.
