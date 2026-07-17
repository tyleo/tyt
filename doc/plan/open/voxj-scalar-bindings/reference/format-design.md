# Scalar bindings: target spec text

_Part of the [voxj scalar-bindings plan](../README.md)._

The complete replacement text for every section of
[voxel-json-file-format.md](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md)
this change touches, drafted against the spec as of commit `7527d30`. This is
the phase 1 iteration surface: the owner amends this file until approved, then
phase 2 copies it into the spec in one commit. Sections not listed here do not
change. A line reading `[unchanged: ...]` splices the named text through from
the current spec verbatim. The four open questions closed in owner review
2026-07-15, and a second review the same day merged the two object layer
lists into one ordered `layers` list with channels derived from palette
shape; the resolutions are folded in below and recorded in the
[README's decisions](../README.md#decisions).

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
bindings: scalar bindings one value for the whole object, array bindings one
value per voxel through a sample channel. A palette may appear in `layers`
any number of times. Layers combine by overriding: contributions apply in
`layers` order and each attribute takes its value from the last layer that
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
Palettes reference pools by index: an array binding references a whole pool
and a scalar binding a single cell of one (see [Palettes](#palettes)).

---

## Palettes

Replaces the section body up to the Attributes subsection, and adds a new
Sharing Idioms subsection before Attributes.

---

A palette binds attribute names to shared [value pools](#value-pools), then
lists the distinct materials it uses as rows over those pools. Bindings come
in two arities, and a palette may carry either, both, or neither. An **array binding**
binds an attribute to a whole pool and takes one `materials` column of
value-indices, so the attribute varies per material. A **scalar binding** pins
an attribute to a single pool cell, `valuePools[poolRef].values[valueRef]`,
one value for the whole palette. A voxel samples a material in each sampled
layer by its index in that layer's palette. A palette may be referenced by
any number of layers and objects (see [Objects](#objects)).

"Scalar" means single-valued: a scalar binding may reference a cell of any
pool kind.

Materials are stored column-major, one column per array binding:

```jsonc
{
  // ordered array bindings; each binds an attribute name to a value pool
  // index. Order fixes the column order in `materials`.
  "arrayBindings": [
    { "attribute": "baseColorFactor", "poolRef": 0 },
    { "attribute": "metallicFactor", "poolRef": 1 },
  ],

  // scalar bindings; each pins an attribute to one pool cell, one value for
  // the whole palette
  "scalarBindings": [
    { "attribute": "emissiveStrength", "poolRef": 2, "valueRef": 1 },
  ],

  // `materials` is column-major: one inner array per array binding (a
  // column), in binding order, so materials.length == arrayBindings.length.
  // Every column has the same length, the material count M. materials[b][m]
  // is a value-index into the pool bound by column b. A voxel samples
  // material m in [0, M); resolve it by reading down the columns:
  //   material 0 = { baseColorFactor: pool0.values[0],
  //                  metallicFactor: pool1.values[2] }
  "materials": [
    [0, 1, 2], // baseColorFactor value-index for materials 0, 1, 2
    [2, 0, 1], // metallicFactor value-index for materials 0, 1, 2
  ],
}
```

`materials` is column-major: each inner array is one array binding's column of
value-indices into a single pool. A palette may have no array bindings.
`materials` is then one empty array per material, so the material count `M`
survives as `materials.length`, and every material resolves every array-bound
attribute to its default.

A voxel's attribute values resolve from its object's `layers` as follows:

1. Each layer supplies its palette's bindings. A scalar binding supplies its
   `attribute` as `valuePools[poolRef].values[valueRef]`, one value for the
   whole object. An array binding supplies its attribute per voxel: read the
   voxel's sample `m` from the layer's channel, a material index; each array
   binding `b` supplies `arrayBindings[b].attribute` as
   `valuePools[arrayBindings[b].poolRef].values[materials[b][m]]`. An
   unsampled layer has no channel and supplies only its scalar bindings.
2. Layers override: contributions apply in `layers` order, back to front,
   and each attribute takes its value from the last layer that supplies it.
   Three layers supplying `{a, b, c}`, then `{a}`, then `{c}` resolve to `b`
   from the first, `a` from the second, and `c` from the third.
3. Unbound attributes take their default from the [Attributes](#attributes)
   table.

A scalar binding wires an attribute to a value; any arithmetic, such as
`emissiveStrength` multiplying `emissiveFactor`, comes from the attribute
vocabulary. Within one palette an attribute appears in `arrayBindings` or
`scalarBindings`, never both, so a single layer never conflicts with itself.

### Sharing Idioms

One pool cell can supply an attribute at every scope without cloning anything:

1. All materials of one palette share a value: put a scalar binding on that
   palette. One `layers` entry supplies both arities; nothing is listed
   twice.
2. Per-object variation over a shared palette: make small palettes of one
   scalar binding each, with `materials: []` so they are never sampled, and
   list one after the shared palette. Switching an object's knob is a
   one-integer edit.
3. Single source of truth: the pool cell. Editing it updates every palette
   that references it.
4. Per-voxel variation: move the attribute from `scalarBindings` to
   `arrayBindings`, giving it a real column and channel.
5. Whole-object override: list a scalar-binding palette after the layer it
   overrides; the object-wide value replaces the per-voxel values for that
   attribute.

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
    "arrayBindings": [ /* ... */ ],
    "scalarBindings": [],
    "materials": [ /* ... */ ],
  },

  // 1: the lamp-glow knob; no materials, so it is never sampled
  {
    "arrayBindings": [],
    "scalarBindings": [
      { "attribute": "emissiveStrength", "poolRef": 0, "valueRef": 1 },
    ],
    "materials": [],
  },

  // 2: the sign-glow knob: the same pool, the next cell
  {
    "arrayBindings": [],
    "scalarBindings": [
      { "attribute": "emissiveStrength", "poolRef": 0, "valueRef": 2 },
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

## Attributes

Replaces the first sentence of the intro paragraph and adds one sentence to
the emission paragraph under glTF conventions; everything else in the
subsection, including the table, is unchanged.

---

An attribute is a named material property, listed in a palette's
`arrayBindings[].attribute` and `scalarBindings[].attribute`. The format wires
attributes without defining them: the name carries the meaning and the pool
carries the values; that pairing is all the format defines.

[unchanged: the rest of the intro paragraph, the vocabulary paragraph, the
glTF conventions intro, the table, and the color paragraph]

At the end of the emission paragraph, append:

A strength shared by a whole palette or object is typically wired as a scalar
binding (see [Palettes](#palettes)).

---

## Versioning and Extensibility

Replaces item 4; the other items are unchanged.

---

4. Unknown **attribute** names in array and scalar bindings are ignored, since
   attributes are advisory and convention-based, so adding attributes is
   backward compatible.

---

## Validation

Replaces rules 5, 6, 8, and 10 and the intro of rule 11; the section intro,
all other rules, and rule 11's per-encoding sub-items are unchanged.

---

5. Unknown keys reject in every closed structure: file, `main`,
   `runtimeState`, `editState`, object, encoding block, palette, array
   binding, scalar binding, value pool, transform, hierarchy node, and edit
   object. The only open points are `main.ext` and binding attribute names.
6. All indices are in range:
   1. each object `layers` entry indexes `runtimeState.palettes`.
   2. each array and scalar binding `poolRef` indexes
      `runtimeState.valuePools`.
   3. each `childNodes` entry indexes `runtimeState.hierarchyNodes`.
   4. each `childObjects` entry indexes `runtimeState.objects`.
   5. each `rootHierarchyNodes` entry indexes `runtimeState.hierarchyNodes`.
8. **Objects**, per object:
   1. `layers` is present, an array of integers, possibly empty.
   2. `voxelPositions` and `voxelSamples` are present; the Positions and
      Samples rules check their structure.
10. **Palettes** (`runtimeState.palettes`): an array, possibly empty. Each
    palette's keys are drawn only from { `arrayBindings`, `scalarBindings`,
    `materials` }.
    1. `arrayBindings` is an array, possibly empty; each array binding has
       exactly the keys `attribute`, a non-empty string, and `poolRef`, an
       integer. `scalarBindings` is an array, possibly empty; each scalar
       binding has exactly the keys `attribute`, a non-empty string,
       `poolRef`, an integer, and `valueRef`, an integer.
    2. no two bindings share an `attribute`, across `arrayBindings` and
       `scalarBindings` together.
    3. when `arrayBindings` is non-empty, `materials` has exactly
       `arrayBindings.length` columns, one per array binding in binding order.
    4. every column is an array of the same length `M >= 0`, the material
       count. When `arrayBindings` is empty, every `materials` entry is
       instead an empty array, one per material, and `M = materials.length`.
    5. every `materials[b][m]` is an integer in
       `[0, valuePools[arrayBindings[b].poolRef].values.length)`.
    6. every scalar binding's `valueRef` is an integer in
       `[0, valuePools[poolRef].values.length)`.
11. **Samples**: let `V` be the voxel count from the position block. A layer
    is sampled iff the material count `M` of its palette is greater than
    zero. `voxelSamples.data` has exactly one channel per sampled layer, in
    `layers` order, so channel `c` belongs to the `c`-th sampled layer. For
    channel `c`, let `M` be the material count of its layer's palette, and
    by encoding:

Note no rule polices layer overlap or repeated layer references: the override
order in [Palettes](#palettes) gives both their meaning. An `M = 0` palette
needs no special case: it is never sampled, so it never has a channel.

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

        // emissive strengths, referenced by cell from a scalar binding
        { "kind": "float", "min": 0, "max": "none", "values": [1, 5] },
      ],

      "palettes": [
        // value pool 1 is bound twice, to metallicFactor and roughnessFactor
        {
          "arrayBindings": [
            { "attribute": "baseColorFactor", "poolRef": 0 },
            { "attribute": "metallicFactor", "poolRef": 1 },
            { "attribute": "roughnessFactor", "poolRef": 1 },
            { "attribute": "emissiveFactor", "poolRef": 2 },
          ],

          "scalarBindings": [],

          // column-major, one column per array binding. Material 2 resolves
          // to baseColorFactor #0000FFFF, metallicFactor 0.5, roughnessFactor
          // 0, emissiveFactor #FF6600.
          "materials": [
            [0, 1, 2],
            [2, 0, 1],
            [1, 1, 0],
            [0, 0, 1],
          ],
        },

        // base color authored in linear form instead of hex
        {
          "arrayBindings": [{ "attribute": "baseColorFactor", "poolRef": 3 }],
          "scalarBindings": [],
          "materials": [[0]],
        },

        // one scalar binding and no materials, so a layer referencing it is
        // never sampled: it supplies one emissive strength to the whole
        // object and carries no channel
        {
          "arrayBindings": [],
          "scalarBindings": [
            { "attribute": "emissiveStrength", "poolRef": 4, "valueRef": 1 },
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
          // supplies it; the other attributes come from layer 0.
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

      "hierarchyNodes": [
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

      "rootHierarchyNodes": [0],
    },
  },
}
```

---

## TypeScript Schema

Replaces the `VoxelObject` interface, the `SampleBlock` type's comments, the
`Palette` section comment and interface, and the `Binding` interface, which
splits into `ArrayBinding` and `ScalarBinding`. Everything else in the schema
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
  // palette's bindings and later layers override earlier ones. A layer is
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

// A palette binds attribute names to value pools, then stores its materials
// column-major: one inner array per array binding, in binding order, each of
// length M, the material count. A voxel samples material m; attribute
// arrayBindings[b].attribute takes
// valuePools[arrayBindings[b].poolRef].values[materials[b][m]], and each
// scalar binding takes valuePools[poolRef].values[valueRef], one value for
// the whole palette. With no array bindings, materials is instead one empty
// array per material, so M survives as materials.length. Layers apply in
// `layers` order; each attribute takes its value from the last layer that
// supplies it.
interface Palette {
  arrayBindings: ArrayBinding[];

  scalarBindings: ScalarBinding[];

  materials: number[][];
}

// One attribute-to-pool binding; fixes one column of materials.
interface ArrayBinding {
  // attribute name (see Attributes); advisory, unknown names ignored
  attribute: string;

  // index into RuntimeState.valuePools
  poolRef: number;
}

// One attribute pinned to a single pool cell, one value for the whole
// palette.
interface ScalarBinding {
  // attribute name (see Attributes); advisory, unknown names ignored
  attribute: string;

  // index into RuntimeState.valuePools
  poolRef: number;

  // index into valuePools[poolRef].values
  valueRef: number;
}
```
