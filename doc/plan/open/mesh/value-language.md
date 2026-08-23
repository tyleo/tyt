# Value language

_Part of the [mesh plan](README.md)._

The expression language behind `vxl mesh`'s material values. A binding,
`name = expr`, defines a named value, and a run gathers every binding it is
given into one [program](#programs). The writer and slot flags listed in
[`vxl mesh`](mesh.md#options) take expressions too, landing the results in
images, JSON files, and the mesh's material. Every property of the
[effective palette](mesh.md#the-palette-atlas) enters the program as a name.

A value sits on three axes, each with a section below:

1. [Shape](#shapes): plain, or an array holding one entry per element of its
   domain
2. [Domain](#domains): what an array runs over, the palette's swatches or the
   mesh's voxels, faces, or corners
3. Type: a [number](#numbers), `f32` or unsigned, vec1 through vec4, or a
   [bool](#booleans) or [string](#strings) with no components at all

## Programs

A program is a sequence of statements, each a binding `name = expr` terminated
by `;`. A statement binds and does nothing else: a bare expression would compute
a value and drop it, so it errors. Bindings evaluate in order, and a name can be
redefined let-style; see the [notes](#notes).

No one writes a program whole. Each `--value` and each profile `values` entry is
a fragment holding one or more bindings, with the trailing `;` optional. The run
appends a `;` to every fragment, joins the fragments in flag order with profile
values expanding at their flag's position, and parses the result once. The empty
statement makes the appended `;` harmless after a fragment that already ends in
one. An all-whitespace fragment errors at its flag, and a parse error names its
fragment's origin, the flag or the profile entry.

```sh
--value "tint = baseColorFactor.rgb"
--value "dim = tint * 0.5; bright = tint * 1.2"   # one fragment, two bindings
```

## Shapes

A value is plain or an array, independent of its vec1-vec4 dimension. A property
is an array holding one element per swatch, a distinct flattened material of the
effective palette, in the [palette atlas](mesh.md#the-palette-atlas)'s texel
order. A numeric literal is plain. Elementwise operations pair arrays element by
element and broadcast a plain value across an array, so `1 - roughnessFactor` is
an array. Two arrays of one domain always align, and mixed domains climb the
[ladder](#domains).

```sh
--value "cutoff = 0.4"                      # plain vec1, a literal
--value "tint = baseColorFactor.rgb"        # array vec3, one entry per swatch
--value "bright = tint * 1.2"               # array * plain broadcasts
--value "mask = step(0.5, metallicFactor)"  # 1 where a material is metal
```

`max(e)`, `min(e)`, `sum(e)`, and `avg(e)` reduce an array across the palette,
per component, to a plain value; the binary `min`/`max` are elementwise like the
operators. The emissive bake is the canonical use:

```sh
--value "emissive = emissiveFactor * emissiveStrength / max(emissiveStrength)"
```

Each material's emissive color, scaled into `[0, 1]` of the palette's strongest
strength. An all-zero palette divides `0 / 0` and errors; guard with
`max(max(emissiveStrength), 0.001)`.

`e[i]` samples an array at entry index `i` into a plain value: `tint[0]` is the
first material's tint. The index has to be a plain unsigned vec1, `u8`, `u16`,
or `u32`, below the array's entry count; an `f32` index, an out-of-range index,
or an array index errors. Indexing and swizzling commute:
`baseColorFactor[0].rgb` and `baseColorFactor.rgb[0]` name the same value.

The destinations read the shape: an image takes an array, one texel per entry, a
material factor takes a plain value, and JSON takes either.

## Domains

A value's domain is what it has one entry per. There are five domains. A plain
value has one entry. A swatch value has one entry per swatch, this is the shape
every property starts with. A voxel value has one entry per solid voxel. A face
value has one entry per face the mesher emits. A corner value has one entry per
face corner, exactly four per face. A voxel, face, or corner value comes from a
[computed value](#computed-values) or from the climbs and reductions below.
Domain is orthogonal to dimension: `albedo` is a swatch vec3, occlusion a corner
vec1.

```sh
# a four-swatch palette, six solid voxels, meshed into ten faces
plain    1 entry
swatch   4 entries    # one per swatch
voxel    6 entries    # one per solid voxel
face    10 entries    # one per face
corner  40 entries    # four corners per face
```

The domains form a ladder, plain to swatch to voxel to face to corner, and every
step up duplicates losslessly: a plain value broadcasts everywhere, a swatch
value reads per voxel through the voxel's swatch, a voxel value reads per face
through the face's voxel, and a face value duplicates onto its four corners.
Climbing is implicit because nothing is lost, the same rule that broadcasts a
scalar across an array: in `albedo * ao`, `albedo` climbs to the corners and the
multiply pairs entries. `swatch(e)`, `voxel(e)`, `face(e)`, and `corner(e)` name
the same climb where the domain is the point. Each lifts its value to the named
domain, acts as the identity on a value already there, and errors on a step
down. `face(albedo)` bakes one swatch map per face while the others stay
compact, `face(0.5)` gives a plain value the texels a texture needs, and
`face(1u32)` under `swatchSum` counts a swatch's faces. A climb moves entries
and never touches them, taking a bool or a [string](#strings) as readily as a
number.

A step down loses entries, so it takes an explicit reduction naming its
destination, and the reduction accepts any array above it. `faceAvg(e)`,
`faceMin(e)`, `faceMax(e)`, and `faceSum(e)` take a corner array to a face
array, reducing each face's four corners per component. `voxelAvg(e)` and its
siblings take a face or corner array to a voxel array, reducing each voxel's
boundary: a merged face reads in piecewise, one piece per voxel it covers. A
piece carries what its face holds, one entry from a face array and four from a
corner array. `swatchAvg(e)` and its siblings take a voxel, face, or corner
array to a swatch array, reducing each swatch's entries; a face or corner array
reads in through the same voxel pieces. A face merged across two swatches
counts toward both. The unary reductions, `min`/`max`/`sum`/`avg`, take any
array to plain across its whole domain. A corner value meeting a face
destination without a reduction is an error, never an implicit average. `min`
and `max` compose exactly across the rungs, so `swatchMin(voxelMin(e))` is
`swatchMin(e)`. The means and sums weigh their own rung:
`swatchAvg(voxelAvg(e))` weighs voxels evenly where `swatchAvg(e)` weighs
entries. The corner rung is uniform, every face owning exactly four, so
`swatchAvg(faceAvg(e))` is still the grand mean of a swatch's corners. The
written chain chooses the weighting.

A destination entry can be empty. Under `greedy` and `culled` a fully enclosed
voxel emits no faces, and a material whose voxels are all enclosed is a swatch
with no faces; `naive` emits all six faces of every solid voxel, leaving nothing
empty. No other destination has the problem: a voxel set always has a boundary,
so the face domain itself holds entries; a face always has its four corners; and
a swatch always owns voxels, so a step down from the voxel domain never meets an
empty swatch. The avg, min, and max reductions error on an empty destination
entry, naming it and the method; the sums answer `0`, the empty sum. A
computation that has to survive a buried swatch is built from the sums, and the
buried swatch then reads the value the author wrote for it:

```sh
--compute-occlusion computedOcclusion
--value "aoFace = faceAvg(computedOcclusion)"
--value "faceCount = swatchSum(face(1u32))"
# a buried swatch reads 0
--value "ao = swatchSum(aoFace) / f32(max(faceCount, 1))"
```

The destinations read the domain. A texture takes a swatch, voxel, face, or
corner array, one texel per entry, its layout and UV stream following the domain
it bakes at; see [UV streams](mesh.md#uv-streams). A
[select](mesh.md#primitives-and-materials) reads at the faces, and the
[vertex attributes](#vertex-attributes) read at the corners, the ladder's top,
with lower domains climbing in. A material factor takes a plain value alone.

## Numbers

A number takes one of four types: `f32`, the 32-bit float, and the unsigned
`u8`, `u16`, and `u32`. The types never mix, and nothing converts implicitly:
every operator, comparison, and function takes one numeric type across its
numeric operands, so `voxelPosition.y * 0.5` errors and
`f32(voxelPosition.y) * 0.5` converts. Unsigned values come from an `int`-kind
palette property, read as `u32`, and from the computed [index](#computed-index)
and [voxel position](#computed-voxel-position), both `u32`.

A literal names its type or takes it from context. A decimal point makes an
`f32`, a suffix pins any type, `2f32`, `2u8`, `2u16`, `2u32`, and a bare whole
number infers from the expression around it, so `mod(voxelPosition.y, 2)` reads
`2` as `u32`. A literal nothing types errors, and a suffix fixes it.

The conversions are explicit and componentwise. `f32(e)` takes any unsigned
value exactly, erroring where `f32` holds no exact image of it. `u8(e)`,
`u16(e)`, and `u32(e)` widen an unsigned value losslessly, narrow one under a
range check, and take an `f32` only at exact whole components, erroring on any
fraction rather than rounding. The `ceil_u8` through `round_u32` forms round an
`f32` by the named mode into the named range.

Arithmetic keeps its type. Unsigned `+`, `-`, and `*` error on overflow and on a
difference below zero rather than wrapping, `/` floors with the floored `mod`
completing it, and unary `-` takes `f32` alone because the unsigned types hold
no negatives. `min`, `max`, and the sums keep the operand type, and every
average returns an `f32` because a mean is fractional.

The destinations read the type. JSON writes an unsigned value as an integer
literal, and a `u8` or `u16` value writes an integer
[vertex attribute](#vertex-attributes) of its width, though a `u32` never writes
one because glTF forbids the width. Every other destination takes `f32` alone,
with `f32(e)` carrying an unsigned value in.

## Booleans

A comparison makes a bool: `<`, `<=`, `>`, `>=`, `==`, and `!=` take a vec1 on
each side and yield one. The literals `true` and `false` name one directly,
plain, with both names reserved and a colliding property backtick-quoted. `==`
and `!=` also compare two [strings](#strings) by value.

A wider comparison names its fold: inside `any(c)` and `all(c)` the sides share
a dimension or either is a vec1, broadcasting as it does through `*`. The
components compare one by one and the reduction folds the answers, `any` with or
and `all` with and, so `all(baseColorFactor.rgb > 0.9)` is true where a color
runs near white. The comparison is legal only directly inside its reduction: a
bare `vec3 < vec3` errors because it names no fold, and the component answers
never escape as a value. No bool vector exists, and the reductions take a
comparison written in place, never a stored bool.

`!`, `&&`, `^`, and `||` combine bools, with `^` the exclusive or. Nothing else
touches the type. A bool never mixes with a number, so there is no `0`/`1`
coercion: `rgb(glowing, 0, 0)` errors, arithmetic on a bool errors, and every
function rejects one except `mix` and the domain climbs. `mix(x, y, cond)` is
the deliberate bridge out, picking `x` or `y` per entry by the bool, so
`mix(0f32, 1, glowing)` makes the `0`/`1` mask; see [Functions](#functions). The
climbs move a bool's entries and never touch them, and beyond these only
grouping parentheses and `e[i]` apply, the index sampling a bool array at an
entry.

The type reaches three destinations: the select of
[`--primitive`](mesh.md#primitives-and-materials), reading at the face domain
with lower domains climbing in; a [JSON value](#json-files), written as `true`
or `false`; and a boolean [material property](#material-slots), plain alone:

```sh
--value "glowing = emissiveStrength > 0"   # bool array, one entry per swatch
--value "solid = !glowing"
--primitive 0 solid
--primitive 1 glowing
```

Shape follows the numeric rules: a comparison against a plain value broadcasts
across an array, two arrays pair element by element, the logical operators do
the same, and `solid && metallic` is an array wherever either side is. The
comparisons bind looser than the numeric operators and the logical operators
looser still, `!` excepted, so `a + 1 > b && c > d` reads as
`((a + 1) > b) && (c > d)`; see the [precedence note](#notes). `==` and `!=`
compare floats exactly, which is right against an authored palette property and
surprising against a computed value, where `0.1 + 0.2 == 0.3` is false.

## Strings

A string literal is double-quoted, `"MASK"`, and a string palette property, the
voxj `string` kind, is an array like any property. The type has the bool's
footprint: no components, no swizzle, no arithmetic, and no coercion, so a
string never meets a number or a bool. A literal takes any characters except the
quote, with no escapes, the backtick-name rule again.

Five operations touch the type. `==` and `!=` compare two strings into a bool,
entry by entry, with a plain side broadcasting across an array; no other
comparison applies because strings hold no order. `mix(x, y, cond)` picks
between two strings by a bool, the bridge numbers already have. `e[i]` samples a
string array at an entry. `default(name, fallback)` fills a string hole. The
domain climbs, `swatch`/`voxel`/`face`/`corner`, lift a string's entries
untouched. The equality does the routing, turning an authored tag into a select:

```sh
--value 'glass = tag == "glass"'
--value 'solid = !glass'
--primitive 0 solid
--primitive 1 glass
```

A string reaches three destinations, each under `linear` alone, the identity
token the bool takes: an enum material property, a JSON file value, and a JSON
extras entry. The JSON forms write the quoted string itself, so
`--write-mesh-extra-json-value tag tag linear` lands `["glass", "steel"]` beside
a palette index. Every numeric destination, a PNG, a texture, a vertex
attribute, a factor, rejects a string.

An enum property takes one word from the fixed list its format's schema defines,
glTF's `alphaMode` taking `OPAQUE`, `MASK`, or `BLEND`. The property reads a
plain string, and the writer checks the value against the list at the edge,
erroring on an unknown token with the format named. No conversion exists in the
language; only the destination knows the list:

```sh
# static: cutout mode, written directly
--write-material-slot-value 0 alphaMode '"MASK"'

# computed: cutout only where the palette holds transparency
--value 'mode = mix("OPAQUE", "MASK", min(baseColorFactor.a) < 1)'
--write-material-slot-value 0 alphaMode mode
```

In a shell, single quotes carry the inner double quotes through, the backtick
advice again.

## Color spaces

Every expression evaluates in linear RGB, and the conversion functions visit
other spaces as plain vec3 math: the language never tracks which space a vec3
sits in, the author does, the same trust the transfer tokens extend. Linear RGB
is the hub every space converts to and from, so a hop between two others is two
calls. The constructor names carry dimension, not meaning, `rgb(...)` assembling
an Oklab triple as readily as a color, and such components read best through the
position alphabet, `lab.x` rather than `lab.r`.

`oklabFromRgb(c)` and `rgbFromOklab(l)` visit Oklab, the perceptual space: equal
numeric steps look like equal visual steps, where linear RGB crowds the
distinguishable dark shades into a sliver of its range. Oklab is defined from
linear sRGB, so the language's native form is exactly its input; the conversion
runs two fixed matrices around a cube root, and the inverse runs the same steps
backward. `distance` there measures how different two colors look, `.x` holds
perceived lightness, 0 black to 1 white, `.y` runs green to red, and `.z` blue
to yellow.

```sh
--value "lab = oklabFromRgb(baseColorFactor.rgb)"
--value "reddish = distance(lab, oklabFromRgb(rgb(1, 0, 0))) < 0.25"
--value "darker = rgbFromOklab(lab * rgb(0.8, 1, 1))"   # dimmed, hue held
```

`oklchFromRgb(c)` and `rgbFromOklch(l)` visit Oklch, Oklab's polar form: `.x`
holds the same lightness, `.y` chroma, 0 at gray and rising with colorfulness,
and `.z` hue. Hue as a plain number is the form's power: `mod(lch.z + 0.1, 1)`
turns every material a tenth of the way around the wheel with lightness and
chroma held.

Hue runs as a turn in `[0, 1]`, and a gray has none, so `oklchFromRgb` answers
hue 0 at zero chroma. `rgbFromOklch` errors on a hue outside `[0, 1]`, leaving
the wrap to the author's `mod(h, 1)`, and on a negative chroma. A converted-back
color can leave the gamut with components outside `[0, 1]`, and no conversion
clamps: an image writer already errors there, and the bound stays the author's
`clamp`.

## JSON files

`--write-file-json-value <dst-file> <dst-name> <src-expr> <linear | srgb>`
writes one value under the name as its key. Repeating it on one path merges, so
a file with several values takes several flags rather than a grouping construct
in the language:

```sh
--write-file-json-value turret-pbr.json albedo albedo linear
--write-file-json-value turret-pbr.json orm orm linear
--write-file-json-value turret-pbr.json emissive emissive linear
```

```jsonc
{
  "albedo": [
    [1, 0, 0, 1],
    [0, 0, 1, 1],
  ],
  "orm": [
    [1, 0.9, 0],
    [1, 0.1, 1],
  ],
  "emissive": [
    [0, 0, 0],
    [0.5, 0.5, 0],
  ],
}
```

Five rules cover it:

1. The flag's `<dst-name>` sets the key, named at the destination the way every
   extras entry is
2. Repeating the flag on one path merges into that file, in flag order
3. The same name twice into one file errors because one key would silently win
4. Two writers of different kinds on one path error, the rule that rejects two
   images resolving to one file name
5. The output is always an object, so one value and five produce the same shape

The values come out one number per component, nested in an array per swatch when
the value is one. A vec1 writes `0.4`, a vec4 writes `[1, 0, 0, 1]`, and either
over the palette writes an array of those. The token names the transfer the
numbers take: `linear` writes them as evaluated, and `srgb` transfer-encodes
them under the image rules, so an alpha component stays linear and a component
outside `[0, 1]` errors. Both write full floats: an `srgb` JSON holds
display-encoded floats where an `srgb` PNG holds display-encoded bytes. The
token rides each flag, so one file can mix encodings, each key taking its
declared transfer, and nothing about the destination appears in an expression.

A bool writes as itself, `true` or `false`, an array of them per entry. Its
token is `linear`, the identity; `srgb` on a bool errors because a transfer
curve belongs to numbers. A runtime wanting `0`/`1` instead takes the written
mask, `mix(0f32, 1, glowing)`. A [string](#strings) writes its quoted JSON form
the same way, `linear` its only token.

Merging at the flag keeps every value simple, a vector or a bool and never a
grouping, so a dot postfix is always a swizzle and the checker asks only shape
and dimension. A [mesh palette](mesh.md#palettes) is a fixed shape the exporter
builds, not a grouping in the language, and a written file is never a voxel-json
value pool.

The single stored form follows the voxel-json
[value kinds](../../closed/voxj-value-kinds/README.md), which deleted their
color kinds because the stored form and the transfer are a writer's choices, not
the value's. voxj stores linear light, the language evaluates in it, and a
conversion happens only where something outside has an opinion.

## Material slots

`--write-material-slot-value <material-index> <dst-property> <src-expr>` sets
one property of the indexed material, destination before source like every
writer, with the index riding first: which material, then what on it. The
examples write material `0`, the mention declaring it. The property takes the
target format's name, the leaf of its material schema, so the flag invents no
vocabulary; the writer does the nesting and the `extensionsUsed` bookkeeping:

```sh
# pbrMetallicRoughness.baseColorTexture
--write-material-slot-value 0 baseColorTexture albedo
# pbrMetallicRoughness.metallicRoughnessTexture
--write-material-slot-value 0 metallicRoughnessTexture orm
# occlusionTexture, sharing one image
--write-material-slot-value 0 occlusionTexture orm
# emissiveTexture
--write-material-slot-value 0 emissiveTexture emissive
# extensions.KHR_materials_emissive_strength
--write-material-slot-value 0 emissiveStrength maxStrength
# extensions.KHR_materials_ior
--write-material-slot-value 0 ior glassIor
```

The vocabulary comes from the resolved output format because one run writes one
mesh file and `--to` has already chosen it. Each format brings its own names:
FBX calls its slots `DiffuseColor` and `NormalMap`, MTL calls them `map_Kd` and
`map_Pr`, and glTF packs roughness into `metallicRoughnessTexture`, which a
neutral `roughness` slot could not honestly target. The vocabularies do not
overlap, so retargeting a script from `--to gltf` to `--to fbx` makes every slot
name unknown at once and errors loudly, with the error naming the format it
checked against.

The property's type decides how its expression reads:

| Property type    | Argument                                                |
| ---------------- | ------------------------------------------------------- |
| `*Texture`       | an array expression to embed, or a file via `-file`     |
| number or vector | a plain expression of that dimension                    |
| boolean          | a plain bool expression                                 |
| enum             | a plain string expression, one of the property's tokens |

```sh
--write-material-slot-value 0 baseColorTexture albedo    # array value, embedded
--write-material-slot-value 0 alphaCutoff "cutoff / 2"   # plain vec1 expression
--write-material-slot-value 0 doubleSided true           # plain bool
--write-material-slot-value 0 alphaMode '"MASK"'         # enum, a plain string
--write-material-slot-file 0 baseColorTexture albedo.png  # a written file
```

A property that is not a texture stays uniform across the atlas's one material,
so its expression is plain, which is how a `max()` reduction lands in the
material. An enum property reads a plain [string](#strings) checked against its
token list.

A texture property takes its image from its argument. A value embeds: the bytes
land in the mesh, the property points at them, and the slot's fixed requirement
supplies the encoding. A `--write-material-slot-file` references the named file
by relative path. The file has to come from this run's `--write-file-png-value`
because only its writer's value gives the image the domain that seats the
reference on a UV stream. A paint-over edits the written file in place after the
run, and the reference holds. A `--write-file-png-value` beside a
`--write-material-slot-value` leaves the mesh referencing the embedded copy,
with the loose file a working duplicate of the same bytes. Two slots
naming one value share the one embedded image, which is how an ORM packing fills
both of its slots; two slots demanding different encodings of one value error.

A writer and a slot stay separate flags because each is whole alone. A writer
alone makes a file the mesh never mentions. A factor is a slot with no bytes,
`--write-material-slot-value 0 emissiveStrength maxStrength` writing a number
straight into the material. A texture slot carries its own image, embedding a
value or referencing a file, so the two families meet where
`--write-material-slot-file` names the file `--write-file-png-value` writes.

The writer sets only what a slot names. Today's bake breaks that rule in one
place, injecting an `emissiveFactor` of `[1, 1, 1]` whenever it binds an
emissive texture. The reason is a real glTF trap: emission multiplies
`emissiveTexture` by `emissiveFactor`, and the factor defaults to black, so
binding the texture alone emits nothing at all. The injection is still a silent
default of the kind this design rejects, and it would fight a
`--write-material-slot-value` sending anything else to `emissiveFactor` with no
rule saying which wins. The profile writes the factor instead, the way it writes
every other default.

glTF fixes each texture slot's encoding: `baseColorTexture` and
`emissiveTexture` are sRGB, and `metallicRoughnessTexture`, `occlusionTexture`,
and `normalTexture` are linear. A value-form slot encodes to order, so it cannot
mismatch. A `--write-material-slot-file` cross-checks its writer's token
against the slot, an error rather than a mesh that renders wrong.

A map with no standard property has two homes: loose beside the mesh through
`--write-file-png-value`, its transfer named by the writer and stamped in the
file's chunks, or inside the mesh through the material extras.

The material extras are the custom namespace: four flags write named entries
under the material's `extras`, the key glTF reserves for application data, in
the `vxl.values` namespace the mesh extras share.

1. `--write-material-extra-json-value <material-index> <dst-name> <src-expr> <linear | srgb>`
   puts the value's numbers in the entry itself, a plain vec1 as one number, a
   vecN as an array of N, and an array value as rows, one per swatch
2. `--write-material-extra-image-value <material-index> <dst-name> <src-expr> <linear | srgb>`
   embeds an array as an image, the entry holding its texture index; a plain
   value errors because an image needs texels
3. `--write-material-extra-image-file <material-index> <dst-name> <src-file>`
   writes the same `{"index"}` entry with the texture referencing the named file
   by relative uri
4. `--write-material-extra-json-file <material-index> <dst-name> <src-file>`
   points the entry at a JSON file instead

The entry shapes cannot be confused:

```jsonc
{
  "asset": { "version": "2.0" },
  "materials": [
    {
      "extras": {
        "vxl": {
          "values": {
            "heatScale": { "index": 3 },
            "accentColor": [0.87, 0.44, 0.44],
            "wear": { "uri": "turret-wear.json" },
          },
        },
      },
    },
  ],
  /* ... */
}
```

A conforming viewer ignores it all; your runtime looks the name up. The `-value`
forms carry the token because a custom entry fixes no encoding: `srgb` for a
color your runtime reads display-encoded and `linear` for everything else, with
an alpha component staying linear like the image rule. `--write-file-json-value`
takes the same token for the same reason, so the choice between the two is
placement, an entry inside the mesh against a file beside it. The `-file` forms
carry none because the named file's chunks or writer already declared what it
takes, and an image entry stays a bare index either way, the PNG's chunks
speaking for it.

Keeping the extras separate from the slots keeps typos loud: an unknown standard
property in `--write-material-slot-value` still errors. The same name twice, in
any two forms, errors. Two `--write-material-extra-image-value` naming one value
share one image, the two-encodings rule applying across the slots and the extras
alike, and a format without `extras` rejects the whole grid.

## Vertex attributes

`--write-primitive-builtin-value <primitive-index> <dst-attribute> <src-expr>`
writes a value to an attribute glTF defines on the indexed primitive: `COLOR_0`,
the vertex color. The defined vocabulary fixes each attribute's encoding the way
the material schema fixes its slots', so the flag carries no token, and an
unknown or underscore name errors; the custom flag is the underscore's home.
Dimension picks the accessor type, vec1 through vec4 writing SCALAR, VEC2, VEC3,
and VEC4 floats.

`--write-primitive-custom-value <primitive-index> <dst-name> <src-expr> <linear | srgb>`
is the custom twin. glTF requires the underscore prefix on application-specific
attributes, so the name is typed with it, `_MY_COLOR` landing exactly as written
and a bare name erroring; an attribute only your shader reads can never collide
with a defined name. Nothing fixes its encoding, so the value's type picks the
accessor and the token picks the transfer. An `f32` value takes `linear` or
`srgb`, the transfer the stored floats take. A `u8` or `u16` value takes
`linear` and writes an integer accessor of its width; `srgb` on an integer
errors. A `u32` value errors because glTF forbids the width on an attribute;
narrow with `u16(e)` first; see [Numbers](#numbers).

The attributes live on the corners, the [ladder](#domains)'s top, so a value of
any domain climbs in: a swatch value gives each corner its face's swatch's
entry, leaving a merged greedy quad uniform, a voxel value splits a span its
entries disagree across, and a corner value writes each corner exactly, which is
what [computed occlusion](#computed-occlusion) wants.

A format without vertex attributes rejects both flags. Each flag writes its
indexed primitive alone, so two primitives carry exactly the attributes their
flags name. The [computed index](#computed-index) rides the custom flag narrowed
to a width, which is how the [palette pattern](mesh.md#palettes)'s join key
lands.

## Computed values

Three flags compute values from the geometry rather than the palette. Each
request binds its result under the flag's `<dst-name>`, ahead of the program the
way palette properties are. Nothing computes unrequested: without the flag the
name does not exist, and an expression naming it gets the ordinary unknown-name
error. Several requests of one computation bind their names as aliases rather
than colliding. A profile requests the same computations through its compute
keys; see [profile values](profile-language.md#profile-values).

### Computed index

`--compute-index <corner | face | swatch | voxel> <dst-name>` binds each entry's
index in the chosen domain, a [`u32`](#numbers) vec1 array where every entry
holds its position: the first entry reads `0`, the next `1`, and so on, in the
order the domain's atlas lays cells out. The swatch index rides the vertices in
the [palette pattern](mesh.md#palettes), and the others serve a runtime keyed to
another layout's cells.

The `voxel` index can change the geometry under the
[computed voxel position](#computed-voxel-position) rules: read at the faces or
above it splits merged spans, and baked at the voxel it caps merging. The
`swatch` index limits merging the way any swatch value does; see
[the palette atlas](mesh.md#the-palette-atlas). A `face` or `corner` index
numbers entries the mesh already has and changes nothing.

```sh
--compute-index swatch swatchIndex
--write-primitive-custom-value 0 _PALETTE "u8(swatchIndex)" linear
```

The pair is the palette pattern's per-vertex half, with the `u8(e)` narrowing
doubling as the width check: a palette past 256 swatches errors.

### Computed occlusion

`--compute-occlusion <dst-name>` binds occlusion computed from the voxel
geometry, each face corner reading the voxels that meet there: a corner `f32`
vec1 in `[0, 1]`, `1` fully open. A corner is where neighbors crowd in, so the
result is a [corner](#domains) value, the first value that varies across a
surface: every palette property is per swatch, which is why the unwrap and
corner atlases exist.

The request is explicit because it can change the geometry: written
corner-exact, through a vertex attribute or a
[corner texture](mesh.md#the-corner-atlas), the value makes greedy merging split
a quad where its corner occlusion disagrees.

The value mixes like any other, so tuning takes one expression each rather than
a flag family:

```sh
--compute-occlusion computedOcclusion
--value "ao = lerp(1, computedOcclusion, 0.8)"     # strength 0.8
--value "ao = max(computedOcclusion, 0.2)"         # min brightness 0.2
--value "aoFace = faceAvg(ao)"                     # corners down to faces
--write-file-png-value turret-ao.png aoFace srgb   # color space: the token
```

Three texture routes carry it. Written whole, the value bakes the
[corner atlas](mesh.md#the-corner-atlas), a texel per corner sampled bilinear,
so the standard `occlusionTexture` slot shades smooth creases in a stock viewer.
Stepped down through `faceAvg` or its siblings, it bakes a face texture, one
flat texel per face. Stepped to the swatches,
`swatchAvg(faceAvg(computedOcclusion))`, it lands in the palette layout, one
value per material beside the other swatch maps, with no extra stream at all.
Each route rides the [UV stream](mesh.md#uv-streams) of the domain it bakes at.
A sampled neighborhood model, a radius and a falloff curve, is a possible
extension beyond the discrete corner method.

### Computed voxel position

`--compute-voxel-position <dst-name>` binds each voxel's grid coordinates, a
[voxel](#domains) [`u32`](#numbers) vec3 read straight off the geometry: every
palette property is per swatch and occlusion lives at the corners, so this is
the first value that varies per voxel.

The request is explicit because it can change the geometry. Read at the faces or
above, through a select, a vertex attribute, or a face or corner texture, a
voxel value splits a merged greedy quad where its entries disagree, the
corner-occlusion rule one rung down. A texture baked at the voxel itself caps
merging at the voxel outright; see [the voxel atlas](mesh.md#the-voxel-atlas).

The value mixes like any other, so a per-voxel pattern takes one expression
each:

```sh
--compute-voxel-position voxelPosition
# height runs 0 to 1 up the object; bands alternates layers
--value "height = f32(voxelPosition.y) / f32(max(voxelPosition.y))"
--value "bands = mod(voxelPosition.y, 2)"
```

## Functions

One item per function; the [grammar](#grammar)'s checking rules give the exact
dimensions, shapes, and numeric types.

1. `r(x)`, `rg(x, y)`, `rgb(x, y, z)`, and `rgba(x, y, z, w)` build a vector
   from vec1 parts. This is how channels pack into a map:

   ```
   rgb(occlusion, roughness, metallic)   # the orm pack
   ```

2. Unary `min(e)` and `max(e)` reduce an array across its domain, per component,
   to a plain value:

   ```
   max(emissiveStrength)   # the palette's strongest strength
   ```

3. Binary `min(a, b)` and `max(a, b)` are elementwise, a vec1 broadcasting from
   either side:

   ```
   max(maxStrength, 0.001)   # floors a divisor
   ```

4. `sum(e)` and `avg(e)` are the other reductions, the total and the mean across
   the array's domain:

   ```
   avg(baseColorFactor)   # the palette's mean color
   ```

5. `abs(e)` is the componentwise magnitude:

   ```
   abs(tint - avg(tint))   # each material's spread around the mean
   ```

6. `dot(a, b)` multiplies matching components and adds the products, one number
   out. The sides share a dimension exactly, nothing broadcasts, and a vec1 pair
   degenerates to the plain product:

   ```
   dot(tint, rgb(0.2126, 0.7152, 0.0722))   # a luminance weighting
   ```

7. `length(e)` is the vector's magnitude, `pow(dot(e, e), 0.5)` under one name;
   a vec1's length is its absolute value:

   ```
   length(emissiveFactor)   # the emissive color's overall strength
   ```

8. `distance(a, b)` is `length(a - b)`, the straight-line gap between two
   points, read in [Oklab](#color-spaces) when the gap should match what the eye
   sees:

   ```
   distance(lab, oklabFromRgb(rgb(1, 0, 0)))   # how far from red
   ```

9. `normalize(e)` is `e / length(e)`, the direction alone at length 1. A zero
   vector divides `0 / 0` and errors like any non-finite:

   ```
   normalize(offset)   # the direction, the magnitude dropped
   ```

10. `cross(a, b)` is the vec3 cross product, the vector perpendicular to both
    sides:

    ```
    cross(u, v)   # a normal for the plane u and v span
    ```

11. `oklabFromRgb(c)` and `rgbFromOklab(l)` convert a vec3 between linear RGB
    and Oklab, the perceptual space; see [Color spaces](#color-spaces):

    ```
    oklabFromRgb(baseColorFactor.rgb).x   # perceived lightness
    ```

12. `oklchFromRgb(c)` and `rgbFromOklch(l)` convert a vec3 between linear RGB
    and Oklch, Oklab's polar form, with hue a number ordinary arithmetic can
    shift; see [Color spaces](#color-spaces):

    ```
    mod(oklchFromRgb(tint).z + 0.1, 1)   # a tenth of a turn around
    ```

13. `pow(a, b)` is the componentwise exponent. A vec1 exponent broadcasts across
    `a`, and `pow(vec1, vecN)` errors, matching the rule for `/`:

    ```
    pow(roughnessFactor, 2.2)   # steepens the roughness curve
    ```

14. `mod(a, b)` is the floored remainder, `a - b * floor(a / b)`, the form that
    wraps. `mod(a, 0)` is non-finite and errors like any other:

    ```
    mod(hue + 0.618, 1)   # wraps back into [0, 1)
    ```

15. `clamp(x, lo, hi)` pins each component into `[lo, hi]`. A component with
    `lo > hi` errors. An explicit `clamp` names the author's bound, exactly what
    the write-time rules ask for:

    ```
    clamp(strength / 4, 0, 1)   # the author's bound
    ```

16. `lerp(a, b, t)` is `a + (b - a) * t`. `t` is unrestricted, so it
    extrapolates outside `[0, 1]`. The name says what the blend does; `mix`
    names the [bool chooser](#booleans) instead:

    ```
    lerp(orm, rgb(1, 1, 1), 0.25)   # a quarter of the way to white
    ```

17. `step(edge, x)` is 0 where `x < edge` and 1 elsewhere, the mask maker:

    ```
    step(0.001, emissiveStrength)   # 1 for every material that emits
    ```

18. `smoothstep(lo, hi, x)` is the Hermite ramp: 0 at `lo`, 1 at `hi`, held flat
    outside. A component with `lo >= hi` errors, one step stricter than `clamp`
    because the ramp divides by `hi - lo`:

    ```
    smoothstep(0.2, 0.8, occlusion)   # eases a mask edge
    ```

19. `floor(e)` and `ceil(e)` snap each component to the integer below or above,
    and `round(e)` to the nearest, halves away from zero, `f32` in and `f32`
    out; the [conversions](#numbers) round into a width:

    ```
    round(smoothness * 4) / 4   # five even levels
    ```

20. The conversions move a value between the numeric types, componentwise, under
    the exactness and rounding rules in [Numbers](#numbers):

    ```
    u8(swatchIndex)                  # the narrowed palette index
    f32(voxelPosition.y)             # unsigned into f32 math
    round_u16(occlusion * 65535.0)   # a baked 16-bit channel
    ```

21. `mix(x, y, cond)` picks per entry, `x` where the bool is false and `y` where
    it is true. The branches share a dimension or are both [strings](#strings),
    the result takes it, and the chooser is the one place a bool meets numbers:

    ```
    mix(0f32, 1, glowing)   # the deliberate 0/1 mask
    ```

22. `any(c)` and `all(c)` fold a comparison's component answers into one bool,
    `any` with or and `all` with and. The argument is a comparison written in
    place, its sides sharing a dimension or either a vec1, and the fold runs per
    entry, so an array comparison folds to a bool array; a vec1 comparison folds
    its one answer, the identity:

    ```
    all(baseColorFactor.rgb > 0.9)   # true where a color runs near white
    ```

23. `faceAvg(e)`, `faceMin(e)`, `faceMax(e)`, and `faceSum(e)` step a corner
    array down to a face array, reducing each face's four corners per component;
    any other domain errors:

    ```
    faceAvg(computedOcclusion)   # one occlusion per face
    ```

24. `voxelAvg(e)`, `voxelMin(e)`, `voxelMax(e)`, and `voxelSum(e)` step a face
    or corner array down to a voxel array, reducing each voxel's boundary per
    component; any other domain errors. A merged face reads in piecewise, so
    `voxelSum(face(1u32))` counts a voxel's faces under `greedy` and `culled`
    alike, and a buried voxel, owning no faces, takes the empty-destination rule
    in [Domains](#domains):

    ```
    voxelAvg(faceAvg(computedOcclusion))   # occlusion per voxel
    voxelSum(face(1u32))                   # a voxel's face count
    ```

25. `swatchAvg(e)`, `swatchMin(e)`, `swatchMax(e)`, and `swatchSum(e)` step a
    voxel, face, or corner array down to a swatch array, reducing each swatch's
    entries per component; a plain or swatch value errors. A material buried
    under `greedy` or `culled` owns no faces and takes the same
    empty-destination rule, guarded through the sums:

    ```
    swatchAvg(faceAvg(computedOcclusion))             # occlusion per material
    swatchSum(faceAvg(computedOcclusion)) / f32(max(swatchSum(face(1u32)), 1))
    ```

26. `swatch(e)`, `voxel(e)`, `face(e)`, and `corner(e)` lift a value to the
    named [domain](#domains), the explicit climb: the identity on a value
    already there, an error on a step down, and any type carried:

    ```
    face(albedo)   # one swatch map baked per face
    face(1u32)     # the counting seed under swatchSum
    ```

27. `default(name, fallback)` evaluates to `name` where it has a value and to
    `fallback` where it does not: a name not yet bound, a property no layer
    supplies, or a material that leaves it unset, filled per element. `name` is
    bare or backtick-quoted, and `fallback` is any expression of the same
    dimension. Nothing auto-defaults and an unbound name errors, so a robust
    expression writes the spec default itself:

    ```
    default(occlusionStrength, 1)   # the glTF default where unset
    ```

## Notes

**Backtick quoting.** Backticks quote what a bare identifier cannot hold:
spaces, a leading digit, or a reserved name. `foo bar` always lexes as two
names; the value is written `` `foo bar` ``. Double quotes make
[string](#strings) literals, never names. In a shell, single-quote an expression
holding backticks, which the shell would read as command substitution, or double
quotes, which it would strip.

**Reserved names.** The function names `r`, `rg`, `rgb`, `rgba`, `min`, `max`,
`sum`, `avg`, `any`, `all`, `abs`, `pow`, `mod`, `clamp`, `lerp`, `mix`, `step`,
`smoothstep`, `floor`, `ceil`, `round`, `f32`, `u8`, `u16`, `u32`, `ceil_u8`,
`ceil_u16`, `ceil_u32`, `floor_u8`, `floor_u16`, `floor_u32`, `round_u8`,
`round_u16`, `round_u32`, `dot`, `length`, `distance`, `normalize`, `cross`,
`oklabFromRgb`, `rgbFromOklab`, `oklchFromRgb`, `rgbFromOklch`, `faceAvg`,
`faceMin`, `faceMax`, `faceSum`, `voxelAvg`, `voxelMin`, `voxelMax`, `voxelSum`,
`swatchAvg`, `swatchMin`, `swatchMax`, `swatchSum`, `swatch`, `voxel`, `face`,
`corner`, and `default` are keywords, and the literals `true` and `false` are
reserved with them. A property sharing one is reached by backtick-quoting:
`` `min` `` is the name, `min(...)` the function.

**Swizzle rules.** A swizzle is any sequence of 1-4 components, repeats allowed,
where every component exists in the source. Two alphabets name the same
components, color `rgba` and position `xyzw`, so `v.xyz` and `v.rgb` name the
same value, whichever reads better for the data. One swizzle draws from one
alphabet, so `v.xg` errors. Existence is per component: `r`/`x` always work,
`g`/`y` need dim >= 2, `b`/`z` need dim >= 3, `a`/`w` need dim 4. Every
dimension can be swizzled, vec1 included: `s.r` is the identity, and repeats
splat upward, so `s.rr` is a vec2 and `s.rrrr` a vec4. Results can be wider or
narrower than the source: `v2.rrgg` is a vec4 and `v4.r` a vec1. With vec1
splats available, `r(s)` duplicates `s.r` and `rg(s, s)` duplicates `s.rr`; the
constructors are still needed when the arguments differ, as in `rg(u, v)`, and
they take the `rgba` names alone, one per width. The parser takes any identifier
after the dot; the checker limits the components.

```sh
baseColorFactor.rgb   # vec4 to vec3, dropping alpha
orm.g                 # one channel, roughness
0.5.rrr               # a grey vec3 splat from one number
tint.rrgg             # wider than its source
offset.xyz            # the position alphabet, the same value as .rgb
```

**Precedence and associativity.** From tightest to loosest: postfix (swizzle,
member, index), unary `-` and `!`, `* /`, `+ -`, the comparisons
`< <= > >= == !=`, `&&`, `^`, `||`. Postfixes chain left to right, so
`baseColorFactor[0].rgb` and `baseColorFactor.rgb[0]` both parse and name the
same value. Unary minus nests, so `- -x` is valid. There is no `--` token in the
expression language, so `--value` never collides with it.

**pow, not `^`.** Exponent is written `pow()`; `^` carries the [bool](#booleans)
exclusive or, never exponent.

**The function set stays small.** `sqrt(x)` is `pow(x, 0.5)`, `fract(x)` is
`mod(x, 1)`, and a signed remap is `n * 0.5 + 0.5`, so none of them is a
function. `normalize(e)` is `e / length(e)` and ships anyway, the one exception.

**Lexing.** Whitespace separates tokens and is otherwise insignificant, with two
exceptions: inside a backtick-quoted name or a string literal it is literal, and
in a postfix it is forbidden. A postfix is attached: the dot and its member hug
the source's final token, and an index bracket does the same. `v.rg` and
`tint[0]` are postfixes; `v .rg`, `v. rg`, and `tint [0]` are all errors.
Numbers use maximal munch: `1.5` is a single number token, a type suffix munches
with its digits, `2u8` one token, and `.5` where an expression is expected is a
number. Since numbers are vec1 values, literals can be swizzled too. In `2.rr`
the munch stops at `2` because `2.` followed by a letter is not a number; the
attached dot then begins a swizzle, giving a vec2 splat. `2.5.rr` works the same
way. The multi-character operators are single tokens under the same munch, `==`,
`<=`, `>=`, and `!=`, so a binding's `=` is never carved out of one.

**Linear evaluation.** `f32` math runs in linear space, and unsigned math is
exact. A color property decodes its stored form to linear on read; a non-color
property reads as written. Transfer back to sRGB happens only at a writer's
edge, per its token or its slot's fixed encoding, and quantization only where an
image is written, so a `linear` JSON file takes the numbers exactly as
evaluated.

**The transfer token lives where no slot fixes an encoding.** A
`--write-file-png-value` file is bound to nothing, a `--write-file-json-value`
key is read only by your own code, and the extras `-value` forms and
`--write-primitive-custom-value` fill destinations the format does not define,
so each names its encoding itself; the PNG token also feeds the
`--write-material-slot-file` cross-check and the file's chunks. A
`--write-material-slot-value` carries no token because its slot's fixed encoding
is the definition, not an inference:
`--write-material-slot-value baseColorTexture albedo` embeds an sRGB image by
what `baseColorTexture` is, and `COLOR_0`'s linear definition decides for
`--write-primitive-builtin-value` the same way. The `-file` forms carry none
either, the named file's chunks saying what it is.

**Why `linear`.** A writer's token names a transfer function, and `srgb` and
`linear` are the two: the sRGB curve and the identity. `raw` and `none` describe
an absence rather than naming the function, and `non-color` names authoring
intent, which reads wrong for a linear-light color.

**The PNG says what it is.** Every PNG the bake encodes stamps its transfer into
the bytes: `srgb` the `sRGB` chunk with its recommended `gAMA`/`cHRM` fallbacks,
`linear` a `gAMA` of 1.0, the nearest standard form of the identity. Color
chunks are ancillary, so a decoder that does not recognize one ignores it and
renders as it would have without; nothing fails to open, and a color-managed
viewer stops showing a `linear` map washed out. glTF ignores the chunks and
takes the encoding from the slot, so the mesh reads the same. A
`--write-material-slot-value` stamps its embedded image identically, and PNG's
`cICP` chunk, which names a linear transfer exactly, can join later without
disturbing any of this.

**Write-time errors.** A non-finite component, NaN or infinity as from `0 / 0`,
errors wherever it appears. Clamping is always the author's, written `clamp()`.
The destination decides the range, and only an image has one: a PNG requires
every component in `[0, 1]`, so `1.5` into a PNG errors while `1.5` into a JSON
field is fine, which is how an unbounded property like `emissiveStrength`
travels.

**Redefinition.** A binding may redefine any name, a property or an earlier
value. The right side evaluates against the bindings visible at that point, so
`roughnessFactor = pow(roughnessFactor, 2)` reads the property and rebinds the
name, and later expressions see the new value. There is no recursion.

## Grammar

The grammar comes in two forms. The first is dimension-typed: vec1 through vec4
expressions each get their own nonterminals, so the same-dimension rules for the
operators, the vec1 broadcast rules, and the result type of every swizzle are
encoded directly in the grammar. It encodes the dimension axis alone, leaving
the shape and numeric-type axes to checking rules. The second form is a compact
untyped grammar plus those checking rules, and it is the form to implement
because a parser cannot know a name's dimension from syntax alone.

Both forms start at a [program](#programs) of `;`-terminated bindings, the empty
statement legal, and differ only below the expression rule. A vec1 is a scalar,
named like the other vectors so swizzling and broadcasting work uniformly across
all four dimensions. The stratification encodes the [precedence](#notes): each
looser-binding operator gets an outer rule, and left recursion gives
`+ - * / && ^ ||` their left-to-right associativity.

### Dimension-typed BNF

```bnf
; ============================================================
; Start rule: a program of ;-terminated bindings; the empty
; statement makes the fragment seams legal
; ============================================================

<program>       ::= <statement>
                  | <statement> <program>

<statement>     ::= <binding> ";"
                  | ";"

<binding>       ::= <name> "=" <expr>

; ============================================================
; An expression of any dimension
; ============================================================

<expr>          ::= <vec1-expr>
                  | <vec2-expr>
                  | <vec3-expr>
                  | <vec4-expr>
                  | <bool-expr>
                  | <string-expr>

; ============================================================
; vec1 (scalar) expressions
; ============================================================

<vec1-expr>     ::= <vec1-expr> "+" <vec1-term>
                  | <vec1-expr> "-" <vec1-term>
                  | <vec1-term>

<vec1-term>     ::= <vec1-term> "*" <vec1-unary>
                  | <vec1-term> "/" <vec1-unary>
                  | <vec1-unary>

<vec1-unary>    ::= "-" <vec1-unary>
                  | <vec1-post>

; Length-1 swizzles extract a single component from any source;
; x.r on a vec1 is the identity. Indexing keeps the dimension.
<vec1-post>     ::= <vec1-prim>
                  | <vec1-post> "[" <vec1-expr> "]"
                  | <vec1-post> "." <swiz-1-of-1>
                  | <vec2-post> "." <swiz-1-of-2>
                  | <vec3-post> "." <swiz-1-of-3>
                  | <vec4-post> "." <swiz-1-of-4>

<vec1-prim>     ::= <num>
                  | <name>
                  | "(" <vec1-expr> ")"
                  | "r" "(" <vec1-expr> ")"
                  | "min" "(" <vec1-expr> ")"                 ; palette minimum
                  | "min" "(" <vec1-expr> "," <vec1-expr> ")"
                  | "max" "(" <vec1-expr> ")"                 ; palette maximum
                  | "max" "(" <vec1-expr> "," <vec1-expr> ")"
                  | "sum" "(" <vec1-expr> ")"                 ; palette sum
                  | "avg" "(" <vec1-expr> ")"                  ; palette mean
                  | "dot" "(" <vec1-expr> "," <vec1-expr> ")"  ; component fold
                  | "dot" "(" <vec2-expr> "," <vec2-expr> ")"
                  | "dot" "(" <vec3-expr> "," <vec3-expr> ")"
                  | "dot" "(" <vec4-expr> "," <vec4-expr> ")"
                  | "length" "(" <vec1-expr> ")"
                  | "length" "(" <vec2-expr> ")"
                  | "length" "(" <vec3-expr> ")"
                  | "length" "(" <vec4-expr> ")"
                  | "distance" "(" <vec1-expr> "," <vec1-expr> ")"
                  | "distance" "(" <vec2-expr> "," <vec2-expr> ")"
                  | "distance" "(" <vec3-expr> "," <vec3-expr> ")"
                  | "distance" "(" <vec4-expr> "," <vec4-expr> ")"
                  | "abs" "(" <vec1-expr> ")"
                  | "normalize" "(" <vec1-expr> ")"
                  | "pow" "(" <vec1-expr> "," <vec1-expr> ")"
                  | "mod" "(" <vec1-expr> "," <vec1-expr> ")"
                  | "clamp" "(" <vec1-expr> "," <vec1-expr> "," <vec1-expr> ")"
                  | "lerp" "(" <vec1-expr> "," <vec1-expr> "," <vec1-expr> ")"
                  | "mix" "(" <vec1-expr> "," <vec1-expr> "," <bool-expr> ")"
                  | "step" "(" <vec1-expr> "," <vec1-expr> ")"
                  | "smoothstep" "(" <vec1-expr> "," <vec1-expr> ","
                                     <vec1-expr> ")"
                  | "floor" "(" <vec1-expr> ")"
                  | "ceil" "(" <vec1-expr> ")"
                  | "round" "(" <vec1-expr> ")"
                  | <convert> "(" <vec1-expr> ")"
                  | "faceAvg" "(" <vec1-expr> ")"            ; corners to faces
                  | "faceMin" "(" <vec1-expr> ")"
                  | "faceMax" "(" <vec1-expr> ")"
                  | "faceSum" "(" <vec1-expr> ")"
                  | "voxelAvg" "(" <vec1-expr> ")"           ; down to voxels
                  | "voxelMin" "(" <vec1-expr> ")"
                  | "voxelMax" "(" <vec1-expr> ")"
                  | "voxelSum" "(" <vec1-expr> ")"
                  | "swatchAvg" "(" <vec1-expr> ")"          ; down to swatches
                  | "swatchMin" "(" <vec1-expr> ")"
                  | "swatchMax" "(" <vec1-expr> ")"
                  | "swatchSum" "(" <vec1-expr> ")"
                  | "swatch" "(" <vec1-expr> ")"             ; domain climbs
                  | "voxel" "(" <vec1-expr> ")"
                  | "face" "(" <vec1-expr> ")"
                  | "corner" "(" <vec1-expr> ")"
                  | "default" "(" <name> "," <vec1-expr> ")"

; ============================================================
; vec2 expressions
; ============================================================

<vec2-expr>     ::= <vec2-expr> "+" <vec2-term>
                  | <vec2-expr> "-" <vec2-term>
                  | <vec2-term>

<vec2-term>     ::= <vec2-term> "*" <vec2-unary>                ; vec2 * vec2
                  | <vec2-term> "*" <vec1-unary>                ; vec2 * vec1
                  | <vec1-term> "*" <vec2-unary>                ; vec1 * vec2
                  | <vec2-term> "/" <vec2-unary>                ; vec2 / vec2
                  | <vec2-term> "/" <vec1-unary>                ; vec2 / vec1
                  | <vec2-unary>

<vec2-unary>    ::= "-" <vec2-unary>
                  | <vec2-post>

; A vec2 can be swizzled out of any source, including a vec1
; splat: x.rr.
<vec2-post>     ::= <vec2-prim>
                  | <vec2-post> "[" <vec1-expr> "]"
                  | <vec1-post> "." <swiz-2-of-1>
                  | <vec2-post> "." <swiz-2-of-2>
                  | <vec3-post> "." <swiz-2-of-3>
                  | <vec4-post> "." <swiz-2-of-4>

<vec2-prim>     ::= <name>
                  | "(" <vec2-expr> ")"
                  | "rg" "(" <vec1-expr> "," <vec1-expr> ")"
                  | "min" "(" <vec2-expr> ")"
                  | "min" "(" <vec2-expr> "," <vec2-expr> ")"
                  | "min" "(" <vec2-expr> "," <vec1-expr> ")"
                  | "min" "(" <vec1-expr> "," <vec2-expr> ")"
                  | "max" "(" <vec2-expr> ")"
                  | "max" "(" <vec2-expr> "," <vec2-expr> ")"
                  | "max" "(" <vec2-expr> "," <vec1-expr> ")"
                  | "max" "(" <vec1-expr> "," <vec2-expr> ")"
                  | "sum" "(" <vec2-expr> ")"
                  | "avg" "(" <vec2-expr> ")"
                  | "abs" "(" <vec2-expr> ")"
                  | "normalize" "(" <vec2-expr> ")"
                  | "pow" "(" <vec2-expr> "," <vec2-expr> ")"
                  | "pow" "(" <vec2-expr> "," <vec1-expr> ")"
                  | "mod" "(" <vec2-expr> "," <vec2-expr> ")"
                  | "mod" "(" <vec2-expr> "," <vec1-expr> ")"
                  | "clamp" "(" <vec2-expr> "," <vec2-expr> "," <vec2-expr> ")"
                  | "clamp" "(" <vec2-expr> "," <vec1-expr> "," <vec1-expr> ")"
                  | "lerp" "(" <vec2-expr> "," <vec2-expr> "," <vec2-expr> ")"
                  | "lerp" "(" <vec2-expr> "," <vec2-expr> "," <vec1-expr> ")"
                  | "mix" "(" <vec2-expr> "," <vec2-expr> "," <bool-expr> ")"
                  | "step" "(" <vec2-expr> "," <vec2-expr> ")"
                  | "step" "(" <vec1-expr> "," <vec2-expr> ")"
                  | "smoothstep" "(" <vec2-expr> "," <vec2-expr> ","
                                     <vec2-expr> ")"
                  | "smoothstep" "(" <vec1-expr> "," <vec1-expr> ","
                                     <vec2-expr> ")"
                  | "floor" "(" <vec2-expr> ")"
                  | "ceil" "(" <vec2-expr> ")"
                  | "round" "(" <vec2-expr> ")"
                  | <convert> "(" <vec2-expr> ")"
                  | "faceAvg" "(" <vec2-expr> ")"
                  | "faceMin" "(" <vec2-expr> ")"
                  | "faceMax" "(" <vec2-expr> ")"
                  | "faceSum" "(" <vec2-expr> ")"
                  | "voxelAvg" "(" <vec2-expr> ")"
                  | "voxelMin" "(" <vec2-expr> ")"
                  | "voxelMax" "(" <vec2-expr> ")"
                  | "voxelSum" "(" <vec2-expr> ")"
                  | "swatchAvg" "(" <vec2-expr> ")"
                  | "swatchMin" "(" <vec2-expr> ")"
                  | "swatchMax" "(" <vec2-expr> ")"
                  | "swatchSum" "(" <vec2-expr> ")"
                  | "swatch" "(" <vec2-expr> ")"
                  | "voxel" "(" <vec2-expr> ")"
                  | "face" "(" <vec2-expr> ")"
                  | "corner" "(" <vec2-expr> ")"
                  | "default" "(" <name> "," <vec2-expr> ")"

; ============================================================
; vec3 expressions
; ============================================================

<vec3-expr>     ::= <vec3-expr> "+" <vec3-term>
                  | <vec3-expr> "-" <vec3-term>
                  | <vec3-term>

<vec3-term>     ::= <vec3-term> "*" <vec3-unary>
                  | <vec3-term> "*" <vec1-unary>
                  | <vec1-term> "*" <vec3-unary>
                  | <vec3-term> "/" <vec3-unary>
                  | <vec3-term> "/" <vec1-unary>
                  | <vec3-unary>

<vec3-unary>    ::= "-" <vec3-unary>
                  | <vec3-post>

<vec3-post>     ::= <vec3-prim>
                  | <vec3-post> "[" <vec1-expr> "]"
                  | <vec1-post> "." <swiz-3-of-1>
                  | <vec2-post> "." <swiz-3-of-2>
                  | <vec3-post> "." <swiz-3-of-3>
                  | <vec4-post> "." <swiz-3-of-4>

<vec3-prim>     ::= <name>
                  | "(" <vec3-expr> ")"
                  | "rgb" "(" <vec1-expr> "," <vec1-expr> "," <vec1-expr> ")"
                  | "min" "(" <vec3-expr> ")"
                  | "min" "(" <vec3-expr> "," <vec3-expr> ")"
                  | "min" "(" <vec3-expr> "," <vec1-expr> ")"
                  | "min" "(" <vec1-expr> "," <vec3-expr> ")"
                  | "max" "(" <vec3-expr> ")"
                  | "max" "(" <vec3-expr> "," <vec3-expr> ")"
                  | "max" "(" <vec3-expr> "," <vec1-expr> ")"
                  | "max" "(" <vec1-expr> "," <vec3-expr> ")"
                  | "sum" "(" <vec3-expr> ")"
                  | "avg" "(" <vec3-expr> ")"
                  | "abs" "(" <vec3-expr> ")"
                  | "normalize" "(" <vec3-expr> ")"
                  | "cross" "(" <vec3-expr> "," <vec3-expr> ")"
                  | "pow" "(" <vec3-expr> "," <vec3-expr> ")"
                  | "pow" "(" <vec3-expr> "," <vec1-expr> ")"
                  | "mod" "(" <vec3-expr> "," <vec3-expr> ")"
                  | "mod" "(" <vec3-expr> "," <vec1-expr> ")"
                  | "clamp" "(" <vec3-expr> "," <vec3-expr> "," <vec3-expr> ")"
                  | "clamp" "(" <vec3-expr> "," <vec1-expr> "," <vec1-expr> ")"
                  | "lerp" "(" <vec3-expr> "," <vec3-expr> "," <vec3-expr> ")"
                  | "lerp" "(" <vec3-expr> "," <vec3-expr> "," <vec1-expr> ")"
                  | "mix" "(" <vec3-expr> "," <vec3-expr> "," <bool-expr> ")"
                  | "step" "(" <vec3-expr> "," <vec3-expr> ")"
                  | "step" "(" <vec1-expr> "," <vec3-expr> ")"
                  | "smoothstep" "(" <vec3-expr> "," <vec3-expr> ","
                                     <vec3-expr> ")"
                  | "smoothstep" "(" <vec1-expr> "," <vec1-expr> ","
                                     <vec3-expr> ")"
                  | "floor" "(" <vec3-expr> ")"
                  | "ceil" "(" <vec3-expr> ")"
                  | "round" "(" <vec3-expr> ")"
                  | <convert> "(" <vec3-expr> ")"
                  | "faceAvg" "(" <vec3-expr> ")"
                  | "faceMin" "(" <vec3-expr> ")"
                  | "faceMax" "(" <vec3-expr> ")"
                  | "faceSum" "(" <vec3-expr> ")"
                  | "voxelAvg" "(" <vec3-expr> ")"
                  | "voxelMin" "(" <vec3-expr> ")"
                  | "voxelMax" "(" <vec3-expr> ")"
                  | "voxelSum" "(" <vec3-expr> ")"
                  | "swatchAvg" "(" <vec3-expr> ")"
                  | "swatchMin" "(" <vec3-expr> ")"
                  | "swatchMax" "(" <vec3-expr> ")"
                  | "swatchSum" "(" <vec3-expr> ")"
                  | "swatch" "(" <vec3-expr> ")"
                  | "voxel" "(" <vec3-expr> ")"
                  | "face" "(" <vec3-expr> ")"
                  | "corner" "(" <vec3-expr> ")"
                  | "oklabFromRgb" "(" <vec3-expr> ")"          ; color spaces
                  | "rgbFromOklab" "(" <vec3-expr> ")"
                  | "oklchFromRgb" "(" <vec3-expr> ")"
                  | "rgbFromOklch" "(" <vec3-expr> ")"
                  | "default" "(" <name> "," <vec3-expr> ")"

; ============================================================
; vec4 expressions
; ============================================================

<vec4-expr>     ::= <vec4-expr> "+" <vec4-term>
                  | <vec4-expr> "-" <vec4-term>
                  | <vec4-term>

<vec4-term>     ::= <vec4-term> "*" <vec4-unary>
                  | <vec4-term> "*" <vec1-unary>
                  | <vec1-term> "*" <vec4-unary>
                  | <vec4-term> "/" <vec4-unary>
                  | <vec4-term> "/" <vec1-unary>
                  | <vec4-unary>

<vec4-unary>    ::= "-" <vec4-unary>
                  | <vec4-post>

<vec4-post>     ::= <vec4-prim>
                  | <vec4-post> "[" <vec1-expr> "]"
                  | <vec1-post> "." <swiz-4-of-1>
                  | <vec2-post> "." <swiz-4-of-2>
                  | <vec3-post> "." <swiz-4-of-3>
                  | <vec4-post> "." <swiz-4-of-4>

<vec4-prim>     ::= <name>
                  | "(" <vec4-expr> ")"
                  | "rgba" "(" <vec1-expr> "," <vec1-expr> ","
                               <vec1-expr> "," <vec1-expr> ")"
                  | "min" "(" <vec4-expr> ")"
                  | "min" "(" <vec4-expr> "," <vec4-expr> ")"
                  | "min" "(" <vec4-expr> "," <vec1-expr> ")"
                  | "min" "(" <vec1-expr> "," <vec4-expr> ")"
                  | "max" "(" <vec4-expr> ")"
                  | "max" "(" <vec4-expr> "," <vec4-expr> ")"
                  | "max" "(" <vec4-expr> "," <vec1-expr> ")"
                  | "max" "(" <vec1-expr> "," <vec4-expr> ")"
                  | "sum" "(" <vec4-expr> ")"
                  | "avg" "(" <vec4-expr> ")"
                  | "abs" "(" <vec4-expr> ")"
                  | "normalize" "(" <vec4-expr> ")"
                  | "pow" "(" <vec4-expr> "," <vec4-expr> ")"
                  | "pow" "(" <vec4-expr> "," <vec1-expr> ")"
                  | "mod" "(" <vec4-expr> "," <vec4-expr> ")"
                  | "mod" "(" <vec4-expr> "," <vec1-expr> ")"
                  | "clamp" "(" <vec4-expr> "," <vec4-expr> "," <vec4-expr> ")"
                  | "clamp" "(" <vec4-expr> "," <vec1-expr> "," <vec1-expr> ")"
                  | "lerp" "(" <vec4-expr> "," <vec4-expr> "," <vec4-expr> ")"
                  | "lerp" "(" <vec4-expr> "," <vec4-expr> "," <vec1-expr> ")"
                  | "mix" "(" <vec4-expr> "," <vec4-expr> "," <bool-expr> ")"
                  | "step" "(" <vec4-expr> "," <vec4-expr> ")"
                  | "step" "(" <vec1-expr> "," <vec4-expr> ")"
                  | "smoothstep" "(" <vec4-expr> "," <vec4-expr> ","
                                     <vec4-expr> ")"
                  | "smoothstep" "(" <vec1-expr> "," <vec1-expr> ","
                                     <vec4-expr> ")"
                  | "floor" "(" <vec4-expr> ")"
                  | "ceil" "(" <vec4-expr> ")"
                  | "round" "(" <vec4-expr> ")"
                  | <convert> "(" <vec4-expr> ")"
                  | "faceAvg" "(" <vec4-expr> ")"
                  | "faceMin" "(" <vec4-expr> ")"
                  | "faceMax" "(" <vec4-expr> ")"
                  | "faceSum" "(" <vec4-expr> ")"
                  | "voxelAvg" "(" <vec4-expr> ")"
                  | "voxelMin" "(" <vec4-expr> ")"
                  | "voxelMax" "(" <vec4-expr> ")"
                  | "voxelSum" "(" <vec4-expr> ")"
                  | "swatchAvg" "(" <vec4-expr> ")"
                  | "swatchMin" "(" <vec4-expr> ")"
                  | "swatchMax" "(" <vec4-expr> ")"
                  | "swatchSum" "(" <vec4-expr> ")"
                  | "swatch" "(" <vec4-expr> ")"
                  | "voxel" "(" <vec4-expr> ")"
                  | "face" "(" <vec4-expr> ")"
                  | "corner" "(" <vec4-expr> ")"
                  | "default" "(" <name> "," <vec4-expr> ")"

; ============================================================
; bool expressions
; ============================================================

<bool-expr>     ::= <bool-expr> "||" <bool-xor>
                  | <bool-xor>

<bool-xor>      ::= <bool-xor> "^" <bool-and>
                  | <bool-and>

<bool-and>      ::= <bool-and> "&&" <bool-unary>
                  | <bool-unary>

<bool-unary>    ::= "!" <bool-unary>
                  | <bool-post>

; Indexing samples a bool array at an entry. Bools have no
; components, so there is no bool swizzle.
<bool-post>     ::= <bool-prim>
                  | <bool-post> "[" <vec1-expr> "]"

<bool-prim>     ::= <name>
                  | "true"
                  | "false"
                  | "(" <bool-expr> ")"
                  | <vec1-expr> <cmp-op> <vec1-expr>
                  | <string-expr> <eq-op> <string-expr>   ; equality alone
                  | "any" "(" <comparison> ")"        ; or-fold of components
                  | "all" "(" <comparison> ")"        ; and-fold of components
                  | "swatch" "(" <bool-expr> ")"      ; domain climbs
                  | "voxel" "(" <bool-expr> ")"
                  | "face" "(" <bool-expr> ")"
                  | "corner" "(" <bool-expr> ")"

; A comparison wider than vec1 lives only here, directly inside its
; reduction, so the component answers never escape as a value.
<comparison>    ::= <vec1-expr> <cmp-op> <vec1-expr>
                  | <vec2-expr> <cmp-op> <vec2-expr>
                  | <vec2-expr> <cmp-op> <vec1-expr>
                  | <vec1-expr> <cmp-op> <vec2-expr>
                  | <vec3-expr> <cmp-op> <vec3-expr>
                  | <vec3-expr> <cmp-op> <vec1-expr>
                  | <vec1-expr> <cmp-op> <vec3-expr>
                  | <vec4-expr> <cmp-op> <vec4-expr>
                  | <vec4-expr> <cmp-op> <vec1-expr>
                  | <vec1-expr> <cmp-op> <vec4-expr>

<cmp-op>        ::= "<" | "<=" | ">" | ">=" | "==" | "!="

<eq-op>         ::= "==" | "!="

; ============================================================
; string expressions
; ============================================================

; Strings take no operators; postfix indexing samples an array
; entry, and equality lives in <bool-prim>.
<string-expr>   ::= <string-post>

<string-post>   ::= <string-prim>
                  | <string-post> "[" <vec1-expr> "]"

<string-prim>   ::= <name>
                  | <string-lit>
                  | "(" <string-expr> ")"
                  | "mix" "(" <string-expr> "," <string-expr> ","
                             <bool-expr> ")"
                  | "swatch" "(" <string-expr> ")"    ; domain climbs
                  | "voxel" "(" <string-expr> ")"
                  | "face" "(" <string-expr> ")"
                  | "corner" "(" <string-expr> ")"
                  | "default" "(" <name> "," <string-expr> ")"

; ============================================================
; Swizzle selectors
; Any sequence of 1-4 components valid for the source; repeats
; allowed, and the result may be wider or narrower than the
; source. Two alphabets name the same components, color rgba
; and position xyzw; a selector draws from one alphabet.
; Selector counts (lengths 1-4): 8 from vec1, 60 from vec2,
; 240 from vec3, 680 from vec4.
; ============================================================

<c1>            ::= "r"                       ; color components of a vec1
<c2>            ::= "r" | "g"                 ; color components of a vec2
<c3>            ::= "r" | "g" | "b"           ; color components of a vec3
<c4>            ::= "r" | "g" | "b" | "a"     ; color components of a vec4

<p1>            ::= "x"                       ; position components of a vec1
<p2>            ::= "x" | "y"                 ; position components of a vec2
<p3>            ::= "x" | "y" | "z"           ; position components of a vec3
<p4>            ::= "x" | "y" | "z" | "w"     ; position components of a vec4

<swiz-1-of-1>   ::= <c1> | <p1>
<swiz-2-of-1>   ::= <c1> <c1> | <p1> <p1>
<swiz-3-of-1>   ::= <c1> <c1> <c1> | <p1> <p1> <p1>
<swiz-4-of-1>   ::= <c1> <c1> <c1> <c1> | <p1> <p1> <p1> <p1>

<swiz-1-of-2>   ::= <c2> | <p2>
<swiz-2-of-2>   ::= <c2> <c2> | <p2> <p2>
<swiz-3-of-2>   ::= <c2> <c2> <c2> | <p2> <p2> <p2>
<swiz-4-of-2>   ::= <c2> <c2> <c2> <c2> | <p2> <p2> <p2> <p2>

<swiz-1-of-3>   ::= <c3> | <p3>
<swiz-2-of-3>   ::= <c3> <c3> | <p3> <p3>
<swiz-3-of-3>   ::= <c3> <c3> <c3> | <p3> <p3> <p3>
<swiz-4-of-3>   ::= <c3> <c3> <c3> <c3> | <p3> <p3> <p3> <p3>

<swiz-1-of-4>   ::= <c4> | <p4>
<swiz-2-of-4>   ::= <c4> <c4> | <p4> <p4>
<swiz-3-of-4>   ::= <c4> <c4> <c4> | <p4> <p4> <p4>
<swiz-4-of-4>   ::= <c4> <c4> <c4> <c4> | <p4> <p4> <p4> <p4>

; ============================================================
; Lexical grammar
; Whitespace separates tokens and is otherwise insignificant,
; except inside a backtick-quoted name or string literal
; (literal) and around the postfix dot and index bracket
; (forbidden). Not modeled below.
; ============================================================

; A conversion names its target type, the rounding forms their mode
; too; see Numbers.
<convert>       ::= "f32" | "u8" | "u16" | "u32"
                  | "ceil_u8" | "ceil_u16" | "ceil_u32"
                  | "floor_u8" | "floor_u16" | "floor_u32"
                  | "round_u8" | "round_u16" | "round_u32"

; A suffix pins a literal's type; a decimal point makes an f32.
<num>           ::= <digits>
                  | <digits> <num-suffix>
                  | <digits> "."
                  | <digits> "." <digits>
                  | "." <digits>

<num-suffix>    ::= "f32" | "u8" | "u16" | "u32"

<digits>        ::= <digit>
                  | <digit> <digits>

<digit>         ::= "0" | "1" | "2" | "3" | "4"
                  | "5" | "6" | "7" | "8" | "9"

; Bare identifiers start with a letter or underscore so they can
; never be confused with <num>; backtick-quote a name to allow
; spaces, a leading digit, or a reserved name.
<name>        ::= <ident>
                  | "`" <quoted-chars> "`"

<ident>         ::= <ident-start>
                  | <ident-start> <ident-rest>
<ident-start>   ::= <letter> | "_"
<ident-rest>    ::= <ident-char>
                  | <ident-char> <ident-rest>
<ident-char>    ::= <letter> | <digit> | "_"

<letter>        ::= "a" | "b" | ... | "z"
                  | "A" | "B" | ... | "Z"          ; informal shorthand

<quoted-chars>  ::= any sequence of characters other than "`"   ; informal

<string-lit>    ::= '"' <string-chars> '"'

<string-chars>  ::= any sequence of characters other than '"'   ; informal
```

### Untyped grammar + checking rules (implementation form)

A `<name>` and a `default(...)` can have any dimension, so the typed grammar
reads ambiguously to a parser that does not know what each name holds. The
implementation parses with the untyped grammar below, then checks the tree
against the dimension, shape, and numeric rules that follow.

```bnf
<u-program>   ::= <u-statement>
                | <u-statement> <u-program>

<u-statement> ::= <u-binding> ";"
                | ";"

<u-binding>   ::= <name> "=" <u-expr>

<u-expr>      ::= <u-expr> "||" <u-xor>
                | <u-xor>

<u-xor>       ::= <u-xor> "^" <u-and>
                | <u-and>

<u-and>       ::= <u-and> "&&" <u-cmp>
                | <u-cmp>

; A chain like a < b < c parses; the checker rejects it because
; the left comparison feeds a bool operand to the right one.
<u-cmp>       ::= <u-cmp> <cmp-op> <u-add>
                | <u-add>

<cmp-op>      ::= "<" | "<=" | ">" | ">=" | "==" | "!="

<u-add>       ::= <u-add> "+" <u-term>
                | <u-add> "-" <u-term>
                | <u-term>

<u-term>      ::= <u-term> "*" <u-unary>
                | <u-term> "/" <u-unary>
                | <u-unary>

<u-unary>     ::= "-" <u-unary>
                | "!" <u-unary>
                | <u-post>

<u-post>      ::= <u-post> "." <member>
                | <u-post> "[" <u-expr> "]"
                | <u-prim>

<u-prim>      ::= <num>
                | "true"
                | "false"
                | <string-lit>
                | <name>
                | "(" <u-expr> ")"
                | "r"    "(" <u-expr> ")"
                | "rg"   "(" <u-expr> "," <u-expr> ")"
                | "rgb"  "(" <u-expr> "," <u-expr> "," <u-expr> ")"
                | "rgba" "(" <u-expr> "," <u-expr> "," <u-expr> "," <u-expr> ")"
                | "min"  "(" <u-expr> ")"
                | "min"  "(" <u-expr> "," <u-expr> ")"
                | "max"  "(" <u-expr> ")"
                | "max"  "(" <u-expr> "," <u-expr> ")"
                | "sum"  "(" <u-expr> ")"
                | "avg"  "(" <u-expr> ")"
                | "any"  "(" <u-expr> ")"
                | "all"  "(" <u-expr> ")"
                | "abs"  "(" <u-expr> ")"
                | "pow"  "(" <u-expr> "," <u-expr> ")"
                | "mod"  "(" <u-expr> "," <u-expr> ")"
                | "clamp" "(" <u-expr> "," <u-expr> "," <u-expr> ")"
                | "lerp" "(" <u-expr> "," <u-expr> "," <u-expr> ")"
                | "mix"  "(" <u-expr> "," <u-expr> "," <u-expr> ")"
                | "step" "(" <u-expr> "," <u-expr> ")"
                | "smoothstep" "(" <u-expr> "," <u-expr> "," <u-expr> ")"
                | "floor" "(" <u-expr> ")"
                | "ceil" "(" <u-expr> ")"
                | "round" "(" <u-expr> ")"
                | <convert> "(" <u-expr> ")"
                | "dot" "(" <u-expr> "," <u-expr> ")"
                | "length" "(" <u-expr> ")"
                | "distance" "(" <u-expr> "," <u-expr> ")"
                | "normalize" "(" <u-expr> ")"
                | "cross" "(" <u-expr> "," <u-expr> ")"
                | "oklabFromRgb" "(" <u-expr> ")"
                | "rgbFromOklab" "(" <u-expr> ")"
                | "oklchFromRgb" "(" <u-expr> ")"
                | "rgbFromOklch" "(" <u-expr> ")"
                | "faceAvg" "(" <u-expr> ")"
                | "faceMin" "(" <u-expr> ")"
                | "faceMax" "(" <u-expr> ")"
                | "faceSum" "(" <u-expr> ")"
                | "voxelAvg" "(" <u-expr> ")"
                | "voxelMin" "(" <u-expr> ")"
                | "voxelMax" "(" <u-expr> ")"
                | "voxelSum" "(" <u-expr> ")"
                | "swatchAvg" "(" <u-expr> ")"
                | "swatchMin" "(" <u-expr> ")"
                | "swatchMax" "(" <u-expr> ")"
                | "swatchSum" "(" <u-expr> ")"
                | "swatch" "(" <u-expr> ")"
                | "voxel" "(" <u-expr> ")"
                | "face" "(" <u-expr> ")"
                | "corner" "(" <u-expr> ")"
                | "default" "(" <name> "," <u-expr> ")"

; A member is always a swizzle: 1-4 components over one alphabet,
; {r,g,b,a} or {x,y,z,w}, repeats allowed. No other member exists,
; and the checker limits the components.
<member>      ::= <ident>
```

Dimension rules for an expression `e` with dimension written `dim(e)`:

- `a + b`, `a - b`: `dim(a) = dim(b)`; result `dim(a)`.
- `a * b`, `a / b`: `dim(a) = dim(b)`, or `dim(b) = 1`, for `*` also
  `dim(a) = 1`; result the larger of the two.
- `-e`: any dimension; result `dim(e)`.
- `r(x)`, `rg(x, y)`, `rgb(x, y, z)`, `rgba(x, y, z, w)`: every argument
  dimension 1; result 1 through 4.
- `e.s`, a swizzle: one alphabet, `rgba` or `xyzw`; every component exists in
  `dim(e)`: `r`/`x` always, `g`/`y` needs >= 2, `b`/`z` needs >= 3, `a`/`w`
  needs 4; `1 <= len(s) <= 4`, repeats allowed; result `len(s)`.
- `e[i]`: `dim(i) = 1`; result `dim(e)`.
- `min(e)`, `max(e)`: any dimension; result `dim(e)`.
- `min(a, b)`, `max(a, b)`: `dim(a) = dim(b)`, or either `= 1`; result the
  larger of the two.
- `sum(e)`, `avg(e)`: any dimension; result `dim(e)`.
- `abs(e)`, `round(e)`: any dimension; result `dim(e)`.
- `pow(a, b)`, `mod(a, b)`: `dim(a) = dim(b)`, or `dim(b) = 1`; result `dim(a)`.
- `clamp(x, lo, hi)`: `dim(lo) = dim(hi)`, equal to `dim(x)` or 1; result
  `dim(x)`.
- `lerp(a, b, t)`: `dim(a) = dim(b)`; `dim(t) = dim(a)` or 1; result `dim(a)`.
- `step(edge, x)`: `dim(edge) = dim(x)`, or `dim(edge) = 1`; result `dim(x)`.
- `smoothstep(lo, hi, x)`: `dim(lo) = dim(hi)`, equal to `dim(x)` or 1; result
  `dim(x)`.
- `floor(e)`, `ceil(e)`: any dimension; result `dim(e)`.
- a conversion, `f32(e)` through `round_u32(e)`: any dimension; result `dim(e)`.
- `<num>`: result 1.
- `true`, `false`: result bool.
- `<name>`: result the dimension of the value it names.
- `default(name, e)`: `dim(name) = dim(e)` where `name` has a value; result
  `dim(e)`.
- `a < b`, `a <= b`, `a > b`, `a >= b`, `a == b`, `a != b`:
  `dim(a) = dim(b) = 1`, numbers on both sides, wider sides only inside
  `any`/`all`; result bool.
- `any(c)`, `all(c)`: `c` a comparison, `dim(a) = dim(b)` or either `= 1`;
  result bool.
- `!e`: `e` bool; result bool.
- `a && b`, `a ^ b`, `a || b`: both bool; result bool.
- `mix(a, b, c)`: `dim(a) = dim(b)`, or both strings; `c` bool; result `dim(a)`.
- `faceAvg(e)`, `faceMin(e)`, `faceMax(e)`, `faceSum(e)`: any dimension; result
  `dim(e)`.
- `voxelAvg(e)`, `voxelMin(e)`, `voxelMax(e)`, `voxelSum(e)`: any dimension;
  result `dim(e)`.
- `swatchAvg(e)`, `swatchMin(e)`, `swatchMax(e)`, `swatchSum(e)`: any dimension;
  result `dim(e)`.
- `swatch(e)`, `voxel(e)`, `face(e)`, `corner(e)`: any dimension, or bool, or
  string; result `dim(e)`, a bool or string keeping its type.
- `dot(a, b)`: `dim(a) = dim(b)`; result 1.
- `length(e)`: any dimension; result 1.
- `distance(a, b)`: `dim(a) = dim(b)`; result 1.
- `normalize(e)`: any dimension; result `dim(e)`.
- `cross(a, b)`: `dim(a) = dim(b) = 3`; result 3.
- the color conversions: `dim(c) = 3`; result 3.
- `<string-lit>`: result string.
- string `==`, `!=`: both sides strings; result bool.

A vec1 broadcasts on the right of `/`, as the exponent of `pow`, on either side
of `*`, and on either side of the comparison inside `any`/`all`. `dot`,
`distance`, and `cross` take no broadcast: their sides share one dimension
exactly.

Shape rules, with a value either plain or an array over the effective palette
(see [Shapes](#shapes)):

1. A property name is an array; a literal is plain; a bound name has its
   definition's shape.
2. The elementwise constructs, the operators, the constructors, swizzles, binary
   `min`/`max`, `abs`, `pow`, `mod`, `clamp`, `lerp`, `step`, `smoothstep`,
   `floor`/`ceil`/`round`, `default()`, `dot`, `length`, `distance`,
   `normalize`, `cross`, and the color conversions, pair arrays element by
   element and broadcast plain values; the result is an array when any operand
   is. `dot`, `length`, and `distance` fold the components inside each entry,
   never entries across the domain, so they reduce dimension to 1, not shape to
   plain.
3. The reductions, unary `min`/`max` and `sum`/`avg`, require an array and yield
   a plain value, computed per component across its whole domain.
4. `e[i]` requires `e` an array and `i` a plain exact non-negative integer below
   the array's entry count, and yields a plain value.
5. A writer takes whatever shape its destination holds: a PNG a swatch, voxel,
   face, or corner array, a `--write-material-slot-value` factor a plain value,
   JSON either shape.
6. The comparisons and logical operators follow rule 2 over their bool results,
   `any`/`all` fold each entry's component answers and keep the comparison's
   shape, and `e[i]` on a bool array follows rule 4. A bool reaches
   `--write-file-json-value` under `linear`, `--primitive`'s select, a boolean
   material property, and `mix`'s chooser; nothing else takes one.
7. An array carries its [domain](#domains), and rule 2 pairs entries after the
   lower domain climbs onto the higher; the climb is implicit, with
   `swatch`/`voxel`/`face`/`corner` naming it, and a step down always takes a
   reduction naming its destination and any array above it, or a reduction to
   plain, never an implicit one.
8. `mix(a, b, cond)` follows rule 2 across `a`, `b`, and `cond`, the result's
   shape from any operand array.
9. A string rides the same axes: a literal is plain, a string property an array,
   its `==`/`!=` and `mix` pair entries by rule 2, `e[i]` follows rule 4, and a
   string reaches an enum property plain, a JSON destination in either shape,
   and nothing else.

Numeric type rules, with every number an `f32`, `u8`, `u16`, or `u32` (see
[Numbers](#numbers)):

1. Every operator, comparison, and function takes one numeric type across its
   numeric operands; nothing converts implicitly.
2. A whole-number literal takes the type its context fixes, a suffix or a
   decimal point fixes one directly, and a literal nothing types errors.
3. The conversions are `f32(e)`, `u8(e)`, `u16(e)`, `u32(e)`, and the
   `ceil_`/`floor_`/`round_` forms, componentwise. The named modes round; every
   other conversion is exact or errors, a fraction, a range overflow, and a
   `u32` beyond `f32`'s exact range each named.
4. Unsigned `+`, `-`, `*` error on overflow and below zero, `/` floors with the
   floored `mod` completing it, and unary `-` takes `f32` alone.
5. `min`, `max`, `sum`, and the `Min`/`Max`/`Sum` reductions keep the operand
   type; `avg` and the `Avg` reductions return `f32`.
6. `floor`, `ceil`, and `round` keep `f32`.
7. `e[i]` takes any unsigned index.
8. Every function not named above takes `f32` alone on its numeric arguments.
