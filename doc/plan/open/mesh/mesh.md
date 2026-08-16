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

3. `--voxel-size <meters>`
   - Default: `1.0`
   - Repeatable: no

   The real-world edge length of one voxel in meters. The voxel grid is
   unitless, so this flag gives a voxel its physical size. The flag is always
   meters; the writer converts into the target format's native unit. glTF is
   meter-native, so the size passes through. `1.0` opens at one meter per voxel,
   and `0.01` opens at one centimeter. The size applies as a uniform scale to
   vertex positions.

4. `--method <culled | greedy | naive>`
   - Default: `greedy`
   - Repeatable: no

   The meshing strategy. Choose `culled` or `naive` only when you need stable
   per-voxel topology.
   1. `culled`: emits one unmerged quad per solid-empty boundary face.
   2. `greedy`: merges coplanar, same-material faces into the fewest quads.
   3. `naive`: emits all six faces of every solid voxel, hidden interior
      faces included.

5. `--texture-shape <fit | line | pot | square | n>`
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

8. `--primitive <material-index | none> <src-expr>`
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

9. `--primitive-name <primitive-index> <name>`
   - Repeatable: yes

   Names a primitive. glTF primitives carry no name field, so the name lands at
   `vxl.name` in the primitive's `extras`. A primitive without the flag carries
   no name.

10. `--uv <corner | face | swatch | voxel>`
    - Default: derived from use
    - Repeatable: yes

    Declares the mesh's stream list. Each `--uv` adds one stream, and the flag
    order sets the `TEXCOORD` numbers. A domain listed twice errors.

    The `--uv` list sets the bake contract. Each texture bakes at the lowest
    listed domain at or above its value's domain. Take `--uv face`. An albedo
    texture reads a `swatch` value from `baseColorFactor`. `face` sits above
    `swatch`, so the albedo bakes per `face` with each `face` repeating its
    `swatch`'s texel. An occlusion texture reads a `corner` value from
    `--compute-occlusion`. `face` sits below `corner`, so `--uv` alone cannot
    hold it, and the texture errors. The bake never steps a value down, because
    stepping down loses detail. The value can take the step itself. For example,
    `faceAvg` turns the `corner` value into a `face` value.

    Without the `--uv` flag the list is derived automatically. Every texture
    value and every `--write-primitive-uv` puts its value's domain on the list.
    The duplicates collapse, and the survivors sort up the ladder: `swatch`,
    then `voxel`, then `face`, then `corner`. Every texture finds its own domain
    on the list, so nothing climbs.

    Every primitive shares the one list, but not every primitive writes every
    stream. Each keeps the listed domains its material samples and writes only
    those. A primitive with no material writes no streams.
    `--write-primitive-uv` replaces the filter: the primitive then writes
    exactly the streams the flag names; see [UV streams](#uv-streams).

11. `--compute-index <corner | face | swatch | voxel> <dst-name>`
    - Repeatable: yes

    Computes each entry's own index in the domain and binds it to `dst-name` as
    a [`u32`](value-language.md#numbers) vec1 array over the domain. Every entry
    holds its own position: the first entry reads `0`, the next `1`, and so on.
    See [Computed index](value-language.md#computed-index).

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

16. `--select <glob>`
    - Default: `*`, selecting every object
    - Repeatable: yes

    Chooses the object by hierarchy path, with a node path selecting its
    subtree. Unions with `--select-index`. Any explicit selector replaces
    the default, so `--select-index` alone never unions with `*`. See
    [Object selectors](../vxl-commands/reference/conventions.md#object-selectors).

17. `--select-index <index>`
    - Repeatable: yes

    Chooses the object by position, an integer or an `a-b` range. Unions
    with `--select`.

18. `--value <bindings>`
    - Repeatable: yes

    Defines values the writers and slots can name. The argument holds one
    or more well-formed statements of the value language. Every property
    of the effective palette is a name. The run joins every `--value` and
    profile values entry into one program, bindings evaluating in order
    with let-style redefinition; see
    [Programs](value-language.md#programs).

19. `--values-from <profile>`
    - Repeatable: yes

    Appends a profile's bindings to the program at the flag's position, the
    profile's `valuesFrom` imports first. Any writer elements the profile holds
    stay behind. See [profile language](profile-language.md).

20. `--write-file-json-value <dst-file> <dst-name> <src-expr> <linear | srgb>`
    - Repeatable: yes

    Writes a value to a JSON file under the name as its key. The token names
    the transfer the written numbers take. A bool value writes `true`/`false`
    under `linear` alone. Repeats on one path merge, so the file is always an
    object; see [JSON files](value-language.md#json-files).

21. `--write-file-png-value <dst-file> <src-expr> <linear | srgb>`
    - Repeatable: yes

    Writes a [swatch, voxel, face, or corner](value-language.md#domains)
    array to an 8-bit PNG beside the mesh, one texel per entry, a corner
    array in the [corner atlas](#the-corner-atlas)'s block layout. The image
    is sized to its value: vec1 through vec4 write grey, grey-alpha, RGB,
    and RGBA, and
    components map to channels by position. Grey-alpha is PNG's only
    two-channel form, so a vec2's second component lands in the alpha
    channel. Pad with `rgb(u, v, 0)` where a viewer should read opaque color.
    A component outside `[0, 1]` errors. The token names the encoding, and the
    file also declares its transfer in its own chunks; see the
    [notes](value-language.md#notes).
    1. `linear`: applies no transfer, for the data channels glTF wants
       linear.
    2. `srgb`: applies the sRGB transfer, for an image a viewer reads as
       color.

22. `--write-material-extra-image-file <material-index> <dst-name> <src-file>`
    - Repeatable: yes

    Sets a custom `extras.vxl.values.<name>` entry on the indexed material to
    an image reference. The entry holds a texture index, and the texture
    points at the named file by relative path; see
    [Material slots](value-language.md#material-slots).

23. `--write-material-extra-image-value <material-index> <dst-name> <src-expr> <linear | srgb>`
    - Repeatable: yes

    Writes an array value as an embedded image. The custom
    `extras.vxl.values.<name>` entry on the indexed material holds its
    texture index. A plain value errors, because an image needs texels; see
    [Material slots](value-language.md#material-slots).

24. `--write-material-extra-json-file <material-index> <dst-name> <src-file>`
    - Repeatable: yes

    Sets a custom `extras.vxl.values.<name>` entry on the indexed material to
    a `{"uri"}` pointer at the named JSON file by relative path; see
    [Material slots](value-language.md#material-slots).

25. `--write-material-extra-json-value <material-index> <dst-name> <src-expr> <linear | srgb>`
    - Repeatable: yes

    Writes a value's numbers into a custom `extras.vxl.values.<name>` entry
    on the indexed material. A plain value writes as its numbers, and an
    array writes as rows; see
    [Material slots](value-language.md#material-slots).

26. `--write-material-slot-file <material-index> <dst-property> <src-file>`
    - Repeatable: yes

    Sets a texture property of the indexed material to reference an existing
    file by relative path. The file can come from `--write-file-png-value` or
    from anywhere else; see
    [Material slots](value-language.md#material-slots).

27. `--write-material-slot-value <material-index> <dst-property> <src-expr>`
    - Repeatable: yes

    Sets one property of the indexed material. A plain value becomes a
    material field. An array value embeds as an image, in the glb binary
    chunk or as a data URI in a `.gltf`; see
    [Material slots](value-language.md#material-slots).

28. `--write-mesh-extra-image-file <dst-name> <src-file>`
    - Repeatable: yes

    Sets a mesh `extras.vxl.values.<name>` entry to an image reference. The
    entry holds a texture index, and the texture points at the named file by
    relative path; see [Palettes](#palettes).

29. `--write-mesh-extra-image-value <dst-name> <src-expr> <linear | srgb>`
    - Repeatable: yes

    Writes an array value as an embedded image. The mesh
    `extras.vxl.values.<name>` entry holds its texture index. A plain value
    errors; see [Palettes](#palettes).

30. `--write-mesh-extra-json-file <dst-name> <src-file>`
    - Repeatable: yes

    Sets a mesh `extras.vxl.values.<name>` entry to a `{"uri"}` pointer at
    the named JSON file by relative path; see [Palettes](#palettes).

31. `--write-mesh-extra-json-value <dst-name> <src-expr> <linear | srgb>`
    - Repeatable: yes

    Writes a value's numbers into a mesh `extras.vxl.values.<name>` entry. A
    plain value writes as its numbers. An array writes as rows, one row per
    swatch; see [Palettes](#palettes).

32. `--write-primitive-builtin-value <primitive-index> <dst-attribute> <src-expr>`
    - Repeatable: yes

    Writes a value to an attribute glTF defines, `COLOR_0`, on the indexed
    primitive. The corners take the value by
    [domain](value-language.md#domains), and a corner value lands exactly.
    The defined vocabulary fixes the encoding, so the flag carries no token.
    An underscore name errors; the custom flag is its home; see
    [Vertex attributes](value-language.md#vertex-attributes).

33. `--write-primitive-custom-value <primitive-index> <dst-name> <src-expr> <linear | srgb | u8 | u16>`
    - Repeatable: yes

    Writes a value to a custom vertex attribute on the indexed primitive. The
    name is typed with the underscore glTF requires of application-specific
    attributes. `_MY_COLOR` lands exactly as written, and a bare name errors.
    An `f32` value takes `linear` or `srgb`, the transfer the stored floats
    take. A `u8` or `u16` value takes its own width as the token, the two
    cross-checked, and writes an integer accessor. A `u32` value errors, glTF
    forbidding the width on an attribute; see
    [Vertex attributes](value-language.md#vertex-attributes).

34. `--write-primitive-normal <primitive-index> <false | true>`
    - Default: `true`
    - Repeatable: yes

    Whether the indexed primitive writes `NORMAL`, the mesher's computed
    normal, beside `POSITION`. glTF leaves the attribute optional, and a
    viewer derives flat normals from the triangles. A voxel face is flat, so
    a conforming viewer draws the same pixels either way. `false` drops the
    stream, bytes a data primitive never reads.

35. `--write-primitive-uv <primitive-index> <corner | face | swatch | voxel>`
    - Default: the mesh's stream list filtered to the material's need
    - Repeatable: yes

    Writes a UV stream on the indexed primitive. The source is the mesher's
    own coordinates, so the flag carries only which streams write, repeats
    stacking in the primitive's `TEXCOORD` order. A domain twice on one
    primitive errors. A spelled primitive writes exactly its spelled streams,
    a `none` primitive's default filtering to nothing. A spelling that omits a
    domain the material samples errors, a draw reading a missing attribute. A
    spelled stream no texture bakes at is legal and emits for an outside
    sampler; see [UV streams](#uv-streams).

A writer's arguments read destination first, then source, then the token
when one exists. The order is an assignment: the location before what
fills it, a binding's `name = expr` order with the encoding trailing. A
material or primitive index is part of the destination, so it rides
ahead of the rest: which object, then what on it. A writer's name ends
in its source kind. `-value` takes an expression and writes its value.
`-file` takes an existing file. `-normal` takes the mesher's computed
normal; source and destination are both fixed, so the flag carries only
whether the write happens. `-uv` takes the mesher's computed coordinates
the same way; the flag carries only which streams write. A `<src-expr>`
is any expression of the [value language](value-language.md), a defined
name the simplest. A `<src-file>` names an existing file.

Every destination takes one claim. A second flag claiming it errors, an
identical spelling included, so writing the same thing twice never passes as
agreement: a slot filled twice, a material or primitive named twice, one JSON
key written twice. A profile element is no second claim, since an explicit flag
replaces it; see the [profile language](profile-language.md). The two exceptions
bind names rather than write destinations: value bindings redefine let-style,
and compute requests alias, each binding its name to the one
computation.

## Primitives and materials

A glTF mesh holds primitives. A primitive is one draw: its own vertex data, its
own triangle list, and at most one material. Two materials on one mesh means two
primitives, each holding the faces it draws. By default a run carries no
materials and one primitive, index 0, holding every face and addressable like
any other. Each `--primitive <material-index | none> <src-expr>` declares a
primitive: the material it draws with and the select that routes its faces. The
first flag replaces the implicit primitive. The implicit primitive draws with
material 0 when the mesh carries materials, and with no material when it carries
none. Everything is 0-indexed, and a primitive flag naming an index at or above
the declared primitives errors rather than growing the count.

The materials declare by use. Every material flag names its material by index,
and the mentions are the declaration: the count is the highest mention plus one,
and a skipped index errors. `--material-count` spells the count instead, the
whole contract: an index at or above it errors rather than growing the count,
and an unmentioned index below it is a deliberate placeholder, an empty material
holding its spot for an index-stable layout. A profile's `materials` list spells
its length the same way, so a hand flag beside a profile still errors rather
than growing the profile's count.

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
ladder, and the primitive takes every face whose entry is true. A swatch bool
routes whole swatches: every face takes its swatch's answer. A voxel bool
routes whole voxels, a merged span its entries disagree across splitting
first; see
[computed voxel position](value-language.md#computed-voxel-position). A face
bool routes faces one by one, reaching below the palette to what only the
mesh knows. A plain bool takes every face or none, and `true` is the
whole-mesh select on any material. The selects partition the faces. A face no
select takes would be a silent drop, so it errors. A face two selects take is
two flags claiming one destination, the error the rest of the flag surface
already throws. The partition covers the faces the mesher emits. A
swatch-bool or voxel-bool partition of the used swatches or voxels holds
under every `--method`, while a face-bool gap can open under one method and
not another. The complement select is whole under all of
them, and a `false` select takes nothing, legal wherever the rest cover the
mesh. Selects only route; they never change what geometry exists.

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

A face select splits inside a swatch. Occlusion lives on the mesh, not the
palette, so a crevice mask sends one swatch's seam faces to their own material:

```sh
# dirt in the creases: material 1 takes the occluded faces
vxl mesh statue.vox
  --compute-occlusion computedOcclusion
  --value "crevice = faceAvg(computedOcclusion) < 0.7"
  --value "open = !crevice"
  --primitive 0 open
  --primitive 1 crevice
```

A styled catch-all is an explicit complement. The last primitive selects
what the others do not, `--value "rest = !(metal || glass)"`, so the
partition stays whole and every face still names its material.

`--material-name` lands as the glTF `material.name`. glTF primitives carry no
name field, so `--primitive-name` rides the primitive's `extras` at `vxl.name`.

## The palette atlas

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

## The voxel atlas

The voxel atlas is a per-mesh layout with a texel per solid voxel, in the
object's own raster order, the layout of every texture that bakes at the
voxel domain. Each face's UVs sit at its voxel's texel center, read with a
nearest-neighbor sampler and clamped wrapping, so a face samples exactly its
voxel's texel. A texel needs whole faces, so a voxel stream in the run caps
merging at the voxel. Every face is then a per-voxel quad under any
`--method`, `greedy` collapsing to `culled`. A buried voxel's texel is
transparent black like any unused cell, and the mesh never samples it.

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
one texel per voxel, its swatch's color dimmed on alternating layers.

## The unwrap atlas

The unwrap atlas is a per-mesh UV unwrap with a texel per face, the layout of
every texture that bakes at the face domain. It serves values that vary across a
surface: the language's [face domain](value-language.md#domains).
[Computed occlusion](value-language.md#computed-occlusion), reduced from its
corners, is the first face value. The layout packs the face cells into the
canvas and generates the face stream's UVs. `--uv face` alone lays every texture
out per face, swatch values climbing in; see [UV streams](#uv-streams).

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

```sh
vxl mesh statue.vox
  --compute-occlusion computedOcclusion
  --write-material-slot-value 0 occlusionTexture computedOcclusion
```

The standard `occlusionTexture` slot then shades smooth creases in a stock
viewer, no custom shader involved. The
[vertex attributes](value-language.md#vertex-attributes) stay the textureless
route for a shader of your own.

## UV streams

A sampled texture is texels plus the coordinates faces read them by, and the two
must agree. Each texture-capable domain therefore has its own arrangement. A
swatch texture holds one texel per swatch, and every face of the swatch reads
the same texel. A voxel texture holds one texel per voxel, the voxel's faces
sharing it. A face texture holds one texel per face, each face its own. A
corner texture holds a 2x2 block per face, each corner its own texel. One mesh
can carry several kinds at once, one face then reading a different spot in
each, so a primitive carries one UV stream per layout its faces read, glTF's
numbered `TEXCOORD_<n>` attributes. A swatch, voxel, or face texture samples
nearest; a corner texture samples linear, the interpolation its point.

The stream list is derived from use when nothing spells it: each texture bakes
at its value's own domain, a `--write-primitive-uv` mention joins, and the list
holds the domains in use in ladder order, `[swatch]` through
`[swatch, voxel, face, corner]`, empty when nothing writes a texture.
`--uv <corner | face | swatch | voxel>`, repeatable, spells the list
instead, one stream per flag in `TEXCOORD` order, and a
profile's `uvs` key is the same list in config, any `--uv` replacing all of it.
The spelled list is the whole contract: each texture bakes at the lowest listed
domain at or above its value's domain, climbing in, so `--uv face` alone bakes
the swatch maps per face, one stream for a consumer that reads one UV set, and
`--write-primitive-uv` domain outside the list errors. A texture whose domain
sits above every listed entry errors, since stepping down is never implicit;
`faceAvg` spells the step. The order pins the relative numbers: an engine that
wants its face maps ahead of its swatch maps spells `--uv face --uv swatch`.

The streams land per primitive. An unspelled primitive writes the list filtered
to the domains its material samples: a primitive of swatch maps carries one
and a `none` primitive carries none. `--write-primitive-uv` spells a primitive's
streams instead, in the primitive's own `TEXCOORD` order, and a spelled stream
no texture bakes at emits anyway, the coordinates an externally-baked texture,
an engine lightmap, samples by.

Every texture's `texCoord` then is derived: the domain it bakes at looks up in
the stream order of each primitive drawing its material, and the position is the
number. Filtered defaults always agree, primitives sharing a material sharing
its need; two spellings seating one material's domain at two positions are two
flags claiming one destination, the usual error. No flag hand-wires a slot:

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
`extras.vxl.values`, the same grid the material extras take. `json-value` puts a
value's numbers in the entry, a plain value as its numbers and an array as rows,
one per swatch in [shape order](value-language.md#shapes).
`json-file` points the entry at an existing JSON file. `image-value` embeds an
array as a PNG and stores its texture index. `image-file` references an existing
image the same way. The entry shapes cannot be confused: numbers and rows are
themselves, an image is an `{"index"}`, and a file pointer is a `{"uri"}`. The
same name twice, in any two forms, is two flags claiming one destination, the
usual error.

The palette pattern is rows beside their join key.
`--write-mesh-extra-json-value` writes the rows. The key is the
[computed swatch index](value-language.md#computed-index) landing through
`--write-primitive-custom-value`, one integer per vertex naming the
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

A vec1 value's rows are numbers, and a vecN value's rows are arrays of N, the
`--write-file-json-value` shapes. The token is that writer's token for the same
reason: the rows are numbers your own runtime reads, nothing fixes their
encoding, so the flag declares it.

The halves stay independent, so each stands alone. The index attribute by
itself is the bare join key, for a runtime that ships its own tables
keyed to the effective palette's swatch order. Rows by themselves are
legal too, data a build step reads in swatch order with no per-vertex
join. Nothing checks the pairing, so a runtime that needs both writes
both.

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
