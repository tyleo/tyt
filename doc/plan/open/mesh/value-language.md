# Value language

_Part of the [mesh plan](README.md)._

The expression language behind `vxl mesh`'s material values. `--value`
defines vector-valued names over the effective palette, and the writer
and slot flags listed in [`vxl mesh`](mesh.md#options) take expressions
of their own, a defined name the simplest, putting the results in
images, JSON files, and the mesh's own material.

Mesh generates materials for a mesh based on the object's layers. The
combination of layers results in an "effective palette", each property read
through the last layer whose palette supplies its name; see
[The palette atlas](mesh.md#the-palette-atlas). Each property of the
effective palette can be used as a value in the final object's materials.

## Shapes

A value is plain or an array, independent of its vec1-vec4 dimension. A
property is an array: one element per distinct flattened material of the
effective palette, in first-seen raster order, the order the atlas lays its
texels out and every [mesh palette](mesh.md#palettes) lists its
rows. A numeric
literal is plain. Elementwise operations pair arrays element by element and
broadcast a plain value across an array, so `1 - roughnessFactor` is an
array; two arrays of one domain always align, and mixed domains climb the
[ladder](#domains).

```
--value cutoff "0.4"                      # plain vec1, a literal
--value tint "baseColorFactor.rgb"        # array vec3, one element per material
--value bright "tint * 1.2"               # array * plain broadcasts
--value mask "step(0.5, metallicFactor)"  # 1 where a material is metal
```

`max(e)`, `min(e)`, `sum(e)`, and `avg(e)` reduce an array across the
palette, per component, to a plain value; the binary `min`/`max` are
elementwise like the operators. The emissive bake is the canonical use:

```
--value emissive "emissiveFactor * emissiveStrength / max(emissiveStrength)"
```

Each material's emissive color, scaled into `0..1` of the palette's strongest
strength. An all-zero palette divides `0 / 0`, an error; guard with
`max(max(emissiveStrength), 0.001)`.

`e[i]` samples an array at entry index `i`, a plain result: `tint[0]` is
the first material's tint. The index is a plain vec1 holding an exact
non-negative integer below the array's entry count; a fraction, a negative,
an out-of-range index, or an array index (a gather) errors. Indexing and
swizzling commute: `baseColorFactor[0].rgb` and `baseColorFactor.rgb[0]` are
the same value.

A PNG writes a [row or face](#domains) array, one texel per entry; a plain
value has no texels and cannot be written to a PNG, and a corner array
steps down through its reductions first. In JSON an array field writes as a
JSON array and a plain field as a single literal.

The plain-or-array split is half an axis: an array also has a
[domain](#domains) saying what its entries run over, the palette rows here
and the mesh's faces or corners elsewhere. A [bool](#booleans) value shares
the axis, though it is no vector: it has no components and takes no
swizzle. A [string](#strings) rides the axes the same way, no vector
either, the type an enum slot and a palette tag speak.

## Domains

A value's domain is what it has one entry per. There are four. A plain
value has one entry. A row value has one entry per distinct flattened
material of the effective palette, the arrays every property starts as. A
face value has one entry per face the mesher emits. A corner value has one
entry per face corner, exactly four per face, and
[computed occlusion](#computed-occlusion) is the domain's producer. Domain
is orthogonal to dimension: `albedo` is a row vec3, occlusion a corner
vec1.

```
# a four-row palette meshed into ten faces
plain    1 entry
row      4 entries    # one per palette row
face    10 entries    # one per face
corner  40 entries    # four corners per face
```

The domains ladder, plain to row to face to corner, and every step up is a
lossless duplication: a plain value broadcasts everywhere, a row value
reads per face through the face's own row, and a face value duplicates onto
its four corners. Climbing is implicit because nothing is lost, the same
rule that broadcasts a scalar across an array, so `albedo * ao` is legal,
`albedo` climbing to the corners before the multiply pairs entries.

A step down loses entries, so it is spelled. `faceAverage(e)`,
`faceMin(e)`, and `faceMax(e)` take a corner array to a face array, each
face's four corners reduced per component, and the unary reductions,
`min`/`max`/`sum`/`avg`, take any array to plain across its whole domain.
A corner value meeting a face destination without a reduction is an error,
never an implicit average.

The destinations read by domain. A texture holds one texel per entry, so
it takes a row or face array, and which UV stream it samples through
derives from which; see [UV streams](mesh.md#uv-streams). A select reads
at the faces, plain and row bools climbing in; see
[Primitives and materials](mesh.md#primitives-and-materials). The
[vertex attributes](#vertex-attributes) live on the corners, the ladder's
top, so any domain climbs in, a corner value landing exactly. A material
factor is one number, plain alone.

## Booleans

A comparison makes a bool: `<`, `<=`, `>`, `>=`, `==`, and `!=` take
a vec1 on each side and yield one, and the literals `true` and
`false` spell one directly, plain, the two names reserved, a
colliding name backtick-quoted. `==` and `!=` also compare two
[strings](#strings) by value. A wider comparison spells its
fold: inside `any(c)` and `all(c)` the sides share a dimension or
either is a vec1, broadcasting as it does through `*`, the
components compare one by one, and the reduction folds the answers,
`any` with or and `all` with and, so
`all(baseColorFactor.rgb > 0.9)` is true where a color runs near
white. The comparison is legal only directly inside its reduction,
so a bare `vec3 < vec3` stays an error, naming no fold, and the
component answers are never a value: no bool vector exists, and the
reductions take a comparison spelled in place, never a stored bool.
`!`, `&&`, `^`, and `||`
combine bools, `^` the exclusive or. Nothing else touches the type:
a bool never mixes with a number, so there is no `0`/`1` coercion,
`rgb(glowing, 0, 0)` errors, arithmetic on a bool errors, and every
function but one rejects one. The one is `mix(x, y, cond)`, the
spelled bridge out: it picks `x` or `y` per entry by the bool, so
`mix(0, 1, glowing)` is the deliberate `0`/`1` mask; see
[Functions](#functions). Beyond it only grouping parentheses and
`e[i]` apply, sampling a bool array at an entry. The type reaches
three destinations: the select of
[`--primitive`](mesh.md#primitives-and-materials), which reads a
bool at the face domain, lower domains climbing the ladder,
`--write-file-json-value`, where a bool writes its own JSON form,
and a boolean [material property](#material-slots), plain alone;
see [JSON files](#json-files):

```
--value glowing "emissiveStrength > 0"   # bool array, one entry per material
--value solid "!glowing"
--material-count 2
--primitive 0 solid
--primitive 1 glowing
```

Shape follows the numeric rules: a comparison against a plain value
broadcasts across an array, two arrays pair element by element, the
logical operators do the same, and `solid && metallic` is an array
wherever either side is. The comparisons bind looser than the
numeric operators and the logical operators looser still, `!`
excepted, so `a + 1 > b && c > d` reads as `((a + 1) > b) && (c > d)`;
see the [precedence note](#notes). `==` and `!=` compare floats
exactly, right against an authored palette property and surprising
against a computed value, where `0.1 + 0.2 == 0.3` is false.

## Strings

A string literal is double-quoted, `"MASK"`, the reserved quotes
taking the job they were held for, and a string palette property,
the voxj `string` kind, is an array like any property, one entry
per flattened material. The type has the bool's footprint: no
components, no swizzle, no arithmetic, and no coercion, a string
never meeting a number or a bool. A literal is any characters
except the quote, no escapes, the backtick-name rule again.

Four operations touch the type. `==` and `!=` compare two strings
into a bool, entry by entry, a plain side broadcasting across an
array, and no other comparison applies, strings holding no order.
`mix(x, y, cond)` picks between two strings by a bool, the bridge
numbers already have. `e[i]` samples a string array at an entry.
`default(name, fallback)` fills a string hole. The equality is the
routing tool, an authored tag turning into a select:

```
--value glass 'tag == "glass"'
--value solid '!glass'
--material-count 2
--primitive 0 solid
--primitive 1 glass
```

A string reaches three destinations, each under `linear` alone,
the identity token the bool takes: an enum material property, a
JSON file value, and a JSON extras entry, the JSON forms writing
the quoted string itself, so
`--write-mesh-extra-json-value tag tag linear` lands
`["glass", "steel"]` beside a palette index. Every numeric
destination, a PNG, a texture, a vertex attribute, a factor,
rejects a string.

An enum property takes one word from the fixed list its format's
schema spells, glTF's `alphaMode` taking `OPAQUE`, `MASK`, or
`BLEND`. The property reads a plain string, and the writer checks
the value against the list at the edge, an unknown token erroring
with the format named, the unknown-slot rule again; no conversion
exists in the language, only the destination knowing the list:

```
# static: cutout mode, spelled
--write-material-slot-value 0 alphaMode '"MASK"'

# computed: cutout only where the palette holds transparency
--value mode 'mix("OPAQUE", "MASK", min(baseColorFactor.a) < 1)'
--write-material-slot-value 0 alphaMode mode
```

In a shell, single quotes carry the inner double quotes through,
the backtick advice again.

## Color spaces

Every expression evaluates in linear RGB, and the conversion
functions visit other spaces as plain vec3 math: the language never
tracks which space a vec3 sits in, the author does, the same trust
the transfer tokens extend. Linear RGB is the hub, every space
converting to and from it, so a hop between two others is two
calls. The constructor names spell dimension, not meaning,
`rgb(...)` assembling an Oklab triple as readily as a color, and
components read best through the position alphabet, `lab.x` rather
than `lab.r`.

`oklabFromRgb(c)` and `rgbFromOklab(l)` visit Oklab, the perceptual
space: equal numeric steps look like equal visual steps, where
linear RGB crowds the distinguishable dark shades into a sliver of
its range. Oklab is defined from linear sRGB, so the language's
native form is exactly its input, the conversion two fixed matrices
around a cube root and the inverse the same steps backward.
`distance` there measures how different two colors look, `.x` is
perceived lightness, 0 black to 1 white, `.y` runs green to red,
and `.z` blue to yellow.

```
--value lab "oklabFromRgb(baseColorFactor.rgb)"
--value reddish "distance(lab, oklabFromRgb(rgb(1, 0, 0))) < 0.25"
--value darker "rgbFromOklab(lab * rgb(0.8, 1, 1))"   # dimmed, hue held
```

`oklchFromRgb(c)` and `rgbFromOklch(l)` visit Oklch, Oklab's polar
form: `.x` the same lightness, `.y` chroma, 0 at gray and rising
with colorfulness, `.z` hue. Hue as a plain number is the form's
power: `mod(lch.z + 0.1, 1)` turns every material a tenth of the
way around the wheel with lightness and chroma held.

Hue is a turn in `0..1`, and a gray has none, `oklchFromRgb`
answering hue 0 at zero chroma. `rgbFromOklch` errors on a hue
outside `0..1`, the wrap staying the author's own `mod(h, 1)`, and
on a negative chroma. A converted-back color can leave the gamut,
components outside `0..1`, and no conversion clamps: an image
writer already errors there, and the bound stays the author's
`clamp`.

## JSON files

`--write-file-json-value <dst-file> <dst-name> <src-expr> <linear | srgb>`
writes one value under the name as its key. Repeating it on one
path merges, so a file with several values is several flags rather
than a grouping construct in the language:

```
--write-file-json-value turret-pbr.json albedo albedo linear
--write-file-json-value turret-pbr.json orm orm linear
--write-file-json-value turret-pbr.json emissive emissive linear
```

```jsonc
{
  "albedo": [
    [1, 0, 0, 1],
    [0, 0, 1, 1]
  ],
  "orm": [
    [1, 0.9, 0],
    [1, 0.1, 1]
  ],
  "emissive": [
    [0, 0, 0],
    [0.5, 0.5, 0]
  ]
}
```

Five rules cover it:

1. The key is the flag's own `<dst-name>`, named at the destination the
   way every extras entry is.
2. Repeating the flag on one path merges into that file, in flag order.
3. The same name twice into one file is an error, since one key would
   silently win.
4. Two writers of different kinds on one path is an error, the same rule that
   rejects two images resolving to one file name.
5. The output is always an object, so one value and five produce the same
   shape.

The values themselves come out one number per component, nested in an array
per material when the value is one. A vec1 writes `0.4`, a vec4 writes
`[1, 0, 0, 1]`, and either over the palette writes an array of those. The
token names the transfer those numbers take: `linear` writes them as
evaluated, and `srgb` transfer-encodes them under the image rules, so an
alpha component stays linear and a component outside `0..1` errors. Both
write full floats, so an `srgb` JSON is display-encoded floats where an
`srgb` PNG is display-encoded bytes. The token rides each flag, so one file
mixes encodings the way one glb does, each key taking its own. Encoding
stays at the flag either way, so nothing about the destination appears in
an expression.

A bool writes as itself: `true` or `false`, an array of them per entry. Its
token is `linear`, the identity, no transfer applied, which is already
what `linear` means for a number; `srgb` on a bool errors, a transfer
curve being a number's to take. The `0`/`1` form a runtime might want
instead is spelled, `mix(0, 1, glowing)`. A [string](#strings) writes
its quoted JSON form the same way, `linear` its only token.

Merging at the flag is what keeps every value simple: a vector or a bool,
never a grouping, so a dot postfix is always a swizzle and the checker
asks only shape and dimension. Nesting was dropped with it: a
[mesh palette](mesh.md#palettes) is a fixed shape the exporter
builds.

This mirrors the voxel-json
[value kinds](../../closed/voxj-value-kinds/README.md), which deleted their six
color kinds for the same reason. A color has two spellings and two transfers,
which is four ways to write one value, and none of the four is a property of
the value itself. Fix the spelling and drop the transfer and what is left is a
bounded array of numbers, which is what glTF's own schema calls a color.
voxj stores one form, linear light; the language evaluates in that form and
converts only where something outside has an opinion.

A written file is never a voxel-json value pool either. Pools are pure
value-shapes, a kind and its values with no ranges, so if emitting one is
ever wanted, the kind falls out of dimension and a property's range stays
the consumer's own vocabulary check, the place the voxj plan puts it.

That boundary is the writer, not the field.
`--write-file-png-value turret.png albedo srgb` names the transfer
its file takes, and a `--write-material-slot-value` encodes to its
slot's fixed requirement, because an image is what a renderer
decodes. A JSON file has no format fixing a contract, so its token is
you declaring one: `linear` is the plain export, and `srgb`
serves a reader that takes its colors display-encoded.

## Material slots

`--write-material-slot-value <material-index> <dst-property>
<src-expr>` sets one property of the indexed material, destination
before source like every writer, the index riding first: which
material, then what on it. The examples write material `0`, a
`--material-count 1` ahead of them declaring it. The property is
the target format's own name, the leaf of its
material schema, so the flag invents no vocabulary and the writer
does the nesting and the `extensionsUsed` bookkeeping:

```
--write-material-slot-value 0 baseColorTexture albedo         # pbrMetallicRoughness.baseColorTexture
--write-material-slot-value 0 metallicRoughnessTexture orm    # pbrMetallicRoughness.metallicRoughnessTexture
--write-material-slot-value 0 occlusionTexture orm            # occlusionTexture, sharing one image
--write-material-slot-value 0 emissiveTexture emissive        # emissiveTexture
--write-material-slot-value 0 emissiveStrength maxStrength    # extensions.KHR_materials_emissive_strength
--write-material-slot-value 0 ior glassIor                    # extensions.KHR_materials_ior
```

The vocabulary comes from the resolved output format, since one run writes
one mesh file and `--to` has already chosen it. Each format brings its own
names: FBX calls its slots `DiffuseColor` and `NormalMap`, MTL calls them
`map_Kd` and `map_Pr`, and glTF packs roughness into
`metallicRoughnessTexture`, which a neutral `roughness` slot could not
honestly target. The vocabularies do not overlap, so retargeting a script
from `--to gltf` to `--to fbx` makes every slot name unknown at once and
errors loudly. The legal names depend on the format, so the error names
the format it checked against.

The property's own type decides how its expression reads:

| Property type    | Argument                                                                 |
| ---------------- | ------------------------------------------------------------------------ |
| `*Texture`       | an array expression to embed, or a file via `--write-material-slot-file` |
| number or vector | a plain expression of that dimension                                     |
| boolean          | a plain bool expression                                                  |
| enum             | a plain string expression, one of the property's tokens                  |

```
--write-material-slot-value 0 baseColorTexture albedo    # array value, embedded
--write-material-slot-value 0 alphaCutoff "cutoff / 2"   # plain vec1 expression
--write-material-slot-value 0 doubleSided true           # plain bool
--write-material-slot-value 0 alphaMode '"MASK"'         # enum, a plain string
--write-material-slot-file 0 baseColorTexture skin.png   # existing file, referenced
```

A property that is not a texture is uniform across the atlas's one
material, so its expression is plain, nothing per-material to compute.
An enum property reads a plain [string](#strings), its value checked
against the property's own token list, an unknown token erroring with
the format named.

A texture property takes its image from its argument. A value embeds: the
bytes land in the mesh, the property points at them, and the slot's own
fixed requirement supplies the encoding. A `--write-material-slot-file`
references the named file by relative path, whether this run's
`--write-file-png-value` wrote it or a paint program did. A
`--write-file-png-value` beside a `--write-material-slot-value` is the
retired `both`: the mesh references the embedded copy and the loose
file is a working duplicate of the same bytes. Two slots naming one
value share the one embedded image, which is how an ORM packing fills
both of its slots; two slots demanding different encodings of one
value error. Every other property takes a plain value, so a `max()`
reduction lands in the material.

A writer and a slot stay separate flags because each is whole alone. A
writer alone is a file the mesh never mentions. A factor is a slot
with no bytes, `--write-material-slot-value 0 emissiveStrength
maxStrength`, a number written straight into the material. A texture
slot carries its own image, embedding a value or referencing a file,
so the two families meet only when `--write-material-slot-file` names
a file `--write-file-png-value` wrote.

The writer sets only what a slot names. Today's bake breaks that rule in one
place, injecting an `emissiveFactor` of `[1, 1, 1]` whenever it binds an
emissive texture. The reason is a real glTF trap: emission is
`emissiveTexture` multiplied by `emissiveFactor`, and that factor defaults to
black, so binding the texture alone emits nothing at all. The injection is
still a silent default of the kind this design rejects, and it would
fight a `--write-material-slot-value` sending anything else to
`emissiveFactor`, with no rule saying which wins.
The profile spells the factor instead, the way it spells every other default.

glTF fixes each texture slot's encoding: `baseColorTexture` and
`emissiveTexture` are sRGB, and `metallicRoughnessTexture`,
`occlusionTexture`, and `normalTexture` are linear. A value-form slot
encodes to order, so it cannot mismatch. A `--write-material-slot-file`
naming a file this run's `--write-file-png-value` wrote cross-checks
that writer's token against the slot, an error rather than a mesh that
renders wrong; a file from anywhere else is trusted to match, since
nothing knows its encoding.

A map with no standard property has two homes: loose beside the mesh
as `--write-file-png-value`, its transfer named by the writer and
stamped in the file's own chunks, or inside the mesh through the
material extras. The shipped bake still lists slotless embedded maps
under an `extras.vxl.maps` key; that listing has no producer left, so
it is deleted with the rest of the retired surface.

The material extras are the custom namespace: four flags writing
named entries under the material's `extras`, the key glTF reserves
for application data, in the `vxl.values` namespace the mesh extras
share, the form spelled in the name.
`--write-material-extra-json-value <material-index> <dst-name>
<src-expr> <linear | srgb>` puts the value's numbers in the entry
itself, a plain vec1 as one number, a vecN as an array of N, and an
array value as rows, one per flattened material.
`--write-material-extra-image-value <material-index> <dst-name>
<src-expr> <linear | srgb>` embeds an array as an image, the entry
holding its texture index; a plain value errors, an image needing
texels. `--write-material-extra-image-file <material-index>
<dst-name> <src-file>` writes the same `{"index"}` entry with the
texture referencing the named file by relative uri, and
`--write-material-extra-json-file <material-index> <dst-name>
<src-file>` points the entry at a JSON file instead. The entry
shapes cannot be confused:

```jsonc
"extras": { "vxl": { "values": {
  "heatScale": { "index": 3 },
  "accentColor": [0.87, 0.44, 0.44],
  "wear": { "uri": "turret-wear.json" }
} } }
```

A conforming viewer ignores it all; your own runtime looks the name
up. The `-value` forms carry the token because a custom entry fixes
no encoding: the transfer the stored components take, `srgb` for a
color your runtime reads display-encoded and `linear` for everything
else, an alpha component staying linear like the image rule.
`--write-file-json-value` takes the same token for the same reason,
so the choice between the two is placement, an entry riding inside
the mesh against a file beside it. The `-file` forms carry none, the
named file's own chunks or writer having declared what it takes, and
an image entry stays a bare index either way, the PNG's chunks
speaking for it. Keeping the extras separate from the slots keeps
typos loud: an unknown standard property in
`--write-material-slot-value` still errors. The same name twice, in
any two forms, errors. Two `--write-material-extra-image-value`
naming one value share one image, the two-encodings rule applying
across the slots and the extras alike, and a format without `extras`
rejects the whole grid.

## Vertex attributes

`--write-primitive-builtin-value <primitive-index> <dst-attribute>
<src-expr>` writes a value to an attribute glTF defines on the
indexed primitive: `COLOR_0`, the vertex color. The defined
vocabulary fixes each attribute's encoding the way the material
schema fixes its slots', so the flag carries no token, and an
unknown or underscore name errors, the custom flag the underscore's
home. Dimension picks the accessor type, vec1 through vec4 writing
SCALAR, VEC2, VEC3, and VEC4 floats.

`--write-primitive-custom-value <primitive-index> <dst-name>
<src-expr> <linear | srgb>` is the custom twin: glTF requires the underscore
prefix of application-specific attributes, so the name is typed with
it, `_MY_COLOR` landing exactly as written and a bare name erroring,
an attribute only your own shader reads that can never collide with
a defined name. Nothing fixes its encoding, so the token declares
the transfer the stored floats take.

The attributes live on the corners, the [ladder](#domains)'s top, so a
value of any domain climbs in: a plain value broadcasts to every corner,
a row value gives each corner its face's row's entry, a merged greedy
quad staying uniform since it is single-row, a face value duplicates
onto its four corners, and a corner value writes each corner exactly,
which is what [computed occlusion](#computed-occlusion) wants.

A format without vertex attributes rejects both flags. Each flag
writes its indexed primitive alone, so two primitives carry exactly
the attributes their flags spell. The index `--write-primitive-index`
writes is not a value: it is a row number into the extras rows
riding beside it, so the [palettes](mesh.md#palettes) stay their own
design.

## Computed occlusion

Occlusion computed from the voxel geometry, each face corner reading the
voxels that meet there. A corner is where neighbors crowd in, so the
result is a [corner](#domains) value, the domain's producer, and the
first value that varies across a surface: every palette property is per
row, which is why the unwrap atlas exists, an unwrap of row values being
the palette atlas with redundant texels.

`--compute-occlusion <dst-name>` requests the computation and binds the
result under the name, a corner vec1 in `0..1`, `1` fully open,
bound ahead of every `--value` the way palette properties are. Nothing
computes unrequested: without the flag the name does not exist, and an
expression naming one is the ordinary unknown-name error. The request
is explicit because it can change the geometry: through
`--write-primitive-builtin-value` the value writes corner-exact,
and greedy merging then splits a quad
only where its corner occlusion disagrees. Several requests bind their
names to one computation, aliases rather than a collision.

The value mixes like any other, and tuning is one expression each
rather than a flag family:

```
--compute-occlusion computedOcclusion
--value ao "lerp(1, computedOcclusion, 0.8)"       # strength 0.8
--value ao "max(computedOcclusion, 0.2)"           # min brightness 0.2
--value aoFace "faceAverage(ao)"                   # corners down to faces
--write-file-png-value turret-ao.png aoFace srgb   # color space: the token
```

A texture holds one texel per entry, so the corner value steps down
first, `faceAverage` or its siblings, and the face texture rides its own
[UV stream](mesh.md#uv-streams) beside any row maps. A sampled
neighborhood model, a radius and a falloff curve, is a possible
extension beyond the discrete corner method.

A profile requests occlusion through its `computeOcclusion` key, the
flag's argument as its value: the `baked-ao` example in the
[profile language](profile-language.md#user-defined-profiles) binds the
name and bakes it into the standard occlusion slot.

## Grammar

This document gives the grammar in two forms. The first is dimension-typed:
vec1, vec2, vec3, and vec4 expressions each get their own nonterminals, so
the same-dimension rules for the operators, the vec1 broadcast rules, and
the result type of every swizzle are encoded directly in the grammar. It
encodes the dimension axis only; the shape axis (plain versus array, see
[Shapes](#shapes)) is enforced by checking rules. The second form is a
compact untyped grammar plus those checking rules. It is the form to
implement, since a parser cannot know a name's dimension from syntax
alone.

A vec1 is a scalar. It is named like the other vectors so that swizzling and
broadcasting work uniformly across all four dimensions.

Precedence is encoded by stratification. From loosest to tightest binding:
or (`||`), xor (`^`), and (`&&`), comparison (`< <= > >= == !=`),
additive (`+ -`), multiplicative (`* /`), unary (`-` and `!`), postfix
(swizzle, member, index), primary. Left recursion gives left-to-right
associativity for `+ - * / && ^ ||`.

### Dimension-typed BNF

```bnf
; ============================================================
; Start rule: an expression of any dimension
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
                  | "min" "(" <vec1-expr> ")"                   ; palette minimum
                  | "min" "(" <vec1-expr> "," <vec1-expr> ")"
                  | "max" "(" <vec1-expr> ")"                   ; palette maximum
                  | "max" "(" <vec1-expr> "," <vec1-expr> ")"
                  | "sum" "(" <vec1-expr> ")"                   ; palette sum
                  | "avg" "(" <vec1-expr> ")"                   ; palette mean
                  | "dot" "(" <vec1-expr> "," <vec1-expr> ")"   ; component fold
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
                  | "faceAverage" "(" <vec1-expr> ")"         ; corners to faces
                  | "faceMin" "(" <vec1-expr> ")"
                  | "faceMax" "(" <vec1-expr> ")"
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
                  | "faceAverage" "(" <vec2-expr> ")"
                  | "faceMin" "(" <vec2-expr> ")"
                  | "faceMax" "(" <vec2-expr> ")"
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
                  | "faceAverage" "(" <vec3-expr> ")"
                  | "faceMin" "(" <vec3-expr> ")"
                  | "faceMax" "(" <vec3-expr> ")"
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
                  | "faceAverage" "(" <vec4-expr> ")"
                  | "faceMin" "(" <vec4-expr> ")"
                  | "faceMax" "(" <vec4-expr> ")"
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
                  | "any" "(" <comparison> ")"        ; or-fold of the components
                  | "all" "(" <comparison> ")"        ; and-fold of the components

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

<num>           ::= <digits>
                  | <digits> "." <digits>
                  | "." <digits>

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

Because a `<name>` (and `default(...)`) can be of any dimension, the typed
grammar above is ambiguous for a parser that does not know what each name
refers to. The practical structure: parse with the untyped grammar below, then
run a checker over the AST using the dimension and shape rules that follow.

```bnf
<u-expr>     ::= <u-expr> "||" <u-xor>
               | <u-xor>

<u-xor>      ::= <u-xor> "^" <u-and>
               | <u-and>

<u-and>      ::= <u-and> "&&" <u-cmp>
               | <u-cmp>

; A chain like a < b < c parses; the checker rejects the bool
; operand its left comparison feeds the right one.
<u-cmp>      ::= <u-cmp> <cmp-op> <u-add>
               | <u-add>

<cmp-op>     ::= "<" | "<=" | ">" | ">=" | "==" | "!="

<u-add>      ::= <u-add> "+" <u-term>
               | <u-add> "-" <u-term>
               | <u-term>

<u-term>     ::= <u-term> "*" <u-unary>
               | <u-term> "/" <u-unary>
               | <u-unary>

<u-unary>    ::= "-" <u-unary>
               | "!" <u-unary>
               | <u-post>

<u-post>     ::= <u-post> "." <member>
               | <u-post> "[" <u-expr> "]"
               | <u-prim>

<u-prim>     ::= <num>
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
               | "dot" "(" <u-expr> "," <u-expr> ")"
               | "length" "(" <u-expr> ")"
               | "distance" "(" <u-expr> "," <u-expr> ")"
               | "normalize" "(" <u-expr> ")"
               | "cross" "(" <u-expr> "," <u-expr> ")"
               | "oklabFromRgb" "(" <u-expr> ")"
               | "rgbFromOklab" "(" <u-expr> ")"
               | "oklchFromRgb" "(" <u-expr> ")"
               | "rgbFromOklch" "(" <u-expr> ")"
               | "faceAverage" "(" <u-expr> ")"
               | "faceMin" "(" <u-expr> ")"
               | "faceMax" "(" <u-expr> ")"
               | "default" "(" <name> "," <u-expr> ")"

; A member is always a swizzle: 1-4 components over one alphabet,
; {r,g,b,a} or {x,y,z,w}, repeats allowed. Every value is a
; vector, so there is nothing else it could be.
<member>     ::= <ident>
```

Dimension rules for an expression `e` with dimension written `dim(e)`:

| Construct                | Requirement                                                                                                                             | Result dimension      |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------------------- | --------------------- |
| `a + b`, `a - b`         | `dim(a) = dim(b)`                                                                                                                       | `dim(a)`              |
| `a * b`, `a / b`         | `dim(a) = dim(b)`, or `dim(b) = 1`; for `*` also `dim(a) = 1`                                                                           | the larger of the two |
| `-e`                     | any dimension                                                                                                                           | `dim(e)`              |
| `r(x)`                   | `dim(x) = 1`                                                                                                                            | 1                     |
| `rg(x, y)`               | all args dimension 1                                                                                                                    | 2                     |
| `rgb(x, y, z)`           | all args dimension 1                                                                                                                    | 3                     |
| `rgba(x, y, z, w)`       | all args dimension 1                                                                                                                    | 4                     |
| `e.s` (swizzle)          | one alphabet, `rgba` or `xyzw`; every component exists in `dim(e)`: `r`/`x` always, `g`/`y` needs >= 2, `b`/`z` needs >= 3, `a`/`w` needs 4; `1 <= len(s) <= 4`, repeats allowed | `len(s)`              |
| `e[i]`                   | `dim(i) = 1`                                                                                                                            | `dim(e)`              |
| `min(e)`, `max(e)`       | any dimension                                                                                                                           | `dim(e)`              |
| `min(a, b)`, `max(a, b)` | `dim(a) = dim(b)`, or either `= 1`                                                                                                      | the larger of the two |
| `sum(e)`, `avg(e)`       | any dimension                                                                                                                           | `dim(e)`              |
| `abs(e)`, `round(e)`     | any dimension                                                                                                                           | `dim(e)`              |
| `pow(a, b)`, `mod(a, b)` | `dim(a) = dim(b)`, or `dim(b) = 1`                                                                                                      | `dim(a)`              |
| `clamp(x, lo, hi)`       | `dim(lo) = dim(hi)`, equal to `dim(x)` or 1                                                                                             | `dim(x)`              |
| `lerp(a, b, t)`          | `dim(a) = dim(b)`; `dim(t) = dim(a)` or 1                                                                                               | `dim(a)`              |
| `step(edge, x)`          | `dim(edge) = dim(x)`, or `dim(edge) = 1`                                                                                                | `dim(x)`              |
| `smoothstep(lo, hi, x)`  | `dim(lo) = dim(hi)`, equal to `dim(x)` or 1                                                                                             | `dim(x)`              |
| `floor(e)`, `ceil(e)`    | any dimension                                                                                                                           | `dim(e)`              |
| `<num>`                  | (none)                                                                                                                                  | 1                     |
| `true`, `false`          | (none)                                                                                                                                  | bool                  |
| `<name>`                 | dimension of the value it names                                                                                                         | that dimension        |
| `default(name, e)`       | `dim(name) = dim(e)` where `name` has a value                                                                                           | `dim(e)`              |
| `a < b`, `a <= b`, `a > b`, `a >= b`, `a == b`, `a != b` | `dim(a) = dim(b) = 1`, numbers on both sides; wider sides only inside `any`/`all`                       | bool                  |
| `any(c)`, `all(c)`       | `c` a comparison, `dim(a) = dim(b)` or either `= 1`                                                                                     | bool                  |
| `!e`                     | `e` bool                                                                                                                                | bool                  |
| `a && b`, `a ^ b`, `a \|\| b` | both bool                                                                                                                          | bool                  |
| `mix(a, b, c)`           | `dim(a) = dim(b)`, or both strings; `c` bool                                                                                            | `dim(a)`              |
| `faceAverage(e)`, `faceMin(e)`, `faceMax(e)` | any dimension                                                                                                       | `dim(e)`              |
| `dot(a, b)`              | `dim(a) = dim(b)`                                                                                                                       | 1                     |
| `length(e)`              | any dimension                                                                                                                           | 1                     |
| `distance(a, b)`         | `dim(a) = dim(b)`                                                                                                                       | 1                     |
| `normalize(e)`           | any dimension                                                                                                                           | `dim(e)`              |
| `cross(a, b)`            | `dim(a) = dim(b) = 3`                                                                                                                   | 3                     |
| the color conversions    | `dim(c) = 3`                                                                                                                            | 3                     |
| `<string-lit>`           | (none)                                                                                                                                  | string                |
| string `==`, `!=`        | both sides strings                                                                                                                      | bool                  |

A vec1 broadcasts on the right of `/`, as the exponent of `pow`, on
either side of `*`, and on either side of the comparison inside
`any`/`all`. `dot`, `distance`, and `cross` take no broadcast: their
sides share one dimension exactly.

Shape rules, with a value either plain or an array over the effective palette
(see [Shapes](#shapes)):

1. A property name is an array; a literal is plain; a `--value` name has
   its definition's shape.
2. The elementwise constructs, the operators, the constructors, swizzles,
   binary `min`/`max`, `abs`, `pow`, `mod`, `clamp`, `lerp`, `step`,
   `smoothstep`, `floor`/`ceil`/`round`, `default()`, `dot`, `length`,
   `distance`, `normalize`, `cross`, and the color conversions, pair
   arrays element by element and broadcast plain values; the result is
   an array when any operand is. `dot`, `length`, and `distance` fold
   the components inside each entry, never entries across the domain,
   so they reduce dimension to 1, not shape to plain.
3. The reductions, unary `min`/`max` and `sum`/`avg`, require an array and
   yield a plain value, computed per component across its whole domain.
4. `e[i]` requires `e` an array and `i` a plain exact non-negative integer
   below the array's entry count, and yields a plain value.
5. A writer takes whatever shape its destination holds: a PNG a row or
   face array, a `--write-material-slot-value` factor a plain value, JSON
   either shape.
6. The comparisons and logical operators follow rule 2 over their bool
   results, `any`/`all` fold each entry's component answers and keep
   the comparison's shape, and `e[i]` on a bool array follows rule 4.
   A bool reaches `--write-file-json-value` under `linear`,
   `--primitive`'s select, a boolean material property, and `mix`'s
   chooser; nothing else takes one.
7. An array carries its [domain](#domains), and rule 2 pairs entries
   after the lower domain climbs onto the higher; the climb is implicit,
   a step down is `faceAverage`/`faceMin`/`faceMax` or a reduction to
   plain, and a corner array meeting a texture destination errors.
8. `mix(a, b, cond)` follows rule 2 across `a`, `b`, and `cond`, the
   result's shape from any operand array.
9. A string rides the same axes: a literal is plain, a string property
   an array, its `==`/`!=` and `mix` pair entries by rule 2, `e[i]`
   follows rule 4, and a string reaches an enum property plain, a JSON
   destination in either shape, and nothing else.

### Functions

One item per function. Dimensions and shapes follow the tables above.

1. `r(x)`, `rg(x, y)`, `rgb(x, y, z)`, and `rgba(x, y, z, w)` build a
   vector from vec1 parts. This is how channels pack into a map:

   ```
   rgb(occlusion, roughness, metallic)   # the orm pack
   ```

2. Unary `min(e)` and `max(e)` reduce an array across its domain, per
   component, to a plain value:

   ```
   max(emissiveStrength)   # the palette's strongest strength
   ```

3. Binary `min(a, b)` and `max(a, b)` are elementwise, a vec1 broadcasting
   from either side:

   ```
   max(maxStrength, 0.001)   # floors a divisor
   ```

4. `sum(e)` and `avg(e)` are the other reductions, the total and the mean
   across the array's domain:

   ```
   avg(baseColorFactor)   # the palette's mean color
   ```

5. `abs(e)` is the componentwise magnitude:

   ```
   abs(tint - avg(tint))   # each material's spread around the mean
   ```

6. `dot(a, b)` multiplies matching components and adds the products,
   one number out. The sides share a dimension exactly, nothing
   broadcasts, and a vec1 pair degenerates to the plain product:

   ```
   dot(tint, rgb(0.2126, 0.7152, 0.0722))   # a luminance weighting
   ```

7. `length(e)` is the vector's magnitude, `pow(dot(e, e), 0.5)` under
   one name; a vec1's length is its absolute value:

   ```
   length(emissiveFactor)   # the emissive color's overall strength
   ```

8. `distance(a, b)` is `length(a - b)`, the straight-line gap between
   two points, read in [Oklab](#color-spaces) when the gap should
   match what the eye sees:

   ```
   distance(lab, oklabFromRgb(rgb(1, 0, 0)))   # how far from red
   ```

9. `normalize(e)` is `e / length(e)`, the direction alone at length 1.
   A zero vector divides `0 / 0` and errors like any non-finite:

   ```
   normalize(offset)   # the direction, the magnitude dropped
   ```

10. `cross(a, b)` is the vec3 cross product, the vector perpendicular
    to both sides:

    ```
    cross(u, v)   # a normal for the plane u and v span
    ```

11. `oklabFromRgb(c)` and `rgbFromOklab(l)` convert a vec3 between
    linear RGB and Oklab, the perceptual space; see
    [Color spaces](#color-spaces):

    ```
    oklabFromRgb(baseColorFactor.rgb).x   # perceived lightness
    ```

12. `oklchFromRgb(c)` and `rgbFromOklch(l)` convert a vec3 between
    linear RGB and Oklch, Oklab's polar form, hue a number ordinary
    arithmetic can shift; see [Color spaces](#color-spaces):

    ```
    mod(oklchFromRgb(tint).z + 0.1, 1)   # a tenth of a turn around
    ```

13. `pow(a, b)` is the componentwise exponent. A vec1 exponent
    broadcasts across `a`, and `pow(vec1, vecN)` errors, matching the
    rule for `/`:

    ```
    pow(roughnessFactor, 2.2)   # steepens the roughness curve
    ```

14. `mod(a, b)` is the floored remainder, `a - b * floor(a / b)`, the
    form that wraps. `mod(a, 0)` is non-finite and errors like any
    other:

    ```
    mod(hue + 0.618, 1)   # wraps back into 0..1
    ```

15. `clamp(x, lo, hi)` pins each component into `lo..hi`. A component
    with `lo > hi` errors. An explicit `clamp` is the author naming a
    bound, which is exactly what the write-time rules ask for:

    ```
    clamp(strength / 4, 0, 1)   # the author's own bound
    ```

16. `lerp(a, b, t)` is `a + (b - a) * t`. `t` is unrestricted, so it
    extrapolates outside `0..1`. The name is HLSL's, because it says
    what the blend does; `mix` names the [bool chooser](#booleans)
    instead:

    ```
    lerp(orm, rgb(1, 1, 1), 0.25)   # a quarter of the way to white
    ```

17. `step(edge, x)` is 0 where `x < edge` and 1 elsewhere, the mask maker:

    ```
    step(0.001, emissiveStrength)   # 1 for every material that emits
    ```

18. `smoothstep(lo, hi, x)` is the Hermite ramp: 0 at `lo`, 1 at `hi`,
    held flat outside. A component with `lo >= hi` errors, one step
    stricter than `clamp`, since the ramp divides by `hi - lo`:

    ```
    smoothstep(0.2, 0.8, occlusion)   # eases a mask edge
    ```

19. `floor(e)` and `ceil(e)` snap each component to the integer below or
    above, and `round(e)` to the nearest, halves away from zero:

    ```
    round(smoothness * 4) / 4   # five even levels
    ```

20. `mix(x, y, cond)` picks per entry: `x` where the bool is false, `y`
    where it is true, GLSL's own bool overload of its `mix`. The
    branches share a dimension, or are both [strings](#strings), and
    the result takes it, and the chooser is the one place a bool meets
    numbers, the spelled bridge out of the type:

    ```
    mix(0, 1, glowing)   # the deliberate 0/1 mask
    ```

21. `any(c)` and `all(c)` fold a comparison's component answers into
    one bool, `any` with or and `all` with and. The argument is a
    comparison spelled in place, its sides sharing a dimension or
    either a vec1, and the fold runs per entry, so an array
    comparison folds to a bool array; a vec1 comparison folds its
    one answer, the identity:

    ```
    all(baseColorFactor.rgb > 0.9)   # true where a color runs near white
    ```

22. `faceAverage(e)`, `faceMin(e)`, and `faceMax(e)` step a corner
    array down to a face array, each face's four corners reduced per
    component, the spelled descent of the [ladder](#domains); any
    other domain errors:

    ```
    faceAverage(computedOcclusion)   # one occlusion per face
    ```

23. `default(name, fallback)` evaluates to `name` where it has a value
    and to `fallback` where it does not: a `--value` name not yet
    defined, a property no layer supplies, or a material that leaves it
    unset, filled per element. `name` is bare or backtick-quoted, and
    `fallback` is any expression of the same dimension. Nothing
    auto-defaults, and an unbound name is an error, so a robust
    expression spells the spec default itself:

    ```
    default(occlusionStrength, 1)   # the glTF default where unset
    ```

### Notes

**Backtick quoting.** Backticks quote a name a bare identifier
cannot spell: spaces, a leading digit, or a reserved name. `foo bar` always
lexes as two separate names; the value is written `` `foo bar` ``. Double
quotes are not name quoting; they spell [string](#strings) literals.
In a shell, single-quote an expression holding backticks, which the shell
would read as command substitution, or double quotes, which it would
strip.

**Reserved names.** The function names `r`, `rg`, `rgb`, `rgba`, `min`,
`max`, `sum`, `avg`, `any`, `all`, `abs`, `pow`, `mod`, `clamp`,
`lerp`, `mix`, `step`, `smoothstep`, `floor`, `ceil`, `round`, `dot`,
`length`, `distance`, `normalize`, `cross`, `oklabFromRgb`,
`rgbFromOklab`, `oklchFromRgb`, `rgbFromOklch`, `faceAverage`,
`faceMin`, `faceMax`, and `default` are keywords, and the literals
`true` and `false` are reserved with them. A
property sharing one is reached by backtick-quoting: `` `min` `` is the
name, `min(...)` the function.

**Swizzle rules.** A swizzle is any sequence of 1-4 components, repeats
allowed, where every component exists in the source. Two alphabets name
the same components, color `rgba` and position `xyzw`, so `v.xyz` and
`v.rgb` name the same value, whichever reads better for the data. One
swizzle draws from one alphabet, so `v.xg` errors, the rule the shader
languages share. Existence is per component: `r`/`x` always work,
`g`/`y` need dim >= 2, `b`/`z` need dim >= 3, `a`/`w` need dim 4. Every
dimension can be swizzled, including vec1: `s.r` is the identity, and
repeats splat upward, so `s.rr` is a vec2 and `s.rrrr` is a vec4.
Results can be wider or narrower than the source: `v2.rrgg` is a vec4
and `v4.r` is a vec1. With vec1 splats available, `r(s)` duplicates
`s.r` and `rg(s, s)` duplicates `s.rr`; the constructors are still
needed when the arguments differ, as in `rg(u, v)`, and they spell
`rgba` alone, one name per function. The parser takes any identifier
after the dot; the checker limits the components.

```
baseColorFactor.rgb   # vec4 to vec3, dropping alpha
orm.g                 # one channel, roughness
0.5.rrr               # a grey vec3 splat from one number
tint.rrgg             # wider than its source
offset.xyz            # the position alphabet, the same value as .rgb
```

**Precedence and associativity.** From tightest to loosest: postfix (swizzle,
member, index), unary `-` and `!`, `* /`, `+ -`, the comparisons
`< <= > >= == !=`, `&&`, `^`, `||`. Postfixes chain left to right, so
`baseColorFactor[0].rgb` and `baseColorFactor.rgb[0]` both parse, and name
the same value. Unary minus nests, so `- -x` is valid. There is no `--` token
in the expression language, so `--value` never collides with it.

**pow, not `^`.** Exponent is spelled `pow()`: the shader languages this
one borrows its swizzles from read `^` as XOR, and that is the meaning
the character carries here, the [bool](#booleans) exclusive or, never
exponent.

**The function set stays small.** `sqrt(x)` is `pow(x, 0.5)`, `fract(x)` is
`mod(x, 1)`, and a signed remap is `n * 0.5 + 0.5`, so none of them is a
function. The vector set is the spelled exception: `normalize(e)` is
`e / length(e)` and ships anyway, the set whole under the names the
shader languages share.

**Lexing.** Whitespace separates tokens and is otherwise insignificant, with
two exceptions: inside a backtick-quoted name or a string literal it is
literal, and in a postfix it is forbidden. A postfix is attached: the dot and its member hug
the source's final token, and an index bracket does the same. `v.rg` and
`tint[0]` are postfixes; `v .rg`, `v. rg`, and `tint [0]` are all errors.
Numbers use maximal munch: `1.5` is a single number token, and `.5` where
an expression is expected is a number. Since numbers are vec1 values,
literals can be swizzled too. In `2.rr` the munch stops at `2`, since `2.`
followed by a letter is not a number; the attached dot then begins a
swizzle, giving a vec2 splat. `2.5.rr` works the same way.

**Linear evaluation.** Every expression evaluates in linear space over
floats. A color property decodes its stored form to linear on read; a
non-color property is read as written. Transfer back to sRGB happens only at
a writer's edge, per its token or its slot's fixed encoding, and
quantization only where an image is written, so a `linear` JSON file takes
the numbers exactly as evaluated.

**The transfer token lives where no slot fixes an encoding.** A
`--write-file-png-value` file is bound to nothing, a
`--write-file-json-value` key is read only by your own code, and the
extras `-value` forms and `--write-primitive-custom-value` fill
destinations the format does not define, so each names its encoding
itself; the PNG token also feeds the `--write-material-slot-file`
cross-check and the file's own chunks. A `--write-material-slot-value`
carries no token because its slot's fixed encoding is the definition,
not an inference: `--write-material-slot-value baseColorTexture
albedo` embeds an sRGB image by what `baseColorTexture` is, and
`COLOR_0`'s linear definition decides for
`--write-primitive-builtin-value` the same way; the `-file` forms
carry none either, the named file's own chunks saying what it is.

**Why `linear`.** A writer's token names a transfer function, and `srgb` and
`linear` are the two: the sRGB curve and the identity. The voxel-json plan
deletes `linear-rgb-float`, but that was a color kind claiming a value was a
color while identifying none, and here nothing claims to be a color, so the
objection does not carry over. `raw` and `none` describe an absence rather
than naming the function, which reads worse beside `srgb`, and `non-color`
names authoring intent rather than an encoding, which is wrong for a
linear-light color.

**The PNG says what it is.** Every PNG the bake encodes stamps its transfer
into the bytes: `srgb` the `sRGB` chunk with its recommended `gAMA`/`cHRM`
fallbacks, `linear` a `gAMA` of 1.0, the nearest standard spelling of the
identity. Color chunks are ancillary, so a decoder that does not recognize
one ignores it and renders as it would have without; nothing fails to open,
and a color-managed viewer stops showing a `linear` map washed out. glTF
ignores the chunks and takes the encoding from the slot, so the mesh reads
the same. A `--write-material-slot-value` stamps its embedded image
identically, the slot's encoding in the chunks, and PNG's newer `cICP`
chunk, which names a linear transfer exactly, can join later without
disturbing any of this.

**Write-time errors.** A non-finite component (NaN or infinity, as from
`0 / 0`) is an error wherever it appears. Clamping is always the author's,
written `clamp()`. The destination decides the range, and only an image
has one: a PNG requires every component in `0..1`, so `1.5` into a PNG
errors while `1.5` into a JSON field is fine, which is how an unbounded
property like `emissiveStrength` travels.

**Redefinition.** `--value` may redefine any name, a property or an earlier
value. The right side is evaluated against the bindings visible at that
point, so `--value roughnessFactor "pow(roughnessFactor, 2)"` reads the
property and rebinds the name, and later expressions see the new value. There is no
recursion.
