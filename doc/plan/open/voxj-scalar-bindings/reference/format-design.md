# Scalar bindings: target spec text

_Part of the [voxj scalar-bindings plan](../README.md)._

The complete replacement text for every section of
[voxel-json-file-format.md](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md)
this change touches, drafted against the spec as of commit `7527d30`.
Approved by the owner 2026-07-16, closing phase 1; phase 2 copies it into the
spec in one commit. Sections not listed here do not change. A line reading `[unchanged: ...]` splices the named text through from
the current spec verbatim. The four open questions closed in owner review
2026-07-15, and a second review the same day merged the two object layer
lists into one ordered `layers` list with channels derived from palette
shape; the resolutions are folded in below and recorded in the
[README's decisions](../README.md#decisions).

One rename applies to the entire spec, spliced `[unchanged]` text and
otherwise-untouched sections included: the binding field and concept
`attribute` becomes `property` (owner review 2026-07-16). glTF, the
recommended vocabulary, reserves attribute for per-vertex mesh data and calls
material parameters properties. The Attributes section retitles to Properties
and its `#attributes` anchor becomes `#properties`; the glTF conventions
table's Attribute column and the Value Pool Kinds table's Typical attributes
column follow.

A second naming pass also applies spec-wide (owner review 2026-07-17,
during the phase 2 review): `arrayBindings` / `scalarBindings` become
`arrayProperties` / `scalarProperties`, and an entry's fields are `name` /
`valuePool` / `valueIndex` (formerly `property`, `poolRef`, `valueRef`). An
entry is the property itself, so the prose says array property and scalar
property, not binding; reference fields name what they point at, like
`layers` and glTF's `"mesh": 0`, and `valueIndex` matches the "value-index"
the materials rules already use (README decision 12). The text below is
updated in place. The same review dropped the trailing validation note,
which only restated facts the Objects section and rule 11 already state,
and renamed the hierarchy fields: `hierarchyNodes` becomes `nodes` and
`rootHierarchyNodes` becomes `rootNodes`, matching `childNodes` and glTF's
`nodes`; the prose term hierarchy node and the TypeScript `HierarchyNode`
interface are unchanged (README decision 13). A further 2026-07-17 review
made `materials` row-major: one row per material, a value-index per array
property in property order, with `M = materials.length` (README decision
14).

## Objects

Replaces the section body.

---

An object is one voxel volume of pure geometry. Aside from its grid `origin`,
it carries no transform of its own: rotation, scale, and placement come from
the hierarchy node that references it.

```jsonc
{
  "name": "Object A",

  // [X, Y, Z] size in voxels
  "bounds": [1, 1, 1],

  // [X, Y, Z] integer translation from the placing node to the grid's min
  // corner
  "origin": [0, 0, 0],
  "voxelPositions": { "encoding": "raw-json", "data": [[0, 0, 0]] },

  // layers: palette references, ordered back to front
  "layers": [0],
  // one channel per sampled layer; each is one material index per voxel
  "voxelSamples": { "encoding": "raw-json", "data": [[0]] },
}
```

An object carries one layer list, `layers`, an array of palette indices
ordered back to front. Each layer supplies all of its palette's
properties: scalar properties one value for the whole object, array
properties one value per voxel through a sample channel. A palette may appear in `layers`
any number of times. Layers combine by overriding: contributions apply in
`layers` order and each property takes its value from the last layer that
supplies it, so later layers override earlier ones (see
[Palettes](#palettes) for the full resolution).

A layer is **sampled** when its palette has at least one material, `M > 0`
(see [Palettes](#palettes)). A palette with no materials, `materials: []`,
is never sampled, so a scalar-only palette carries no per-voxel data.
`voxelSamples` carries exactly one channel per sampled layer, in `layers`
order: channel `c` belongs to the `c`-th sampled layer, and its samples are
material indices into that layer's palette.

`voxelPositions` and `voxelSamples` are encoded blocks (see
[Voxel Encoding](#voxel-encoding)). Each voxel has a position `(x, y, z)` and
one material sample per sampled layer. The number of voxels is implicit: it
is the number of positions decoded from `voxelPositions`. Positions within an
object must be unique, and every voxel samples every sampled layer.

[unchanged: the `bounds` paragraph, the `origin` paragraph, and the closing
voxel-order paragraph]

---

## Sample Encodings

Subsection of Voxel Encoding. Replaces the intro paragraph, rewords the three
encodings, and retitles the example; the example body is unchanged.

---

A sample block holds one channel per sampled layer, in `layers` order. Each
channel gives, for every voxel in the position block's voxel order, a material
index into that layer's palette.

1. `raw-json`: one channel per sampled layer, each a plain array of that
   layer's material index for every voxel: `[[l0v0, l0v1, ...], [l1v0, l1v1,
   ...], ...]`.
2. `rle-json`: one channel per sampled layer; each channel is a flat
   run-length encoding `[value1, count1, value2, count2, ...]`. Counts are
   positive integers and, in every channel, sum to the number of voxels.
3. `packed-base64`: one bit-packed channel per sampled layer. For the channel
   of a layer whose palette has `M` materials, each voxel's material index is
   packed at fixed width `b = max(1, bitLength(M - 1))` bits, MSB-first, 8 per
   byte, with the final byte zero-padded; the width is derived from `M` and
   not stored. `data` is one base64 string per sampled layer, in `layers`
   order, each encoding exactly `ceil(voxelCount * b / 8)` bytes. This is the
   same packing scheme as the `bitmap-base64` position encoding, which is its
   `b = 1` special case. An empty object has one `""` per sampled layer. Best
   for incoherent or many-material objects, where `rle-json` would approach
   one run per voxel.

#### Example: two sampled layers over four voxels; layer 0 material indices `0, 0, 0, 1` (palette `M = 2`) and layer 1 material indices `2, 2, 3, 3` (palette `M = 4`), in the position block's voxel order

[unchanged: the example code block]

---

## Voxel Order

Replaces the first sentence; the numbered list and the closing paragraph are
unchanged.

---

The position block defines the object's single canonical voxel order, and
every sample channel, one per sampled layer, is in that same order,
voxel-for-voxel, for every combination of position and sample encoding:

[unchanged: the numbered list and the re-encoding paragraph]

---

## Value Pools

Replaces the intro paragraph's last sentence only.

---

Value pools live in `main.runtimeState.valuePools`, a shared array referenced
by index, siblings of `objects` and `palettes`. A value pool holds `values`,
all of one value-shape given by `kind`. `kind` tags the shape of the values.
Palettes reference pools by index: an array property references a whole
pool and a scalar property a single cell of one (see [Palettes](#palettes)).

---

## Palettes

Replaces the section body up to the Properties subsection, and adds a new
Sharing Idioms subsection before Properties.

---

A palette binds property names to shared [value pools](#value-pools), then
lists the distinct materials it uses as rows over those pools. Properties
come in two arities, and a palette may carry either, both, or neither. An
**array property** binds to a whole pool and takes one value-index per
material, so its value varies per material. A **scalar property** is pinned
to a single pool cell of any kind,
`valuePools[valuePool].values[valueIndex]`, one value for the whole palette.
A voxel samples a material in each sampled layer by its index in that
layer's palette. A palette may be referenced by any number of layers and
objects (see [Objects](#objects)).

A scalar property wires a name to a value; any arithmetic, such as
`emissiveStrength` multiplying `emissiveFactor`, comes from the property
vocabulary. Within one palette a name appears in `arrayProperties` or
`scalarProperties`, so a single layer never conflicts with itself.

A material is one row of value-indices, one per array property, so the
material count `M` is `materials.length`; with no array properties every
row is empty:

```jsonc
{
  // ordered array properties; each binds a property name to a value pool
  // index. Order fixes the value-index order in each `materials` row.
  "arrayProperties": [
    { "name": "baseColorFactor", "valuePool": 0 },
    { "name": "metallicFactor", "valuePool": 1 },
  ],

  // scalar properties; each pins a property name to one pool cell, one
  // value for the whole palette
  "scalarProperties": [
    { "name": "emissiveStrength", "valuePool": 2, "valueIndex": 1 },
  ],

  // one row per material, a value-index per array property, in property
  // order. materials[m][b] is a value-index into the pool bound by
  // arrayProperties[b]. A voxel samples material m in [0, M); resolve it by
  // reading across its row:
  //   material 0 = {
  //     baseColorFactor: pool0.values[0],
  //     metallicFactor: pool1.values[2]
  //   }
  "materials": [
    [0, 2], // material 0
    [1, 0], // material 1
    [2, 1], // material 2
  ],
}
```

A voxel's property values resolve from its object's `layers` as follows:

1. Each layer supplies its palette's properties. A scalar property supplies
   its `name` as `valuePools[valuePool].values[valueIndex]`, one value for
   the whole object. An array property supplies its `name` per voxel: read
   the voxel's sample `m` from the layer's channel, a material index; array
   property `b` supplies `arrayProperties[b].name` as
   `valuePools[arrayProperties[b].valuePool].values[materials[m][b]]`. An
   unsampled layer has no channel and supplies only its scalar properties.
2. Layers override: contributions apply in `layers` order, back to front,
   and each property takes its value from the last layer that supplies it.
   Three layers supplying `{a, b, c}`, then `{a}`, then `{c}` resolve to `b`
   from the first, `a` from the second, and `c` from the third.
3. Unbound properties are left to the vocabulary; the recommended glTF
   conventions supply a default for each (see [Properties](#properties)).

### Sharing Idioms

One pool cell can supply a property at every scope without cloning anything:

1. All materials of one palette share a value: put a scalar property on that
   palette. One `layers` entry supplies both arities; nothing is listed
   twice.
2. Per-object variation over a shared palette: make small palettes of one
   scalar property each, with `materials: []` so they are never sampled, and
   list one after the shared palette. Switching an object's knob is a
   one-integer edit.
3. Single source of truth: the pool cell. Editing it updates every palette
   that references it.
4. Per-voxel variation: move the property from `scalarProperties` to
   `arrayProperties`, giving it a per-material value-index and a channel.
5. Whole-object override: list a scalar-property palette after the layer it
   overrides; the object-wide value replaces the per-voxel values for that
   property.

Idiom 2, two lamp objects sharing one base palette but glowing at different
strengths:

```jsonc
"valuePools": [
  // 0: emissive strengths, referenced by cell
  { "kind": "float", "min": 0, "max": "none", "values": [1, 5, 40] },
],

"palettes": [
  // 0: the shared base palette; the array side elided
  {
    "arrayProperties": [ /* ... */ ],
    "scalarProperties": [],
    "materials": [ /* ... */ ],
  },

  // 1: the lamp-glow knob; no materials, so it is never sampled
  {
    "arrayProperties": [],
    "scalarProperties": [
      { "name": "emissiveStrength", "valuePool": 0, "valueIndex": 1 },
    ],
    "materials": [],
  },

  // 2: the sign-glow knob: the same pool, the next cell
  {
    "arrayProperties": [],
    "scalarProperties": [
      { "name": "emissiveStrength", "valuePool": 0, "valueIndex": 2 },
    ],
    "materials": [],
  },
],

"objects": [
  // "Lamp A": the shared palette plus its own knob; the knob layer carries
  // no channel, so voxelSamples has one channel, for palette 0
  { /* ... */ "layers": [0, 1] },
  // "Neon Sign": the same base palette; switching knobs is a one-integer
  // edit in layers
  { /* ... */ "layers": [0, 2] },
]
```

---

## Properties

Retitled from Attributes; the `#attributes` anchor becomes `#properties`.
Replaces the first sentence of the intro paragraph and adds one sentence to
the emission paragraph under glTF conventions; everything else in the
subsection, including the table, is unchanged apart from the global rename.

---

A property is a named material parameter, listed in a palette's
`arrayProperties[].name` and `scalarProperties[].name`. The format wires
properties without defining them: the name carries the meaning and the pool
carries the values; that pairing is all the format defines.

[unchanged: the rest of the intro paragraph, the vocabulary paragraph, the
glTF conventions intro, the table, and the color paragraph]

At the end of the emission paragraph, append:

A strength shared by a whole palette or object is typically wired as a scalar
property (see [Palettes](#palettes)).

---

## Versioning and Extensibility

Replaces item 4; the other items are unchanged.

---

4. Unknown property **names** in `arrayProperties` and `scalarProperties` are
   ignored, since properties are advisory and convention-based, so adding
   properties is backward compatible.

---

## Validation

Replaces rules 5, 6, 8, and 10 and the intro of rule 11; the section intro,
all other rules, and rule 11's per-encoding sub-items are unchanged.

---

5. Unknown keys reject in every closed structure: file, `main`,
   `runtimeState`, `editState`, object, encoding block, palette, array
   property, scalar property, value pool, transform, hierarchy node, and
   edit object. The only open points are `main.ext` and property names.
6. All indices are in range:
   1. each object `layers` entry indexes `runtimeState.palettes`.
   2. each array and scalar property `valuePool` indexes
      `runtimeState.valuePools`.
   3. each `childNodes` entry indexes `runtimeState.nodes`.
   4. each `childObjects` entry indexes `runtimeState.objects`.
   5. each `rootNodes` entry indexes `runtimeState.nodes`.
8. **Objects**, per object:
   1. `layers` is present, an array of integers, possibly empty.
   2. `voxelPositions` and `voxelSamples` are present; the Positions and
      Samples rules check their structure.
10. **Palettes** (`runtimeState.palettes`): an array, possibly empty. Each
    palette's keys are drawn only from { `arrayProperties`,
    `scalarProperties`, `materials` }.
    1. `arrayProperties` is an array, possibly empty; each array property
       has exactly the keys `name`, a non-empty string, and `valuePool`, an
       integer. `scalarProperties` is an array, possibly empty; each scalar
       property has exactly the keys `name`, a non-empty string,
       `valuePool`, an integer, and `valueIndex`, an integer.
    2. no two properties share a `name`, across `arrayProperties` and
       `scalarProperties` together.
    3. `materials` is an array of `M >= 0` rows, the material count; every
       row is an array of exactly `arrayProperties.length` integers, one
       value-index per array property in property order.
    4. every `materials[m][b]` is an integer in
       `[0, valuePools[arrayProperties[b].valuePool].values.length)`.
    5. every scalar property's `valueIndex` is an integer in
       `[0, valuePools[valuePool].values.length)`.
11. **Samples**: let `V` be the voxel count from the position block. A layer
    is sampled iff the material count `M` of its palette is greater than
    zero. `voxelSamples.data` has exactly one channel per sampled layer, in
    `layers` order, so channel `c` belongs to the `c`-th sampled layer. For
    channel `c`, let `M` be the material count of its layer's palette, and
    by encoding:

---

## File Example

Replaces the example document.

---

```jsonc
{
  "version": 1,
  "main": {
    "runtimeState": {
      "valuePools": [
        {
          "kind": "srgba-hex",
          "values": ["#FF0000FF", "#00FF00FF", "#0000FFFF"],
        },

        // one shared float pool, bound by metallicFactor and roughnessFactor
        { "kind": "float", "min": 0, "max": 1, "values": [0, 0.5, 1] },

        { "kind": "srgb-hex", "values": ["#000000", "#FF6600"] },

        { "kind": "linear-rgba-float", "values": [[1, 0, 0, 1]] },

        // emissive strengths, referenced by cell from a scalar property
        { "kind": "float", "min": 0, "max": "none", "values": [1, 5] },
      ],

      "palettes": [
        // value pool 1 is bound twice, to metallicFactor and roughnessFactor
        {
          "arrayProperties": [
            { "name": "baseColorFactor", "valuePool": 0 },
            { "name": "metallicFactor", "valuePool": 1 },
            { "name": "roughnessFactor", "valuePool": 1 },
            { "name": "emissiveFactor", "valuePool": 2 },
          ],

          "scalarProperties": [],

          // one row per material, a value-index per array property. Material
          // 2 resolves to baseColorFactor #0000FFFF, metallicFactor 0.5,
          // roughnessFactor 0, emissiveFactor #FF6600.
          "materials": [
            [0, 2, 1, 0],
            [1, 0, 1, 0],
            [2, 1, 0, 1],
          ],
        },

        // base color authored in linear form instead of hex
        {
          "arrayProperties": [{ "name": "baseColorFactor", "valuePool": 3 }],
          "scalarProperties": [],
          "materials": [[0]],
        },

        // one scalar property and no materials, so a layer referencing it is
        // never sampled: it supplies one emissive strength to the whole
        // object and carries no channel
        {
          "arrayProperties": [],
          "scalarProperties": [
            { "name": "emissiveStrength", "valuePool": 4, "valueIndex": 1 },
          ],
          "materials": [],
        },
      ],

      "objects": [
        {
          "name": "Object A",

          // Two voxels at (0, 0, 0) and (1, 0, 0).
          "bounds": [2, 1, 1],

          "origin": [0, 0, 0],

          "voxelPositions": {
            "encoding": "raw-json",
            "data": [
              [0, 0, 0],
              [1, 0, 0],
            ],
          },

          // two layers, back to front: palette 0, then palette 1. Both are
          // sampled and both bind baseColorFactor, so the later layer
          // supplies it; the other properties come from layer 0.
          "layers": [0, 1],

          // one channel per sampled layer, each a material index per voxel:
          //   layer 0 -> materials 0, 2 of palette 0
          //   layer 1 -> materials 0, 0 of palette 1
          "voxelSamples": {
            "encoding": "raw-json",
            "data": [
              [0, 2],
              [0, 0],
            ],
          },
        },

        {
          "name": "Object B",
          "bounds": [1, 1, 1],
          "origin": [0, 0, 0],
          "voxelPositions": { "encoding": "raw-json", "data": [[0, 0, 0]] },
          // the same shared palette as Object A, plus this object's own
          // emissive knob: palette 2 has no materials, so it is never
          // sampled and supplies emissiveStrength 5 to the whole object.
          // voxelSamples has one channel, for palette 0.
          "layers": [0, 2],
          "voxelSamples": { "encoding": "raw-json", "data": [[2]] },
        },
      ],

      "nodes": [
        {
          "name": "parent-1",

          "transform": {
            "position": [0, 0, 0],
            "rotation": [0, 0, 0, 1],
            "scale": [1, 1, 1],
          },

          "childNodes": [1],

          "childObjects": [0],
        },

        {
          "name": "parent-2",

          "transform": {
            "position": [0, 0, 0],
            "rotation": [0, 0, 0, 1],
            "scale": [1, 1, 1],
          },

          "childNodes": [],
          "childObjects": [1],
        },
      ],

      "rootNodes": [0],
    },
  },
}
```

---

## TypeScript Schema

Replaces the `VoxelObject` interface, the `SampleBlock` type's comments, the
`Palette` section comment and interface, and the `Binding` interface, which
splits into `ArrayProperty` and `ScalarProperty`. Everything else in the schema
is unchanged.

---

```typescript
// Pure geometry; placed only by a hierarchy node that references it.
interface VoxelObject {
  name: string;

  // [X, Y, Z] size in voxels; voxels occupy [0, X) x [0, Y) x [0, Z). Exactly
  // tight: per-axis the min voxel coordinate is 0 and the bound is the max plus
  // one; [0, 0, 0] when empty, a point at origin. No margin here (that is
  // editState). Required to decode bitmap-base64 and
  // hilbert-delta-varint-base64.
  bounds: Vec3;

  // [X, Y, Z] integer translation from the placing node to the grid's min
  // corner. Does not affect the voxel encodings.
  origin: Vec3;

  voxelPositions: PositionBlock;

  // palette indices, ordered back to front; each layer supplies all of its
  // palette's properties and later layers override earlier ones. A layer is
  // sampled iff its palette has materials (M > 0); each sampled layer
  // carries one voxelSamples channel (see Objects)
  layers: number[];

  voxelSamples: SampleBlock;
}

type SampleBlock =
  // One channel per sampled layer (in `layers` order): that layer's material
  // index for every voxel, in voxel order.
  | { encoding: "raw-json"; data: number[][] }
  // One channel per sampled layer: a flat run stream
  // [value1, count1, value2, count2, ...].
  | { encoding: "rle-json"; data: number[][] }
  // One channel per sampled layer: each voxel's material index bit-packed at
  // width b = max(1, bitLength(M - 1)) for that layer's palette material
  // count M, MSB-first, base64-encoded (same packing as the bitmap-base64
  // position encoding).
  | { encoding: "packed-base64"; data: string[] };

// ## Palettes

// A palette binds property names to value pools, then lists its materials:
// one row per material, a value-index per array property in property order,
// so the material count M is materials.length. A voxel samples material m;
// property arrayProperties[b].name takes
// valuePools[arrayProperties[b].valuePool].values[materials[m][b]], and each
// scalar property takes valuePools[valuePool].values[valueIndex], one value
// for the whole palette. Layers apply in `layers` order; each property takes
// its value from the last layer that supplies it.
interface Palette {
  arrayProperties: ArrayProperty[];

  scalarProperties: ScalarProperty[];

  materials: number[][];
}

// One property bound to a whole pool, one value-index per material.
interface ArrayProperty {
  // property name (see Properties); advisory, unknown names ignored
  name: string;

  // index into RuntimeState.valuePools
  valuePool: number;
}

// One property pinned to a single pool cell, one value for the whole
// palette.
interface ScalarProperty {
  // property name (see Properties); advisory, unknown names ignored
  name: string;

  // index into RuntimeState.valuePools
  valuePool: number;

  // index into valuePools[valuePool].values
  valueIndex: number;
}
```
