# `vxl mesh`

_Part of the [mesh plan](README.md)._

```sh
vxl mesh <input> [output] [options]
```

`vxl mesh` triangulates one object's voxels into a mesh. It bakes the object's
palette materials into values. The values can ride along as textures, material
fields, and files beside the mesh. The default output path is the input stem
with the mesh extension. The format comes from `--to`, else the output
extension, else `.glb`.

```sh
# turret.glb, geometry only
vxl mesh turret.voxj

# + embedded albedo, orm, and emissive maps
vxl mesh turret.voxj
  --profile pbr
```

`mesh` writes one object as pure geometry with no hierarchy-node transform.
`--select` and `--select-index` choose the object, the default `--select *`
takes every object. The selection must resolve to exactly one object, so a
multi-object document needs a selector and `--select *` only works for documents
with one object. See
[Object selectors](../vxl-commands/reference/conventions.md#object-selectors).

## Options

1. `--to <glb | gltf>`
   - Default: the output extension, else `glb`
   - Repeatable: no

   The target mesh format. More formats may be added later.
   1. `glb`: binary glTF (`.glb`).
   2. `gltf`: glTF text (`.gltf`).

2. `--from <format>`
   - Default: the input extension
   - Repeatable: no

   The source voxel format.

3. `--method <culled | greedy | naive>`
   - Default: `greedy`
   - Repeatable: no

   The meshing strategy. Choose `culled` or `naive` only when you need stable
   per-voxel topology.
   1. `culled`: emits one unmerged quad per solid-empty boundary face.
   2. `greedy`: merges coplanar, same-material faces into the fewest quads.
   3. `naive`: emits all six faces of every solid voxel, hidden interior
      faces included.

4. `--texture-shape <fit | line | pot | square | n>`
   - Default: `pot`
   - Repeatable: no

   The atlas canvas, counted in cells. A cell is one texel, and in a
   [corner texture](#the-corner-atlas) a 2x2 texel block. Unused cells are
   transparent black, and the mesh never samples them.
   1. `fit`: the near-square packing.
   2. `line`: a single row of cells.
   3. `pot`: the smallest square power of two.
   4. `square`: the smallest square.
   5. `<n>`: an exact `n`x`n` canvas of cells, and a canvas too small errors.

5. `--voxel-size <meters>`
   - Default: `1.0`
   - Repeatable: no

   The real-world edge length of one voxel in meters. The voxel grid is
   unitless, so this flag gives a voxel its physical size. The flag is always
   meters; the writer converts into the target format's native unit. glTF is
   meter-native, so the size passes through. `1.0` opens at one meter per voxel,
   and `0.01` opens at one centimeter. The size applies as a uniform scale to
   vertex positions.

6. `--material-count <count>`
   - Default: derived from use
   - Repeatable: no

   How many materials the mesh carries, numbered from `0`. Every material flag
   names one material by index, and the derived count is the highest mentioned
   index plus one, a skipped index erroring. When defined, the count is the
   whole contract: an index at or above it errors rather than growing the count,
   and an unmentioned index below it is a deliberate placeholder emitting an
   empty material; see [Primitives and materials](#primitives-and-materials).

7. `--material-name <material-index> <name>`
   - Repeatable: yes

   Names a material. The name lands as the glTF `material.name`. A material
   without the flag carries no name.

8. `--material-uv <material-index> <corner | face | swatch | voxel>`
   - Default: derived from use
   - Repeatable: yes

   Declares the indexed material's stream list. Each `--material-uv` adds one
   stream, and the flag order sets the `TEXCOORD` numbers. A domain listed twice
   on a material errors.

   The list sets the material's bake contract. Each of its textures bakes at the
   lowest listed domain at or above its value's domain. Take
   `--material-uv 0 face`. An albedo texture reads a `swatch` value from
   `baseColorFactor`. `face` sits above `swatch`, so the albedo bakes per `face`
   with each `face` repeating its `swatch`'s texel. An occlusion texture reads a
   `corner` value from `--compute-occlusion`. `face` sits below `corner`, so the
   list cannot hold it, and the texture errors. The bake never steps a value
   down, because stepping down loses detail. The value can take the step itself.
   For example, `faceAvg` turns the `corner` value into a `face` value.

   Without the flag the list is derived automatically. Every texture of the
   material puts its value's domain on the list. The duplicates collapse, and
   the survivors sort up the ladder: `swatch`, then `voxel`, then `face`, then
   `corner`. Every texture finds its exact domain on the list, so nothing
   climbs.

   A primitive writes the list of the material it draws with, and a primitive
   with no material writes no streams. `--write-primitive-uv` replaces that
   default: the primitive then writes exactly the streams the flag names; see
   [UV streams](#uv-streams).

9. `--primitive <material-index | none> <src-expr>`
   - Default: the implicit primitive
   - Repeatable: yes

   Declares a primitive. The first argument is the material the primitive draws
   with, and `none` is no material at all. The expression is the select that
   routes the primitive's faces. Primitives number from `0` in flag order. The
   select is a [bool](value-language.md#booleans) read at the
   [face domain](value-language.md#domains), lower domains climbing in, and the
   primitive takes every face whose entry is true. The selects partition the
   faces: a face no select takes errors, and a face two selects take errors too.
   Without the flag the mesh has one primitive, index 0, holding every face and
   addressable like any other. That implicit primitive draws with material 0
   when the mesh carries materials, and with no material when it carries none.
   The first `--primitive` replaces it, so the declared primitives are exactly
   the mesh's; see [Primitives and materials](#primitives-and-materials).

10. `--primitive-name <primitive-index> <name>`
    - Repeatable: yes

    Names a primitive. glTF primitives carry no name field, so the name lands at
    `vxl.name` in the primitive's `extras`. A primitive without the flag carries
    no name.

11. `--compute-index <corner | face | swatch | voxel> <dst-name>`
    - Repeatable: yes

    Computes each entry's index in the domain and binds it to `dst-name` as a
    [`u32`](value-language.md#numbers) vec1 array over the domain. Every entry
    holds its position: the first entry reads `0`, the next `1`, and so on. See
    [Computed index](value-language.md#computed-index).

12. `--compute-occlusion <dst-name>`
    - Repeatable: yes

    Computes occlusion from the voxel geometry and binds it to `dst-name` as a
    per-corner `f32` vec1 in `[0, 1]`: `0` is fully occluded and `1` is fully
    open. See [Computed occlusion](value-language.md#computed-occlusion).

13. `--compute-voxel-position <dst-name>`
    - Repeatable: yes

    Computes each voxel's grid position and binds it to `dst-name` as a
    per-voxel `u32` vec3. Every expression can read it the way it reads a
    palette property. Without the flag nothing computes and the name does not
    exist; see
    [Computed voxel position](value-language.md#computed-voxel-position).

14. `--file-stem <file-stem>`
    - Default: the output mesh's stem
    - Repeatable: no

    Replaces `{file-stem}` in profile file templates. The default resolves
    based on the output path, so an output of `turret.glb` fills
    `{file-stem}-mse.png` as `turret-mse.png`.

15. `--profile <profile>`
    - Repeatable: no

    Applies a profile. The profile expands to its flags, the `valuesFrom` values
    applying first. An explicit flag replaces the element it collides with; see
    the [profile language](profile-language.md).

16. `--value <bindings>`
    - Repeatable: yes

    Defines values the writers and slots can name. The argument holds one
    or more well-formed statements of the value language. Every property
    of the effective palette is a name. The run joins every `--value` and
    profile values entry into one program, bindings evaluating in order
    with let-style redefinition; see
    [Programs](value-language.md#programs).

17. `--values-from <profile>`
    - Repeatable: yes

    Appends a profile's bindings to the program at the flag's position, the
    profile's `valuesFrom` imports first. Any writer elements the profile holds
    stay behind. See [profile language](profile-language.md).

18. `--select <glob>`
    - Default: `*`, selecting every object
    - Repeatable: yes

    Chooses the object by hierarchy path, with a node path selecting its
    subtree. Unions with `--select-index`. Any explicit selector replaces
    the default, so `--select-index` alone never unions with `*`. See
    [Object selectors](../vxl-commands/reference/conventions.md#object-selectors).

19. `--select-index <index>`
    - Repeatable: yes

    Chooses the object by position, an integer or an `a-b` range. Unions
    with `--select`.

20. `--write-file-json-value <dst-file> <dst-name> <src-expr> <linear | srgb>`
    - Repeatable: yes

    Writes a value to a JSON file under `<dst-name>` with the specified
    transfer. The output is one object per file so repeating the flag on one
    path merges into that file; see [JSON files](value-language.md#json-files).

21. `--write-file-png-value <dst-file> <src-expr> <linear | srgb>`
    - Repeatable: yes

    Writes a [swatch, voxel, face, or corner](value-language.md#domains) array
    to an 8-bit PNG beside the mesh, one texel per entry. A corner array takes
    the [corner atlas](#the-corner-atlas)'s block layout. The value's width sets
    the channel format: vec1 writes grey; vec2, grey-alpha; vec3, RGB; vec4,
    RGBA. A component outside `[0, 1]` errors. The file declares its transfer in
    its chunks; see the [notes](value-language.md#notes).
    1. `linear`: applies no transfer.
    2. `srgb`: applies the sRGB transfer, for an image a viewer reads as color.

22. `--write-material-extra-image-file <material-index> <dst-name> <src-file>`
    - Repeatable: yes

    Sets a custom `extras.vxl.values.<dst-name>` entry on the indexed material
    to an image reference. The entry holds a texture index, and the texture
    points at `<src-file>` by relative path; see
    [Material slots](value-language.md#material-slots).

23. `--write-material-extra-image-value <material-index> <dst-name> <src-expr> <linear | srgb>`
    - Repeatable: yes

    Writes an array value as an embedded image. The custom
    `extras.vxl.values.<dst-name>` entry on the indexed material holds its
    texture index. A plain value errors; see
    [Material slots](value-language.md#material-slots).

24. `--write-material-extra-json-file <material-index> <dst-name> <src-file>`
    - Repeatable: yes

    Sets a custom `extras.vxl.values.<dst-name>` entry on the indexed material
    to a `{ "uri": "<src-file>" }` pointer, the path relative; see
    [Material slots](value-language.md#material-slots).

25. `--write-material-extra-json-value <material-index> <dst-name> <src-expr> <linear | srgb>`
    - Repeatable: yes

    Writes a value's numbers into a custom `extras.vxl.values.<dst-name>` entry
    on the indexed material. A plain value writes as its numbers, and an array
    writes as rows; see [Material slots](value-language.md#material-slots).

26. `--write-material-slot-file <material-index> <dst-property> <src-file>`
    - Repeatable: yes

    Sets the texture property `<dst-property>` of the indexed material to
    reference `<src-file>` by relative path. The file can come from
    `--write-file-png-value` or from anywhere else; see
    [Material slots](value-language.md#material-slots).

27. `--write-material-slot-value <material-index> <dst-property> <src-expr>`
    - Repeatable: yes

    Sets the property `<dst-property>` of the indexed material. A plain value
    becomes a material field. An array value embeds as an image, in the glb
    binary chunk or as a data URI in a `.gltf`; see
    [Material slots](value-language.md#material-slots).

28. `--write-mesh-extra-image-file <dst-name> <src-file>`
    - Repeatable: yes

    Sets a mesh `extras.vxl.values.<dst-name>` entry to an image reference. The
    entry holds a texture index, and the texture points at `<src-file>` by
    relative path; see [Palettes](#palettes).

29. `--write-mesh-extra-image-value <dst-name> <src-expr> <linear | srgb>`
    - Repeatable: yes

    Writes an array value as an embedded image. The mesh
    `extras.vxl.values.<dst-name>` entry holds its texture index. A plain value
    errors; see [Palettes](#palettes).

30. `--write-mesh-extra-json-file <dst-name> <src-file>`
    - Repeatable: yes

    Sets a mesh `extras.vxl.values.<dst-name>` entry to a
    `{ "uri": "<src-file>" }` pointer, the path relative; see
    [Palettes](#palettes).

31. `--write-mesh-extra-json-value <dst-name> <src-expr> <linear | srgb>`
    - Repeatable: yes

    Writes a value's numbers into a mesh `extras.vxl.values.<dst-name>` entry. A
    plain value writes as its numbers. An array writes as rows, one row per
    swatch; see [Palettes](#palettes).

32. `--write-primitive-builtin-value <primitive-index> <dst-attribute> <src-expr>`
    - Repeatable: yes

    Writes a value to an attribute glTF defines, `COLOR_0`, on the indexed
    primitive. The corners take the value by
    [domain](value-language.md#domains) so a corner value lands exactly.
    The flag carries no token because the defined vocabulary fixes the encoding.
    An underscore name errors; see
    [Vertex attributes](value-language.md#vertex-attributes).

33. `--write-primitive-custom-value <primitive-index> <dst-name> <src-expr> <linear | srgb | u8 | u16>`
    - Repeatable: yes

    Writes a value to a custom vertex attribute on the indexed primitive.
    `<dst-name>` carries the leading underscore glTF requires of
    application-specific attributes: `_MY_COLOR` lands exactly as written, and a
    bare name errors. The last argument depends on the value's type: `f32` takes
    `linear` or `srgb`, `u8` takes `u8`, and `u16` takes `u16`; a mismatch
    errors. `u8` and `u16` write integer accessors. A `u32` value errors, glTF
    forbidding the width on an attribute; see
    [Vertex attributes](value-language.md#vertex-attributes).

34. `--write-primitive-normal <primitive-index> <false | true>`
    - Default: `true`
    - Repeatable: yes

    Whether the indexed primitive writes `NORMAL`, the mesher's computed
    normal, beside `POSITION`. glTF leaves the attribute optional, and a
    viewer derives flat normals from the triangles. A voxel face is flat, so
    a conforming viewer draws the same pixels either way. `false` drops the
    stream.

35. `--write-primitive-uv <primitive-index> <corner | face | swatch | voxel>`
    - Default: the material's stream list
    - Repeatable: yes

    Writes one UV stream on the indexed primitive. The mesher computes the
    coordinates, and the flag order sets the primitive's `TEXCOORD` numbers. A
    domain listed twice on a primitive errors.

    With the flag the primitive writes exactly the named streams; without it the
    primitive writes its material's list, a `none` primitive writing nothing.
    The list must include every domain the material samples; omitting one
    errors, because the draw would read a missing attribute. A named stream no
    texture bakes at is legal and emits for an outside sampler; see
    [UV streams](#uv-streams).

In every writer, `<src-expr>` holds any expression of the
[value language](value-language.md), and `<src-file>` names an existing file.

## Primitives and materials

A glTF mesh holds primitives. A primitive is one draw with its own vertex data,
its own triangle list, and at most one material. Two materials on one mesh means
two primitives, each holding the faces it draws. `vxl mesh` starts with an
implicit whole-mesh primitive. The first `--primitive` replaces it, and each
further flag adds another. Everything is 0-indexed, and a primitive flag naming
an index at or above the count errors rather than growing it.

A material is the surface a primitive draws with; material flags fill it by
index. Materials are declared by use, with `--material-count` setting the
count outright. At count `0` the glTF carries no `materials` array. A declared
material no primitive draws is legal and emits unused.

`--primitive none` declares a primitive with no material. glTF leaves `primitive.material`
optional, and a viewer draws such a primitive with the spec's default material,
rendering the pixels an empty material produces. `COLOR_0` multiplies into base
color, so a mesh of vertex values alone still shows its colors.

The `--primitive` select routes the faces: a
[bool](value-language.md#booleans) read at the
[face domain](value-language.md#domains) takes every true face into the
primitive. A lower-domain bool can route whole swatches or whole voxels. A
greedy quad can cover several voxels, and when their answers differ the quad
splits, so each piece follows its voxel; see
[computed voxel position](value-language.md#computed-voxel-position). The
selects partition the faces: a face no select takes
would be a silent drop, and a face two selects take would be claimed twice, so
both error. The partition covers the faces the mesher emits. A swatch or voxel
partition covers faces through the used swatches and solid voxels, which no
method changes, so it is the same under every method. A face bool answers per
emitted face, so a face partition that holds under one method can leave
another's faces unclaimed. The fix is a complement: `rest = !(metal || glass)`
covers whatever the mesher emits. A `false` select takes nothing, which is legal
wherever the rest cover the mesh. Selects only route and never change what
geometry exists.

The selects split the model. Every face draws once with its swatch's material:

```sh
# split: solid swatches with material 0, glowing swatches with material 1
vxl mesh turret.voxj
  --value "glowing = emissiveStrength > 0"
  --value "solid = !glowing"
  --primitive 0 solid
  --primitive 1 glowing
```

```jsonc
{
  "asset": { "version": "2.0" },
  "materials": [{}, {}],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": { "POSITION": 0, "NORMAL": 1 },
          "indices": 2,
          "material": 0,
        },
        {
          "attributes": { "POSITION": 3, "NORMAL": 4 },
          "indices": 5,
          "material": 1,
        },
      ],
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
}
```

A face select can split faces that share a swatch. The palette cannot tell
these faces apart, since each reads the same material, but occlusion comes from
the geometry, so a threshold on it sends the crevice faces to a second
material:

```sh
# dirt in the creases: material 1 takes the occluded faces
vxl mesh statue.vox
  --compute-occlusion computedOcclusion
  --value "crevice = faceAvg(computedOcclusion) < 0.7"
  --value "open = !crevice"
  --primitive 0 open
  --primitive 1 crevice
```

## Atlases

Each texture-capable [domain](value-language.md#domains) arranges texels
differently, so the bake has four atlases, one per domain up the ladder, and a
texture takes the atlas of the domain it bakes at. Every atlas sits on the
`--texture-shape` canvas and puts each face's UVs at texel centers. The
palette, voxel, and unwrap atlases sample nearest with clamped wrapping, so a
face reads exactly its texels. The corner atlas samples linear, because
interpolation is its point.

### The palette atlas

Every swatch map of one bake shares a single layout: one texel per distinct
flattened material. The object's layers merge per property name by the format's
layer-override resolution. Each property reads through the last layer whose
palette supplies its name. A voxel's texel is therefore keyed by the tuple of
materials it samples in those winning layers, deduplicated in first-seen raster
order. A single-layer object reduces to one texel per material its voxels use.
Each map fills the same layout from its own value, so

```sh
vxl mesh turret.voxj
  --to gltf
  --profile pbr
```

writes `turret.gltf` with embedded albedo, orm, and emissive maps. Every map is
the same size, with the same flattened material at the same texel, and every
face's UVs sit at its texel's center. The atlas depends on the materials the
object uses, so it is per-mesh.

Nothing auto-defaults. A profile supplies the glTF spec defaults through the
`defaults` mixin, and a hand-written `--value` reads `default()` for a property
no layer supplies; see the
[profile language](profile-language.md#built-in-profiles). Once maps bake,
greedy meshing merges only faces that share a flattened material, since a merged
quad samples one texel. Pure geometry merges on shape alone.

### The voxel atlas

The voxel atlas gives each solid voxel one texel, in the object's raster order.
Each face's UVs sit at its voxel's texel center. A texel needs whole faces, so
a voxel stream in the run caps merging at the voxel: every face is then a
per-voxel quad under any `--method`, and `greedy` collapses to `culled`. A
buried voxel's texel is an unused cell.

The layout serves values that vary per voxel, and
[computed voxel position](value-language.md#computed-voxel-position) is the
domain's producer:

```sh
# alternating layers, baked into the albedo
vxl mesh turret.voxj
  --compute-voxel-position voxelPosition
  --value "bands = mod(voxelPosition.y, 2)"
  --value "albedo = baseColorFactor * lerp(0.8, 1, f32(bands))"
  --write-material-slot-value 0 baseColorTexture albedo
```

The swatch value climbs to the voxels through the multiply, so the map holds
one texel per voxel: the swatch's color, dimmed on alternating layers.

### The unwrap atlas

The unwrap atlas gives each face one texel. It serves values that vary across a
surface: the language's [face domain](value-language.md#domains).
[Computed occlusion](value-language.md#computed-occlusion), reduced from its
corners, is the first face value. The layout packs the face cells into the
canvas and generates the face stream's UVs. `--material-uv 0 face` alone lays
material 0's textures out per face, with swatch values climbing in; see
[UV streams](#uv-streams).

### The corner atlas

The corner atlas gives each face a 2x2 texel block, one texel per corner. The
block's texels sit in the face's corner order, and the face's UVs sit at the
four texel centers, so bilinear interpolation between the centers blends only
the block's texels and reproduces the per-corner gradient. No gutter padding is
needed, since the UVs never leave the block. The linear sampling skips mipmaps,
so minification cannot bleed across blocks. A merged greedy face still carries
one block, so a span whose corner occlusion disagrees splits; see
[computed occlusion](value-language.md#computed-occlusion).

The layout exists for corner values written whole, and computed occlusion is the
first:

```sh
vxl mesh statue.vox
  --compute-occlusion computedOcclusion
  --write-material-slot-value 0 occlusionTexture computedOcclusion
```

The standard `occlusionTexture` slot then shades smooth creases in a stock
viewer with no custom shader. The
[vertex attributes](value-language.md#vertex-attributes) stay the textureless
route for a shader of your own.

## UV streams

A sampled texture is texels plus the coordinates faces read them by, and the
two must agree. One mesh can carry several [atlases](#atlases) at once, and a
face then reads a different spot in each, so a primitive carries one UV stream
per atlas its faces read. The streams are glTF's numbered `TEXCOORD_<n>`
attributes.

The stream list is per material, and the rules live at `--material-uv`: the
declaration, the derived default, and the bake contract. The contract serves the
consumer. `--material-uv 0 face` alone bakes material 0's swatch maps per face,
so an engine that reads one UV set gets one stream. An engine that wants its
face maps ahead of its swatch maps lists `face` first, since the flag order
sets the numbers.

The streams land per primitive. A primitive writes its material's list, a
primitive with no material writes nothing, and `--write-primitive-uv` names a
primitive's streams instead.

Every texture's `texCoord` is derived: the domain it bakes at finds its
position in the stream order of each primitive drawing the material, and that
position is the number. Defaults always agree, since primitives sharing a
material share its list. Two `--write-primitive-uv` orders seating one
material's domain at two positions claim one destination twice, so they error.
No flag hand-wires a slot:

```jsonc
// vxl mesh turret.voxj
//   --profile albedo
//   --compute-occlusion computedOcclusion
//   --value "ao = faceAvg(computedOcclusion)"
//   --write-material-slot-value 0 occlusionTexture ao
// swatch and face both write, so the streams derive [swatch, face]
{
  "asset": { "version": "2.0" },
  "textures": [
    { "sampler": 0, "source": 0 },
    { "sampler": 0, "source": 1 },
  ],
  "materials": [
    {
      "pbrMetallicRoughness": {
        "baseColorTexture": { "index": 0, "texCoord": 0 }, // the swatch stream
      },
      "occlusionTexture": { "index": 1, "texCoord": 1 }, // the face stream
    },
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": {
            "POSITION": 0,
            "NORMAL": 1,
            "TEXCOORD_0": 2,
            "TEXCOORD_1": 3,
          },
          "indices": 4,
          "material": 0,
        },
      ],
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
}
```

## Palettes

The mesh extras flags serve a runtime that resolves materials itself: a game
that swaps team colors or ramps a glow per damage state without re-exporting the
mesh. The four `--write-mesh-extra-*` flags write named entries under the mesh's
`extras.vxl.values`, the same grid the material extras take. The entry shapes
cannot be confused: numbers and rows are themselves, an image is an
`{"index"}`, and a file pointer is a `{"uri"}`. The same name twice, in any two
forms, errors.

The palette pattern is rows beside their join key.
`--write-mesh-extra-json-value` writes the rows, one per swatch in
[shape order](value-language.md#shapes). The key is the
[computed swatch index](value-language.md#computed-index), which lands through
`--write-primitive-custom-value` as one integer per vertex naming the
swatch its face samples, under any custom attribute name. The narrowing
is the width check: `u8(e)` holds 256 swatches, `u16(e)` 65536, and a
palette the width cannot index errors rather than truncates. Every array
value runs over the one effective palette, so every entry of rows shares
the one index. A shader reads `values.albedo[_PALETTE]` against as many
entries as the line writes, and each is plain data the runtime can
replace at will:

```jsonc
// vxl mesh turret.voxj
//   --profile pbr
//   --write-mesh-extra-json-value albedo albedo linear
//   --write-mesh-extra-json-value emissive emissive linear
//   --compute-index swatch swatchIndex
//   --write-primitive-custom-value 0 _PALETTE "u8(swatchIndex)" u8
{
  "asset": { "version": "2.0" },
  "extensionsUsed": ["KHR_materials_emissive_strength"],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": { "POSITION": 0, "NORMAL": 1, "_PALETTE": 2 },
          "indices": 3,
          "material": 0,
        },
      ],
      "extras": {
        "vxl": {
          "values": {
            "albedo": [
              [1, 0, 0, 1],
              [1, 0, 0, 1],
              [0, 0, 1, 1],
            ],
            "emissive": [
              [0, 0, 0],
              [0, 0, 0],
              [4, 3, 0],
            ],
          },
        },
      },
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
}
```

A vec1 value's rows are numbers, and a vecN value's rows are arrays of N, as
`--write-file-json-value` writes them. The encoding argument is that writer's,
for the same reason: the rows are numbers your runtime reads, so nothing but
the flag fixes their encoding.

The halves stay independent. The index attribute alone is the bare join
key, for a runtime that ships its own tables keyed to the effective
palette's swatch order. Rows alone are legal too: a build step reads them
in swatch order with no per-vertex join. Nothing checks the pairing, so a
runtime that needs both writes both.

A storage choice is a flag combination. Embedded rows are
`--write-mesh-extra-json-value`. A sidecar is `--write-file-json-value` with a
`--write-mesh-extra-json-file` pointer at it. Both at once are the rows beside
the json:

```jsonc
// vxl mesh turret.voxj
//   --profile pbr
//   --write-file-json-value turret-values.json albedo albedo linear
//   --write-mesh-extra-json-file albedo turret-values.json
//   --compute-index swatch swatchIndex
//   --write-primitive-custom-value 0 _PALETTE "u8(swatchIndex)" u8
{
  "asset": { "version": "2.0" },
  "extensionsUsed": ["KHR_materials_emissive_strength"],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": { "POSITION": 0, "NORMAL": 1, "_PALETTE": 2 },
          "indices": 3,
          "material": 0,
        },
      ],
      "extras": {
        "vxl": {
          "values": {
            "albedo": { "uri": "turret-values.json" },
          },
        },
      },
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
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
// turret.voxj: colorId groups materials by color
{
  "version": 1,
  "main": {
    "runtimeState": {
      "valuePools": [
        {
          "kind": "vec-4-float",
          "values": [
            [1, 0, 0, 1],
            [0, 0, 1, 1],
          ],
        },
        { "kind": "float", "values": [0.9, 0.1] },
        { "kind": "int", "values": [0, 1] },
      ],
      "palettes": [
        {
          "properties": [
            { "name": "baseColorFactor", "valuePool": 0 },
            { "name": "roughnessFactor", "valuePool": 1 },
            { "name": "colorId", "valuePool": 2 },
          ],
          "materials": [
            [0, 0, 0], // dull red
            [0, 1, 0], // shiny red
            [1, 1, 1], // shiny blue
          ],
        },
      ],
      "rootNodes": [0],
      /* ... */
    },
  },
}
```

```sh
vxl mesh turret.voxj
  --profile pbr
  --write-mesh-extra-json-value colorId colorId linear
  --compute-index swatch swatchIndex
  --write-primitive-custom-value 0 _PALETTE "u8(swatchIndex)" u8
```

The shader reads `myColors[values.colorId[_PALETTE]]`, and swapping the
two-entry `myColors` recolors every material in the group, dull and shiny alike.
