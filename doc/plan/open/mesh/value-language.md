# Mesh value language

_A subplan of [`vxl mesh`](../vxl-commands/reference/mesh.md): a proposed redesign of the material-map
flags around an expression language for values, and writers that put them in
images, JSON, and the mesh's own material._

1. [ ] Options
   1. `--value <symbol> <expr>`
      1. Creates a new value which can be used in a material. For example,
         `--value smoothnessFactor "1 - roughnessFactor"` creates a new value
         named `smoothnessFactor` which is the inverse of the `roughnessFactor`
         property
      2. Every property of the effective palette can be used as a symbol.
      3. Symbols defined with `--value` can be used in other `--value`
         expressions. They are evaluated in order. If a symbol is used before
         it is defined, the expression will fail to evaluate unless it is used
         in `default()`.
      4. See
         [Texture-Map Value Language Grammar](#texture-map-value-language-grammar)
         for the expression grammar.
   2. `--write-png <linear | srgb> <value> <file>`: writes an array value to an
      8-bit PNG beside the mesh, one texel per material in
      [shape order](#shapes). The image is sized to its value: vec1 through
      vec4 write grey, grey-alpha, RGB, and RGBA, components mapping to
      channels by position. Grey-alpha is PNG's only two-channel form, so a
      vec2's second component lands in the alpha channel; pad with
      `rgb(u, v, 0)` where a viewer should read opaque color. The token
      names the encoding: `srgb` applies the sRGB transfer, for an image a
      viewer reads as color, and `linear` applies none, for the data
      channels glTF wants linear. A component outside `0..1` errors. The
      file also declares its transfer in its own chunks; see the notes.
   3. `--write-json <linear | srgb> <value> <file>`: writes a value to a
      JSON file under its own symbol as the key, the token naming the
      transfer the written numbers take. Repeatable, and repeating it on one
      path merges, so the file is always an object; see
      [JSON files](#json-files).
   4. `--write-vertex <linear | srgb> <value> <target>`: writes a value to
      a vertex attribute, glTF's `COLOR_0` or a custom `_NAME`, each face
      corner taking its material's value. Deferred until the vertex work
      lands; see [Vertex attributes](#vertex-attributes).
   5. `--slot <value> <property>`: sets one property of the output material.
      A plain value becomes a material field; an array value embeds, its
      image landing in the glb binary chunk or as a data URI in a `.gltf`;
      see [Material slots](#material-slots).
   6. `--slot-file <file> <property>`: sets a texture property to reference
      an existing file by relative path, whether `--write-png` wrote it or
      not; see [Material slots](#material-slots).
   7. `--slot-extra <linear | srgb> <value> <name>`: writes a value under
      the material's `extras.vxl.<name>`, an array embedding as an image and
      a plain value landing as its numbers, the custom slot for a schema you
      own; see [Material slots](#material-slots).
   8. `--slot-extra-file <file> <name>`: sets a custom `extras.vxl.<name>`
      entry to reference an existing file by relative path, the
      `--slot-file` twin for the custom slot; see
      [Material slots](#material-slots).
   9. A writer's `<value>` names a symbol defined with `--value`, never an
      inline expression.
2. [ ] `--profile <profile>`: applies a profile's values as if each were a
       `--value` at the flag's own position, and queues its outputs for the
       write that runs by default; the built-ins ship in the binary and the
       rest come from `.vxlconfig`. Repeatable; see [Profiles](#profiles) and
       the [profile language](profile-language.md).
   1. `--write-profile <true | false>`: whether the merged outputs of
      every named profile fire. The default is `true`, so naming a
      profile writes what it spells; `false` keeps every named profile a
      values mixin. At most once per run; see [Profiles](#profiles).
   2. `--stem <stem>`: replaces `{stem}` in profile file templates. The
      default is the output mesh's own stem, so `--to turret.glb` fills
      `{stem}-mse.png` as `turret-mse.png` with no flag at all.

Together these retire five of the shipped map flags. The slots and writers
replace `--texture-storage`: an image goes where its flag puts it, and the
old `both` is a `--write-png` beside a value-form `--slot`. `--value`
replaces `--texture-map`, and backtick quoting reaches a voxel-json key
directly, retiring `--define-property`. The two naming flags retire as
well: a profile names its own files through `{stem}` templates, an exact
rename is a `.vxlconfig` profile respelling the template, and `--stem`
replaces the prefix flag. A hand-written writer names its own file inline.

Mesh generates materials for a mesh based on the object's layers. The
combination of layers results in an "effective palette", each property read
through the last layer whose palette supplies its name; see
[The palette atlas](../vxl-commands/reference/mesh.md#the-palette-atlas). Each property of the
effective palette can be used as a value in the final object's materials.

## Shapes

A value is plain or an array, independent of its vec1-vec4 dimension. A
property is an array: one element per distinct flattened material of the
effective palette, in first-seen raster order, the order the atlas lays its
texels out and [`PaletteData.materials`](../vxl-commands/reference/mesh.md#deferred) lists. A numeric
literal is plain. Elementwise operations pair arrays element by element and
broadcast a plain value across an array, so `1 - roughnessFactor` is an
array; two arrays always align, since every array runs over the one effective
palette.

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

`e[i]` samples an array at material index `i`, a plain result: `tint[0]` is
the first material's tint. The index is a plain vec1 holding an exact
non-negative integer below the material count; a fraction, a negative, an
out-of-range index, or an array index (a gather) errors. Indexing and
swizzling commute: `baseColorFactor[0].rgb` and `baseColorFactor.rgb[0]` are
the same value.

A PNG writes an array, one texel per element; a plain value has no texels and
cannot be written to a PNG. In JSON an array field writes as a JSON array and
a plain field as a single literal.

A third shape is reserved for the deferred [unwrap atlas](../vxl-commands/reference/mesh.md#deferred).
Computed occlusion varies across a surface, so it is per-face rather than
per-material, and the axis extends to plain, per-material, and per-face. A
per-material value broadcasts into a per-face one, since a face knows its
material, and only a reduction goes the other way. The writers then follow one
rule rather than two: a PNG's shape matches the atlas's texel domain,
per-material under `--atlas palette` and per-face under `unwrap`. None of this
is built: the shape lands with the unwrap atlas, `computedOcclusion` its
first symbol, and the deferred [vertex writer](#vertex-attributes) writes it
corner-exact; see [Computed occlusion](#computed-occlusion).

## JSON files

`--write-json <linear | srgb> <value> <file>` writes one value under its own
symbol as the key. Repeating it on one path merges, so a file with several
values is several flags rather than a grouping construct in the language:

```
--write-json linear albedo   turret-pbr.json
--write-json linear orm      turret-pbr.json
--write-json linear emissive turret-pbr.json
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

1. The key is the value's symbol. Rename by defining an alias first, as
   `--value rough roughnessFactor`, so a value carries one name everywhere it
   appears.
2. Repeating the flag on one path merges into that file, in flag order.
3. The same symbol twice into one file is an error, since one key would
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

Merging at the flag is what keeps every value numeric. Every value is a
vector, so a dot postfix is always a swizzle and the checker asks only
shape and dimension. Nesting was dropped with it: the deferred
[`PaletteData`](../vxl-commands/reference/mesh.md#deferred) is a fixed shape the exporter builds.

This mirrors the voxel-json
[value kinds](../voxj-value-kinds/README.md), which deleted their six
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

That boundary is the writer, not the field. `--write-png srgb albedo ...`
names the transfer its file takes, and a value-form `--slot` encodes to its
slot's fixed requirement, because an image is what a renderer decodes. A
JSON file has no format fixing a contract, so its token is you declaring
one: `linear` is the plain export, and `srgb` serves a reader that takes
its colors display-encoded.

## Material slots

`--slot <value> <property>` sets one property of the output material, source
before destination like the file writers. The property is the target
format's own name, the leaf of its material schema, so the flag invents no
vocabulary and the writer does the nesting and the `extensionsUsed`
bookkeeping:

```
--slot albedo baseColorTexture         # pbrMetallicRoughness.baseColorTexture
--slot orm metallicRoughnessTexture    # pbrMetallicRoughness.metallicRoughnessTexture
--slot orm occlusionTexture            # occlusionTexture, sharing one image
--slot emissive emissiveTexture        # emissiveTexture
--slot maxStrength emissiveStrength    # extensions.KHR_materials_emissive_strength
--slot glassIor ior                    # extensions.KHR_materials_ior
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

The property's own type decides how its argument reads:

| Property type    | Argument                                        |
| ---------------- | ----------------------------------------------- |
| `*Texture`       | a value to embed, or a file via `--slot-file`   |
| number or vector | a symbol naming a plain value of that dimension |
| enum             | one of the property's literal values            |
| boolean          | `true` or `false`                               |

```
--slot albedo baseColorTexture          # array value, embedded
--slot cutoff alphaCutoff               # plain vec1 symbol
--slot MASK alphaMode                   # enum literal
--slot true doubleSided                 # boolean literal
--slot-file skin.png baseColorTexture   # existing file, referenced
```

No token is ambiguous, since a property whose domain is `OPAQUE`, `MASK`, or
`BLEND` never holds a number. Enums and booleans read as plain literals
because the atlas produces one material: every property that is not a
texture is uniform across it, so there is nothing per-material to compute.
Double quotes stay reserved for future string literals.

A texture property takes its image from its argument. A value embeds: the
bytes land in the mesh, the property points at them, and the slot's own
fixed requirement supplies the encoding. A `--slot-file` references the
named file by relative path, whether this run's `--write-png` wrote it or
a paint program did. A `--write-png` beside a value-form `--slot` is the
retired `both`: the mesh references the embedded copy and the loose file
is a working duplicate of the same bytes. Two slots naming one value share
the one embedded image, which is how an ORM packing fills both of its
slots; two slots demanding different encodings of one value error. Every
other property takes a plain value, so a `max()` reduction lands in the
material.

A writer and a slot stay separate flags because each is whole alone. A
writer alone is a file the mesh never mentions. A factor is a slot with no
bytes, `--slot maxStrength emissiveStrength`, a number written straight
into the material. A texture slot carries its own image, embedding a value
or referencing a file, so the two families meet only when `--slot-file`
names a file `--write-png` wrote.

The writer sets only what a slot names. Today's bake breaks that rule in one
place, injecting an `emissiveFactor` of `[1, 1, 1]` whenever it binds an
emissive texture. The reason is a real glTF trap: emission is
`emissiveTexture` multiplied by `emissiveFactor`, and that factor defaults to
black, so binding the texture alone emits nothing at all. The injection is
still a silent default of the kind this design rejects, and it would fight a
`--slot` sending anything else to `emissiveFactor`, with no rule saying
which wins.
The profile spells the factor instead, the way it spells every other default.

glTF fixes each texture slot's encoding: `baseColorTexture` and
`emissiveTexture` are sRGB, and `metallicRoughnessTexture`,
`occlusionTexture`, and `normalTexture` are linear. A value-form slot
encodes to order, so it cannot mismatch. A `--slot-file` naming a file this
run's `--write-png` wrote cross-checks that writer's token against the
slot, an error rather than a mesh that renders wrong; a file from anywhere
else is trusted to match, since nothing knows its encoding.

A map with no standard property has two homes: loose beside the mesh as
`--write-png`, its transfer named by the writer and stamped in the file's
own chunks, or inside the mesh through `--slot-extra`. Only a slot embeds.
The shipped bake still lists slotless embedded maps under an
`extras.vxl.maps` key; that listing has no producer left, so it is deleted
with the rest of the retired surface.

`--slot-extra <linear | srgb> <value> <name>` is the custom slot: it puts a
value under the material's `extras`, the key glTF reserves for application
data, in a `vxl` namespace under the name you choose. An array embeds like
a value-form `--slot` and its entry is the texture index; a plain value's
numbers land in the entry itself, a vec1 as one number and a vecN as an
array of N, so the shapes cannot be confused:

```jsonc
"extras": { "vxl": {
  "heatScale": { "index": 3 },
  "accentColor": [0.87, 0.44, 0.44]
} }
```

A conforming viewer ignores it; your own runtime looks the name up. The
token is required because a custom slot fixes no encoding. It means the
same thing for both shapes: the transfer the stored components take,
`srgb` for a color your runtime reads display-encoded and `linear` for
everything else, an alpha component staying linear like the image rule.
`--write-json` takes the same token for the same reason, so the choice
between the two is placement, an entry riding inside the mesh against a
file beside it. An image entry stays a bare index, since the embedded
PNG's own chunks already declare its transfer. Keeping the flag separate
keeps typos loud: an unknown standard property in `--slot` still errors.
The same name twice errors. Two `--slot-extra` naming one value share one
image, the two-encodings rule applying across `--slot` and `--slot-extra`
alike, and a format without `extras` rejects the flag.

`--slot-extra-file <file> <name>` is the referencing twin, `--slot-file`
for the custom slot: the entry stays `{"index": N}`, the texture behind
it pointing at the named file by relative uri instead of embedded bytes.
It carries no token, the file's own chunks saying what it is.

## Vertex attributes

`--write-vertex <linear | srgb> <value> <target>` is the fourth writer,
the one that writes into the mesh, since a vertex attribute lives
nowhere else. The target is `COLOR_0`, the vertex color glTF defines, or
a custom `_NAME` attribute only your own shader reads; glTF requires the
underscore on application-specific attributes, so a misspelled defined
name stays an error rather than a quiet custom one. Dimension picks the
accessor type, vec1 through vec4 writing SCALAR, VEC2, VEC3, and VEC4
floats.

The value broadcasts onto face corners by material: every corner of a
face takes the face's material's value, and a merged greedy quad is
single-material, so a quad stays uniform. A plain value broadcasts to
every corner. When the [reserved per-face shape](#shapes) lands, a
per-face value writes each corner exactly, which is what
[computed occlusion](#computed-occlusion) wants.

`COLOR_0` is defined linear, so its token must be `linear`, the fixed
encoding cross-check the material slots already apply. A custom `_NAME`
fixes nothing, so its token declares the transfer the stored floats
take, the `--slot-extra` reasoning. A format without vertex attributes
rejects the flag. This folds in the deferred `--vertex`,
`--vertex-target`, and `--vertex-map` carriers, which spelled the same
values through presets and the channel grammar; the index carriers
(`palette-index`, `palette-layers`) write indices and palette tables
rather than values, so they stay their own deferred design in
[orphaned options](orphaned-options.md#deferred-features).

## Computed occlusion

Occlusion computed from the voxel geometry, each face corner reading the
voxels that meet there. It varies across a surface, so it is the first
per-face value: `computedOcclusion` arrives as a supplied symbol when
`--atlas unwrap` and the [reserved shape](#shapes) land, and whoever
builds the unwrap atlas builds the shape with it, since every other
value is per-material and an unwrap atlas of per-material values is the
palette atlas with redundant texels.

The symbol is a per-face vec1 in `0..1`, `1` fully open. It mixes like
any value, and the three tuning flags the old design carried are one
expression each:

```
--value ao "lerp(1, computedOcclusion, 0.8)"   # strength 0.8
--value ao "max(computedOcclusion, 0.2)"       # min brightness 0.2
--write-png srgb ao turret-ao.png              # color space: the token
```

As a texture it needs the unwrap texel domain, a per-face PNG under
`--atlas unwrap` or a second unwrap UV set riding beside the palette
maps under `--atlas palette`. Through `--write-vertex` it writes
corner-exact, and greedy merging splits a quad only where its corner
occlusion disagrees. Whether per-corner is the true domain the per-face
texels derive from is a call for when it is built. A sampled
neighborhood model, a radius and a falloff curve, is a possible
extension beyond the discrete corner method.

A profile reaches occlusion the way the command line does, the symbol in
its values: the `baked-ao` example in the
[profile language](profile-language.md#user-defined-profiles) bakes it
into the standard occlusion slot.

## Profiles

A profile is a named set of values and outputs. Five are built in:
`defaults`, `albedo`, `orm`, `emissive`, and the `pbr` bundle, shipping in
the binary, so `--profile pbr` works before any `.vxlconfig` exists. The
rest are user-defined under `.vxlconfig`'s `mesh.profiles` key, loaded
through tyt-preferences; a config profile sharing a built-in's name
replaces it wholesale, and extending one is a new name with `basedOn`. The
schema, the built-in definitions, and the user-defined examples live in
the [profile language](profile-language.md). Hyphenated profile names take
camel-case value names, since `-` is subtraction in the language: a
`metallic-smoothness` profile would bake `metallicSmoothness`.

`--profile <profile>` applies the profile's values as if each were a
`--value` at the flag's own position, `basedOn` first: depth-first in list
order, every profile visited once, cycles an error, the profile's own
values last. So

```
--value a "0.5" --profile pbr --value b "a * 2"
```

defines `a`, then pbr's values, then `b`, and redefinition stays let-style
throughout, so a `--value` after the flag overrides a profile value and
every output picks up the override.

`--profile` applies values and queues outputs, and the outputs fire by
default; `--write-profile false` keeps every named profile a values
mixin. A profile's outputs map holds one entry per value, each field
named for the flag it fires as and carrying its arguments minus the
value the entry key names: `png` and `json` are `{transfer, file}`
records firing `--write-png` and `--write-json` on `{stem}` templates;
`vertex` is a `{transfer, target}` record firing `--write-vertex`;
`slots` and `slotFiles` list the properties `--slot` and `--slot-file`
fill; `slotExtras` lists `{transfer, name}` records firing
`--slot-extra`, and `slotExtraFiles` the names `--slot-extra-file`
fills. The profile spells its own writes, and the merged entries fire as
they are, under three rules:

1. A `png` or `json` writes its file, and a `vertex` its attribute.
   Entries naming one json file merge
   into it, each value under its own symbol, the way repeated
   `--write-json` flags do. An entry carries one `png` record at most,
   so a value written to a second file is a hand-written `--write-png`
   beside the profile.
2. `slots` and `slotExtras` embed their value; `slotFiles` and
   `slotExtraFiles` reference the entry's `png` instead, and the
   referencing pair carries no transfer, the `png` record already naming
   it.
3. A plain value lands as a material field or an inline extras entry,
   since a factor cannot ride in a file, and a `png` no slot references
   still writes, which is how the user-defined `mse` example writes its
   mask beside the `emissiveStrength` it sets.

So with `--to turret.glb`, `--profile orm` expands to

```
--value occlusionStrength "default(occlusionStrength, 1)"   # basedOn: defaults
--value roughnessFactor "default(roughnessFactor, 1)"       # basedOn: defaults
--value metallicFactor "default(metallicFactor, 1)"         # basedOn: defaults
--value orm "rgb(occlusionStrength, roughnessFactor, metallicFactor)"
--slot orm occlusionTexture                                 # slots: embeds
--slot orm metallicRoughnessTexture                         # slots: embeds
```

with the three unused defaults elided. The config `orm-files` variant in
the [profile language](profile-language.md#user-defined-profiles) adds
`"png": { "transfer": "linear", "file": "{stem}-orm.png" }` to the same
entry and moves the properties to `slotFiles`, so the file writes and
the slots reference it:

```
# the values as above
--write-png linear orm turret-orm.png            # png
--slot-file turret-orm.png occlusionTexture      # slots: references
--slot-file turret-orm.png metallicRoughnessTexture
```

`--profile emissive` shows the plain slots, factors landing beside the
embedded texture:

```
# the defaults elided
--value maxStrength "max(emissiveStrength)"
--value emissive "emissiveFactor * emissiveStrength / max(maxStrength, 0.001)"
--value white "rgb(1, 1, 1)"
--slot emissive emissiveTexture        # array: embeds
--slot white emissiveFactor            # plain: material field
--slot maxStrength emissiveStrength    # plain: material field
```

Outputs merge across profiles the way values do. Entries key on their
value, a later profile's entry replacing an earlier one wholesale, so an
entry's shape, embedding or referencing, travels with it; naming `pbr`
beside `albedo` is legal and the identical entries simply win. The merged
entries then flatten to their destinations, a file keying on its name and
a slot on its property, and a later profile still wins a destination two
values contest. Inside one profile that contest fails at load: there is
no later profile to win it, and two entries claiming one png file, slot,
extras name, or vertex attribute are two hand-written flags colliding,
spelled in config.
Entries sharing one json file stay the merge rule 1 spells. The
profile's `basedOn` carries values alone; outputs
opt into parents through a `basedOn` key of their own inside the outputs
map, merged the same way, deps-first with the profile's own entries
last. The two lists are separate so a profile takes a parent's values
with or without its writes, and `basedOn` is a reserved key in the
outputs map, so no value of that name takes an entry. This is what makes
`pbr` two bare lists: its values and its outputs are its members',
nothing respelled. An explicit flag
beats a profile: a hand-written writer or slot on a profile's path or
property replaces the profile's entry wherever it sits on the line, while
two hand-written flags colliding stays the error it always was. An
explicit `--write-profile true` whose merged outputs come to nothing
errors, `--write-profile` without a `--profile` included, so `--profile
defaults --write-profile true` fails loudly. The default write skips
that error, so `--profile defaults` still serves as a plain values mixin
ahead of hand-written flags.

Two rules keep a config honest: `slotFiles` or `slotExtraFiles` on an
entry with no `png` errors, since there is no file to reference, and an
entry whose `png` transfer fights a `slotFiles` slot is the
`--slot-file` cross-check spelled in config rather than flags. No
built-in carries a `png`, so none can hit either.

Files take their names from `{stem}` templates, `--stem` replacing the
default, the output mesh's own stem. A template spells its file name
literally, so a hyphenated profile keeps its hyphens: a
`metallic-smoothness` profile writes `turret-metallic-smoothness.png` even
though the value it bakes is `metallicSmoothness`.

`emissiveStrength` has a minimum of 0 and no maximum, so the `emissive`
profile and the `mse` example both normalize by the palette's strongest
strength and send that strength to the `emissiveStrength` slot. Packing it
raw would put an unbounded value in an 8-bit channel, which errors on the
first material above 1 rather than clamping. Normalizing is also the
convention the packed maps target, a `0..1` mask in the image and the
intensity on the material. The two agree on this deliberately, and merging
them is harmless, since each binds the slot to the same `maxStrength`.

Every profile spells its own defaults through the `defaults` mixin, which
is what makes a profile never fail on a missing property: a property no
layer supplies, or a material that leaves it unset, takes the spec default
the mixin names. A hand-written `--value` gets no such guarantee, since
nothing auto-defaults.

The emissive pair is the whole profile: the image carries each material's
color scaled into `0..1` of the palette's strongest strength, and the factor
slot carries that strength back, so absolute brightness survives a `0..1`
image. The white `emissiveFactor` beside them is the third piece, leaving the
image untinted where glTF would otherwise multiply it by black.

## Texture-Map Value Language Grammar

This document gives the grammar in two forms. The first is dimension-typed:
vec1, vec2, vec3, and vec4 expressions each get their own nonterminals, so
the same-dimension rules for the operators, the vec1 broadcast rules, and
the result type of every swizzle are encoded directly in the grammar. It
encodes the dimension axis only; the shape axis (plain versus array, see
[Shapes](#shapes)) is enforced by checking rules. The second form is a
compact untyped grammar plus those checking rules. It is the form to
implement, since a parser cannot know a symbol's dimension from syntax
alone.

A vec1 is a scalar. It is named like the other vectors so that swizzling and
broadcasting work uniformly across all four dimensions.

Precedence is encoded by stratification. From loosest to tightest binding:
additive (`+ -`), multiplicative (`* /`), unary minus, postfix (swizzle,
member, index), primary. Left recursion gives left-to-right associativity for
`+ - * /`.

### Dimension-typed BNF

```bnf
; ============================================================
; Start symbol: an expression of any dimension
; ============================================================

<expr>          ::= <vec1-expr>
                  | <vec2-expr>
                  | <vec3-expr>
                  | <vec4-expr>

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
                  | <symbol>
                  | "(" <vec1-expr> ")"
                  | "r" "(" <vec1-expr> ")"
                  | "min" "(" <vec1-expr> ")"                   ; palette minimum
                  | "min" "(" <vec1-expr> "," <vec1-expr> ")"
                  | "max" "(" <vec1-expr> ")"                   ; palette maximum
                  | "max" "(" <vec1-expr> "," <vec1-expr> ")"
                  | "sum" "(" <vec1-expr> ")"                   ; palette sum
                  | "avg" "(" <vec1-expr> ")"                   ; palette mean
                  | "abs" "(" <vec1-expr> ")"
                  | "pow" "(" <vec1-expr> "," <vec1-expr> ")"
                  | "mod" "(" <vec1-expr> "," <vec1-expr> ")"
                  | "clamp" "(" <vec1-expr> "," <vec1-expr> "," <vec1-expr> ")"
                  | "lerp" "(" <vec1-expr> "," <vec1-expr> "," <vec1-expr> ")"
                  | "step" "(" <vec1-expr> "," <vec1-expr> ")"
                  | "smoothstep" "(" <vec1-expr> "," <vec1-expr> ","
                                     <vec1-expr> ")"
                  | "floor" "(" <vec1-expr> ")"
                  | "ceil" "(" <vec1-expr> ")"
                  | "round" "(" <vec1-expr> ")"
                  | "default" "(" <symbol> "," <vec1-expr> ")"

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

<vec2-prim>     ::= <symbol>
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
                  | "pow" "(" <vec2-expr> "," <vec2-expr> ")"
                  | "pow" "(" <vec2-expr> "," <vec1-expr> ")"
                  | "mod" "(" <vec2-expr> "," <vec2-expr> ")"
                  | "mod" "(" <vec2-expr> "," <vec1-expr> ")"
                  | "clamp" "(" <vec2-expr> "," <vec2-expr> "," <vec2-expr> ")"
                  | "clamp" "(" <vec2-expr> "," <vec1-expr> "," <vec1-expr> ")"
                  | "lerp" "(" <vec2-expr> "," <vec2-expr> "," <vec2-expr> ")"
                  | "lerp" "(" <vec2-expr> "," <vec2-expr> "," <vec1-expr> ")"
                  | "step" "(" <vec2-expr> "," <vec2-expr> ")"
                  | "step" "(" <vec1-expr> "," <vec2-expr> ")"
                  | "smoothstep" "(" <vec2-expr> "," <vec2-expr> ","
                                     <vec2-expr> ")"
                  | "smoothstep" "(" <vec1-expr> "," <vec1-expr> ","
                                     <vec2-expr> ")"
                  | "floor" "(" <vec2-expr> ")"
                  | "ceil" "(" <vec2-expr> ")"
                  | "round" "(" <vec2-expr> ")"
                  | "default" "(" <symbol> "," <vec2-expr> ")"

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

<vec3-prim>     ::= <symbol>
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
                  | "pow" "(" <vec3-expr> "," <vec3-expr> ")"
                  | "pow" "(" <vec3-expr> "," <vec1-expr> ")"
                  | "mod" "(" <vec3-expr> "," <vec3-expr> ")"
                  | "mod" "(" <vec3-expr> "," <vec1-expr> ")"
                  | "clamp" "(" <vec3-expr> "," <vec3-expr> "," <vec3-expr> ")"
                  | "clamp" "(" <vec3-expr> "," <vec1-expr> "," <vec1-expr> ")"
                  | "lerp" "(" <vec3-expr> "," <vec3-expr> "," <vec3-expr> ")"
                  | "lerp" "(" <vec3-expr> "," <vec3-expr> "," <vec1-expr> ")"
                  | "step" "(" <vec3-expr> "," <vec3-expr> ")"
                  | "step" "(" <vec1-expr> "," <vec3-expr> ")"
                  | "smoothstep" "(" <vec3-expr> "," <vec3-expr> ","
                                     <vec3-expr> ")"
                  | "smoothstep" "(" <vec1-expr> "," <vec1-expr> ","
                                     <vec3-expr> ")"
                  | "floor" "(" <vec3-expr> ")"
                  | "ceil" "(" <vec3-expr> ")"
                  | "round" "(" <vec3-expr> ")"
                  | "default" "(" <symbol> "," <vec3-expr> ")"

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

<vec4-prim>     ::= <symbol>
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
                  | "pow" "(" <vec4-expr> "," <vec4-expr> ")"
                  | "pow" "(" <vec4-expr> "," <vec1-expr> ")"
                  | "mod" "(" <vec4-expr> "," <vec4-expr> ")"
                  | "mod" "(" <vec4-expr> "," <vec1-expr> ")"
                  | "clamp" "(" <vec4-expr> "," <vec4-expr> "," <vec4-expr> ")"
                  | "clamp" "(" <vec4-expr> "," <vec1-expr> "," <vec1-expr> ")"
                  | "lerp" "(" <vec4-expr> "," <vec4-expr> "," <vec4-expr> ")"
                  | "lerp" "(" <vec4-expr> "," <vec4-expr> "," <vec1-expr> ")"
                  | "step" "(" <vec4-expr> "," <vec4-expr> ")"
                  | "step" "(" <vec1-expr> "," <vec4-expr> ")"
                  | "smoothstep" "(" <vec4-expr> "," <vec4-expr> ","
                                     <vec4-expr> ")"
                  | "smoothstep" "(" <vec1-expr> "," <vec1-expr> ","
                                     <vec4-expr> ")"
                  | "floor" "(" <vec4-expr> ")"
                  | "ceil" "(" <vec4-expr> ")"
                  | "round" "(" <vec4-expr> ")"
                  | "default" "(" <symbol> "," <vec4-expr> ")"

; ============================================================
; Swizzle selectors
; Any sequence of 1-4 components valid for the source; repeats
; allowed, and the result may be wider or narrower than the
; source. Selector counts (lengths 1-4): 4 from vec1, 30 from
; vec2, 120 from vec3, 340 from vec4.
; ============================================================

<c1>            ::= "r"                       ; components of a vec1
<c2>            ::= "r" | "g"                 ; components of a vec2
<c3>            ::= "r" | "g" | "b"           ; components of a vec3
<c4>            ::= "r" | "g" | "b" | "a"     ; components of a vec4

<swiz-1-of-1>   ::= <c1>
<swiz-2-of-1>   ::= <c1> <c1>
<swiz-3-of-1>   ::= <c1> <c1> <c1>
<swiz-4-of-1>   ::= <c1> <c1> <c1> <c1>

<swiz-1-of-2>   ::= <c2>
<swiz-2-of-2>   ::= <c2> <c2>
<swiz-3-of-2>   ::= <c2> <c2> <c2>
<swiz-4-of-2>   ::= <c2> <c2> <c2> <c2>

<swiz-1-of-3>   ::= <c3>
<swiz-2-of-3>   ::= <c3> <c3>
<swiz-3-of-3>   ::= <c3> <c3> <c3>
<swiz-4-of-3>   ::= <c3> <c3> <c3> <c3>

<swiz-1-of-4>   ::= <c4>
<swiz-2-of-4>   ::= <c4> <c4>
<swiz-3-of-4>   ::= <c4> <c4> <c4>
<swiz-4-of-4>   ::= <c4> <c4> <c4> <c4>

; ============================================================
; Lexical grammar
; Whitespace separates tokens and is otherwise insignificant,
; except inside a backtick-quoted symbol (literal) and around
; the postfix dot and index bracket (forbidden). Not modeled
; below.
; ============================================================

<num>           ::= <digits>
                  | <digits> "." <digits>
                  | "." <digits>

<digits>        ::= <digit>
                  | <digit> <digits>

<digit>         ::= "0" | "1" | "2" | "3" | "4"
                  | "5" | "6" | "7" | "8" | "9"

; Bare identifiers start with a letter or underscore so they can
; never be confused with <num>; backtick-quote a symbol to allow
; spaces, a leading digit, or a reserved name.
<symbol>        ::= <ident>
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
```

### Untyped grammar + checking rules (implementation form)

Because a `<symbol>` (and `default(...)`) can be of any dimension, the typed
grammar above is ambiguous for a parser that does not know what each symbol
refers to. The practical structure: parse with the untyped grammar below, then
run a checker over the AST using the dimension and shape rules that follow.

```bnf
<u-expr>     ::= <u-expr> "+" <u-term>
               | <u-expr> "-" <u-term>
               | <u-term>

<u-term>     ::= <u-term> "*" <u-unary>
               | <u-term> "/" <u-unary>
               | <u-unary>

<u-unary>    ::= "-" <u-unary>
               | <u-post>

<u-post>     ::= <u-post> "." <member>
               | <u-post> "[" <u-expr> "]"
               | <u-prim>

<u-prim>     ::= <num>
               | <symbol>
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
               | "abs"  "(" <u-expr> ")"
               | "pow"  "(" <u-expr> "," <u-expr> ")"
               | "mod"  "(" <u-expr> "," <u-expr> ")"
               | "clamp" "(" <u-expr> "," <u-expr> "," <u-expr> ")"
               | "lerp" "(" <u-expr> "," <u-expr> "," <u-expr> ")"
               | "step" "(" <u-expr> "," <u-expr> ")"
               | "smoothstep" "(" <u-expr> "," <u-expr> "," <u-expr> ")"
               | "floor" "(" <u-expr> ")"
               | "ceil" "(" <u-expr> ")"
               | "round" "(" <u-expr> ")"
               | "default" "(" <symbol> "," <u-expr> ")"

; A member is always a swizzle: 1-4 components over {r,g,b,a},
; repeats allowed. Every value is a vector, so there is nothing
; else it could be.
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
| `e.s` (swizzle)          | every component of `s` exists in `dim(e)`: `r` always, `g` needs >= 2, `b` needs >= 3, `a` needs 4; `1 <= len(s) <= 4`, repeats allowed | `len(s)`              |
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
| `<symbol>`               | dimension of the value it names                                                                                                         | that dimension        |
| `default(sym, e)`        | `dim(sym) = dim(e)` where `sym` has a value                                                                                             | `dim(e)`              |

A vec1 broadcasts on the right of `/`, as the exponent of `pow`, and on
either side of `*`.

Shape rules, with a value either plain or an array over the effective palette
(see [Shapes](#shapes)):

1. A property symbol is an array; a literal is plain; a `--value` symbol has
   its definition's shape.
2. The elementwise constructs, the operators, the constructors, swizzles,
   binary `min`/`max`, `abs`, `pow`, `mod`, `clamp`, `lerp`, `step`,
   `smoothstep`, `floor`/`ceil`/`round`, and `default()`, pair arrays
   element by element and broadcast plain values; the result is an array
   when any operand is.
3. The reductions, unary `min`/`max` and `sum`/`avg`, require an array and
   yield a plain value, computed per component across the palette.
4. `e[i]` requires `e` an array and `i` a plain exact non-negative integer
   below the material count, and yields a plain value.
5. A writer takes whatever shape its destination holds: a PNG an array, a
   `--slot` factor a plain value, JSON either.

### Functions

One item per function. Dimensions and shapes follow the tables above.

1. `r(x)`, `rg(x, y)`, `rgb(x, y, z)`, and `rgba(x, y, z, w)` build a
   vector from vec1 parts. This is how channels pack into a map:

   ```
   rgb(occlusion, roughness, metallic)   # the orm pack
   ```

2. Unary `min(e)` and `max(e)` reduce an array across the palette, per
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
   across the palette:

   ```
   avg(baseColorFactor)   # the palette's mean color
   ```

5. `abs(e)` is the componentwise magnitude:

   ```
   abs(tint - avg(tint))   # each material's spread around the mean
   ```

6. `pow(a, b)` is the componentwise exponent. A vec1 exponent broadcasts
   across `a`, and `pow(vec1, vecN)` errors, matching the rule for `/`:

   ```
   pow(roughnessFactor, 2.2)   # steepens the roughness curve
   ```

7. `mod(a, b)` is the floored remainder, `a - b * floor(a / b)`, the form
   that wraps. `mod(a, 0)` is non-finite and errors like any other:

   ```
   mod(hue + 0.618, 1)   # wraps back into 0..1
   ```

8. `clamp(x, lo, hi)` pins each component into `lo..hi`. A component with
   `lo > hi` errors. An explicit `clamp` is the author naming a bound,
   which is exactly what the write-time rules ask for:

   ```
   clamp(strength / 4, 0, 1)   # the author's own bound
   ```

9. `lerp(a, b, t)` is `a + (b - a) * t`. `t` is unrestricted, so it
   extrapolates outside `0..1`. The name is HLSL's, over GLSL's `mix`,
   because it says what the function does:

   ```
   lerp(orm, rgb(1, 1, 1), 0.25)   # a quarter of the way to white
   ```

10. `step(edge, x)` is 0 where `x < edge` and 1 elsewhere, the mask maker:

    ```
    step(0.001, emissiveStrength)   # 1 for every material that emits
    ```

11. `smoothstep(lo, hi, x)` is the Hermite ramp: 0 at `lo`, 1 at `hi`,
    held flat outside. A component with `lo >= hi` errors, one step
    stricter than `clamp`, since the ramp divides by `hi - lo`:

    ```
    smoothstep(0.2, 0.8, occlusion)   # eases a mask edge
    ```

12. `floor(e)` and `ceil(e)` snap each component to the integer below or
    above, and `round(e)` to the nearest, halves away from zero:

    ```
    round(smoothness * 4) / 4   # five even levels
    ```

13. `default(sym, fallback)` evaluates to `sym` where it has a value and
    to `fallback` where it does not: a `--value` symbol not yet defined, a
    property no layer supplies, or a material that leaves it unset, filled
    per element. `sym` is a bare or backtick-quoted symbol, and `fallback`
    is any expression of the same dimension. Nothing auto-defaults, and an
    unbound symbol is an error, so a robust expression spells the spec
    default itself:

    ```
    default(occlusionStrength, 1)   # the glTF default where unset
    ```

### Notes

**Backtick quoting.** Backticks quote a symbol whose name a bare identifier
cannot spell: spaces, a leading digit, or a reserved name. `foo bar` always
lexes as two separate symbols; the value is written `` `foo bar` ``. Double
quotes are not symbol quoting; they stay reserved for future string literals.
In a shell, single-quote an expression holding backticks so the shell does
not read them as command substitution.

**Reserved names.** The function names `r`, `rg`, `rgb`, `rgba`, `min`,
`max`, `sum`, `avg`, `abs`, `pow`, `mod`, `clamp`, `lerp`, `step`,
`smoothstep`, `floor`, `ceil`, `round`, and `default` are keywords. A
property sharing one is reached by backtick-quoting: `` `min` `` is the
symbol, `min(...)` the function.

**Swizzle rules.** A swizzle is any sequence of 1-4 components, repeats
allowed, where every component exists in the source: `r` always works, `g`
needs dim >= 2, `b` needs dim >= 3, `a` needs dim 4. Every dimension can be
swizzled, including vec1: `x.r` is the identity, and repeats splat upward, so
`x.rr` is a vec2 and `x.rrrr` is a vec4. Results can be wider or narrower than
the source: `v2.rrgg` is a vec4 and `v4.r` is a vec1. With vec1 splats
available, `r(x)` duplicates `x.r` and `rg(x, x)` duplicates `x.rr`; the
constructors are still needed when the arguments differ, as in `rg(x, y)`.
Components spell `rgba` only: `xyzw` aliases were rejected, a second spelling
for the same component being the redundancy the value kinds deleted. Adding
them later is a compatible relaxation, since the parser takes any identifier
after the dot and only the checker limits the components.

```
baseColorFactor.rgb   # vec4 to vec3, dropping alpha
orm.g                 # one channel, roughness
0.5.rrr               # a grey vec3 splat from one number
tint.rrgg             # wider than its source
```

**Precedence and associativity.** From tightest to loosest: postfix (swizzle,
member, index), unary `-`, `* /`, `+ -`. Postfixes chain left to right, so
`baseColorFactor[0].rgb` and `baseColorFactor.rgb[0]` both parse, and name
the same value. Unary minus nests, so `- -x` is valid. There is no `--` token
in the expression language, so `--value` never collides with it.

**pow, not `^`.** The operator was rejected: the shader languages this one
borrows its swizzles from spell exponent `pow()` and read `^` as XOR, and
it was the grammar's only right-associative stratum. The character is
reserved: the lexer rejects `^`, keeping it free for a future meaning.

**The function set stays small.** `sqrt(x)` is `pow(x, 0.5)`, `fract(x)` is
`mod(x, 1)`, and a signed remap is `n * 0.5 + 0.5`, so none of them is a
function.

**Lexing.** Whitespace separates tokens and is otherwise insignificant, with
two exceptions: inside a backtick-quoted symbol it is literal, and in a
postfix it is forbidden. A postfix is attached: the dot and its member hug
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
`--write-png` file is bound to nothing, a `--write-json` key is read only
by your own code, and a `--slot-extra` fills a slot the format does not
define, so each names its encoding itself; the PNG token also feeds the
`--slot-file` cross-check and the file's own chunks. A value-form `--slot`
carries no token because its slot's fixed encoding is the definition, not
an inference: `--slot albedo baseColorTexture` embeds an sRGB image by what
`baseColorTexture` is; `--slot-file` carries none either, the named file's
own chunks saying what it is.

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
the same. A value-form `--slot` stamps its embedded image identically, the
slot's encoding in the chunks, and PNG's newer `cICP` chunk, which names a
linear transfer exactly, can join later without disturbing any of this.

**Write-time errors.** A non-finite component (NaN or infinity, as from
`0 / 0`) is an error wherever it appears. Clamping is always the author's,
written `clamp()`. The destination decides the range, and only an image
has one: a PNG requires every component in `0..1`, so `1.5` into a PNG
errors while `1.5` into a JSON field is fine, which is how an unbounded
property like `emissiveStrength` travels.

**Redefinition.** `--value` may redefine any symbol, a property or an earlier
value. The right side is evaluated against the bindings visible at that
point, so `--value roughnessFactor "pow(roughnessFactor, 2)"` reads the
property and rebinds the name, and later expressions see the new value. There is no
recursion.

## The language crate

The language ships as its own crate, referencing none of the vxl
crates. The crate owns the whole language: `parse` takes text to a
syntax tree, `check` takes the tree and each symbol's type to the
result type, and `eval` takes the tree and each symbol's value to the
result. The symbol information comes in through an environment the
caller supplies:

```rust
let tree = parse("rgb(occlusionStrength, roughnessFactor, metallicFactor)")?;

let ty = check(&tree, &env)?; // env: name -> Option<Type>, shape x dimension

let value = eval(&tree, &env)?; // env: name -> Option<Value>, plain or array
```

To the crate an array is a length, so the palette is vxl's
interpretation: vxl binds the effective palette into the environment in
atlas-texel order and keeps the edges, the transfer encoding, the png
sizing, and the slot cross-checks, which is where the linear-floats
rule already puts them. A future domain like the
[per-face shape](#shapes) is another length to evaluate over, so the
crate never learns what a domain means.

The tree stays internal; `parse`, `check`, and `eval` are the API.
Exporting the tree would split the semantics from the grammar, every
new function landing in two crates. `check` stays public beside `eval`:
[loading](profile-language.md#loading) wants `parse` alone, and a
hand-written flag wants its shape and dimension errors before anything
evaluates.

## Open questions

The open questions for this page and the
[profile language](profile-language.md) live on
[their own page](open-questions.md).
