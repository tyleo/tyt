# Scalar bindings: target spec text

_Part of the [voxj scalar-bindings plan](../README.md)._

The complete replacement text for every section of
[voxel-json-file-format.md](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md)
this change touches, drafted against the spec as of commit `7527d30`. This is
the phase 1 iteration surface: the owner amends this file until approved, then
phase 2 copies it into the spec in one commit. Sections not listed here do not
change. A line reading `[unchanged: ...]` splices the named text through from
the current spec verbatim.

The [README's open questions](../README.md#open-questions) are marked inline
as `**[OPEN n]**` where their answers land. Each marker is removed once its
question resolves and the wording is folded in.

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

  // [X, Y, Z] integer translation from the placing node to the grid's min corner
  "origin": [0, 0, 0],
  "voxelPositions": { "encoding": "raw-json", "data": [[0, 0, 0]] },

  // array layers: palette references, one per sampled layer
  "arrayLayers": [0],
  // scalar layers: scalar-palette references; no channels, no per-voxel data
  "scalarLayers": [],
  // one channel per arrayLayers entry; each channel is one material index per voxel
  "voxelSamples": { "encoding": "raw-json", "data": [[0]] },
}
```

An object carries two layer lists, both required arrays of palette indices.
`arrayLayers` lists the sampled layers: each entry is one layer mapping every
voxel to one material in its palette. `scalarLayers` lists
[scalar palettes](#palettes), each contributing its palette-scoped
scalar-binding values to the object as a whole; a scalar layer carries no
channel and no per-voxel data of any kind, and no palette appears twice in one
object's `scalarLayers`. Two array layers may reference the same palette; what
the overlap means is left to the consuming application, as is how the
contributions of different layers combine (see [Palettes](#palettes)).

**[OPEN 1: if back-to-front becomes the canonical `arrayLayers` order, its
wording lands here: first entry rearmost, last frontmost; normative rule or
documented convention; and whether `scalarLayers` order has any meaning.]**

`voxelPositions` and `voxelSamples` are encoded blocks (see
[Voxel Encoding](#voxel-encoding)). Each voxel has a position `(x, y, z)` and
one material sample per array layer. `voxelSamples` carries exactly one
channel per array layer, in `arrayLayers` order, and the sample in channel `c`
is a material index into the palette `arrayLayers[c]`. Scalar layers carry no
channels. The number of voxels is implicit: it is the number of positions
decoded from `voxelPositions`. Positions within an object must be unique, and
every voxel samples every array layer.

[unchanged: the `bounds` paragraph, the `origin` paragraph, and the closing
voxel-order paragraph]

---

## Sample Encodings

Subsection of Voxel Encoding. Replaces the intro paragraph, rewords the three
encodings, and retitles the example; the example body is unchanged.

---

A sample block holds one channel per array layer, in `arrayLayers` order;
scalar layers have no channels. Each channel gives, for every voxel in the
position block's voxel order, a material index into that layer's palette.

1. `raw-json`: one channel per array layer, each a plain array of that layer's
   material index for every voxel: `[[l0v0, l0v1, ...], [l1v0, l1v1, ...],
   ...]`.
2. `rle-json`: one channel per array layer; each channel is a flat run-length
   encoding `[value1, count1, value2, count2, ...]`. Counts are positive
   integers and, in every channel, sum to the number of voxels.
3. `packed-base64`: one bit-packed channel per array layer. For the channel of
   a layer whose palette has `M` materials, each voxel's material index is
   packed at fixed width `b = max(1, bitLength(M - 1))` bits, MSB-first, 8 per
   byte, with the final byte zero-padded; the width is derived from `M` and
   not stored. `data` is one base64 string per array layer, in `arrayLayers`
   order, each encoding exactly `ceil(voxelCount * b / 8)` bytes. This is the
   same packing scheme as the `bitmap-base64` position encoding, which is its
   `b = 1` special case. An empty object has one `""` per array layer. Best
   for incoherent or many-material objects, where `rle-json` would approach
   one run per voxel.

#### Example: two array layers over four voxels; layer 0 material indices `0, 0, 0, 1` (palette `M = 2`) and layer 1 material indices `2, 2, 3, 3` (palette `M = 4`), in the position block's voxel order

[unchanged: the example code block]

---

## Voxel Order

Replaces the first sentence; the numbered list and the closing paragraph are
unchanged.

---

The position block defines the object's single canonical voxel order, and
every sample channel, one per array layer, is in that same order,
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
in two arities. An **array binding** binds an attribute to a whole pool and
takes one `materials` column of value-indices, so the attribute varies per
material. A **scalar binding** pins an attribute to a single pool cell,
`valuePools[poolRef].values[valueRef]`, one value for the whole palette; it
takes no column. A voxel samples a material in each array layer by its index
in that layer's palette. A palette may be referenced by any number of layers
and objects (see [Objects](#objects)).

"Scalar" means single-valued, not numeric: a scalar binding may reference a
cell of any pool kind, a color or `json` value included.

**[OPEN 2: `arrayBindings` / `scalarBindings` / `arrayLayers` /
`scalarLayers` are working names; `columnBindings` / `valueBindings` are the
noted alternates if the "scalar" reading grates. The TypeScript interface
names follow the field names.]**

Materials are stored column-major, one column per array binding:

```jsonc
{
  // ordered array bindings; each binds an attribute name to a value pool
  // index. Order fixes the column order in `materials`.
  "arrayBindings": [
    { "attribute": "baseColorFactor", "poolRef": 0 },
    { "attribute": "metallicFactor", "poolRef": 1 },
  ],

  // scalar bindings; each pins an attribute to one pool cell for the whole
  // palette. No column, no entry in `materials`. No attribute repeats across
  // arrayBindings and scalarBindings together.
  "scalarBindings": [
    { "attribute": "emissiveStrength", "poolRef": 2, "valueRef": 1 },
  ],

  // `materials` is column-major: one inner array per array binding (a
  // column), in binding order, so materials.length == arrayBindings.length.
  // Every column has the same length, the material count M. materials[b][m]
  // is a value-index into the pool bound by column b. A voxel samples
  // material m in [0, M); resolve it by reading down the columns:
  //   material 0 = { baseColorFactor: pool0.values[0], metallicFactor: pool1.values[2] }
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
attribute to its default. Scalar bindings never contribute to or read from
`materials`.

A **scalar palette** is a palette with `arrayBindings: []` and
`materials: []`, so `M = 0`. It carries only scalar bindings and is the only
palette shape an object's `scalarLayers` may reference (see
[Objects](#objects) and [Validation](#validation)). A scalar palette with
`scalarBindings: []` is legal; the format does not police pointlessness.

A layer contributes attribute values to a voxel as follows:

1. An array layer contributes a sampled material plus its palette's scalars.
   Read the voxel's sample `m`, a material index. For each array binding `b`,
   the attribute `arrayBindings[b].attribute` takes
   `valuePools[arrayBindings[b].poolRef].values[materials[b][m]]`. Each scalar
   binding on the same palette takes `valuePools[poolRef].values[valueRef]`,
   the same value for every voxel.
2. A scalar layer contributes exactly its palette's scalar-binding values: no
   sample, no channel, no per-voxel read.
3. Unbound attributes take their default from the [Attributes](#attributes)
   table. How the contributions of multiple layers combine is left to the
   consuming application, for scalar layers exactly as for overlapping array
   layers.

A scalar binding is a fixed value, not a default and not a modifier: the
format defines wiring, never arithmetic. Where an attribute's vocabulary
defines modifier behavior, as `emissiveStrength` multiplying `emissiveFactor`,
that behavior comes from the vocabulary and needs nothing from the binding's
arity. Within one palette an attribute may appear in `arrayBindings` or
`scalarBindings`, never both.

### Sharing Idioms

One pool cell can supply an attribute at every scope without cloning anything:

1. All materials of one palette share a value: put a scalar binding on that
   palette, next to its array bindings. No extra layer needed.
2. Per-object variation over a shared palette: make small scalar palettes and
   reference them through `scalarLayers`; switching an object's knob is a
   one-integer edit.
3. Single source of truth: the pool cell. Editing it updates every palette
   that references it.
4. Per-voxel variation is the escape hatch: move the attribute from
   `scalarBindings` to `arrayBindings`, giving it a real column and channel.

Idiom 2, two lamp objects sharing one base palette but glowing at different
strengths:

```jsonc
"valuePools": [
  // 0: emissive strengths, referenced by cell
  { "kind": "float", "min": 0, "max": "none", "values": [1, 5, 40] },
],

"palettes": [
  // 0: the shared base palette; the array side elided
  { "arrayBindings": [ /* ... */ ], "scalarBindings": [], "materials": [ /* ... */ ] },

  // 1: scalar palette "lamp glow"
  {
    "arrayBindings": [],
    "scalarBindings": [{ "attribute": "emissiveStrength", "poolRef": 0, "valueRef": 1 }],
    "materials": [],
  },

  // 2: scalar palette "sign glow": the same pool, the next cell
  {
    "arrayBindings": [],
    "scalarBindings": [{ "attribute": "emissiveStrength", "poolRef": 0, "valueRef": 2 }],
    "materials": [],
  },
],

"objects": [
  // "Lamp A": the shared palette plus its own knob
  { /* ... */ "arrayLayers": [0], "scalarLayers": [1] },
  // "Neon Sign": same base palette, its own knob; switching knobs is a
  // one-integer edit in scalarLayers
  { /* ... */ "arrayLayers": [0], "scalarLayers": [2] },
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

**[OPEN 4: `version` stays `1`, the format changes in place, and this section
needs no other wording. Confirm.]**

**[OPEN 1: if the canonical layer order is normative rather than a documented
convention, check whether this section's ignore-versus-reject framing needs a
line about it.]**

---

## Validation

Replaces rules 5 through 8, 10, and the intro sentence of rule 11; the
section intro, all other rules, and rule 11's per-encoding sub-items are
unchanged.

---

5. Unknown keys reject in every closed structure: file, `main`,
   `runtimeState`, `editState`, object, encoding block, palette, array
   binding, scalar binding, value pool, transform, hierarchy node, and edit
   object. The only open points are `main.ext` and binding attribute names.
6. All indices are in range:
   1. each object `arrayLayers` and `scalarLayers` entry indexes
      `runtimeState.palettes`.
   2. each array and scalar binding `poolRef` indexes
      `runtimeState.valuePools`.
   3. each `childNodes` entry indexes `runtimeState.hierarchyNodes`.
   4. each `childObjects` entry indexes `runtimeState.objects`.
   5. each `rootHierarchyNodes` entry indexes `runtimeState.hierarchyNodes`.
7. References are unique: no hierarchy node lists the same child node or the
   same child object twice, no node appears in `rootHierarchyNodes` twice, and
   no object lists the same palette twice in `scalarLayers` (two identical
   scalar refs carry zero distinguishing information). Duplicates in
   `arrayLayers` stay legal: each entry carries its own channel.
8. **Objects**, per object:
   1. `arrayLayers` and `scalarLayers` are present, each an array of integers,
      possibly empty.
   2. every `scalarLayers` entry references a scalar palette, one with
      `arrayBindings: []` and `materials: []`; any other palette shape
      rejects, since its array machinery would be dead data for that
      reference.
   3. `voxelPositions` and `voxelSamples` are present; the Positions and
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
11. **Samples**: let `V` be the voxel count from the position block.
    `voxelSamples.data` has exactly `arrayLayers.length` channels, one per
    array layer in `arrayLayers` order. For channel `c`, let `M` be the
    material count of palette `arrayLayers[c]`, and by encoding:

**[OPEN 3: under rule 11 as reworded, an `M = 0` palette in `arrayLayers` is
satisfiable only when `V = 0`, the same vacuous legality as today. Keep that,
or add an explicit rejection of scalar palettes in `arrayLayers` to rule 8 or
11.]**

Note no rule gives `M = 0` a special case, and no rule checks cross-layer
attribute overlap: two scalar layers supplying the same attribute, or a scalar
layer overlapping an array layer's binding, is app-defined meaning, the same
posture as two `baseColorFactor` array layers.

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

        // one shared float pool, bound by both metallicFactor and roughnessFactor
        { "kind": "float", "min": 0, "max": 1, "values": [0, 0.5, 1] },

        { "kind": "srgb-hex", "values": ["#000000", "#FF6600"] },

        { "kind": "linear-rgba-float", "values": [[1, 0, 0, 1]] },

        // emissive strengths, referenced by cell from a scalar binding
        { "kind": "float", "min": 0, "max": "none", "values": [1, 5] },
      ],

      "palettes": [
        // value pool 1 is bound twice here, to metallicFactor and roughnessFactor
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

        // a scalar palette: no columns, no materials, M = 0. Referenced from
        // an object's scalarLayers, it contributes one emissive strength to
        // the whole object.
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

          // two array layers: palette 0, then palette 1. Layers do not merge;
          // the app decides what two baseColorFactor layers mean.
          "arrayLayers": [0, 1],
          "scalarLayers": [],

          // one channel per array layer, each a material index per voxel:
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
          // emissive knob: scalar palette 2 contributes emissiveStrength 5.
          // Scalar layers get no sample channel.
          "arrayLayers": [0],
          "scalarLayers": [2],
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

  // palette indices, one per array layer (see Objects). Array layers are
  // independent sampled material channels; the format defines no merge across
  // layers.
  arrayLayers: number[];

  // scalar-palette indices (see Palettes): each contributes its palette's
  // scalar-binding values to the whole object. No channels, no per-voxel
  // data, no duplicate entries.
  scalarLayers: number[];

  voxelSamples: SampleBlock;
}

type SampleBlock =
  // One channel per array layer (in `arrayLayers` order): that layer's
  // material index for every voxel, in voxel order.
  | { encoding: "raw-json"; data: number[][] }
  // One channel per array layer: a flat run stream [value1, count1, value2, count2, ...].
  | { encoding: "rle-json"; data: number[][] }
  // One channel per array layer: each voxel's material index bit-packed at
  // width b = max(1, bitLength(M - 1)) for that layer's palette material count
  // M, MSB-first, base64-encoded (same packing as the bitmap-base64 position
  // encoding).
  | { encoding: "packed-base64"; data: string[] };

// ## Palettes

// A palette binds attribute names to value pools, then stores its materials
// column-major: one inner array per array binding, in binding order, each of
// length M, the material count. A voxel samples material m; attribute
// arrayBindings[b].attribute takes
// valuePools[arrayBindings[b].poolRef].values[materials[b][m]], and each
// scalar binding takes valuePools[poolRef].values[valueRef], one value for
// the whole palette. With no array bindings, materials is instead one empty
// array per material, so M survives as materials.length. A scalar palette
// has arrayBindings [] and materials [] (M = 0) and is the only shape an
// object's scalarLayers may reference.
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
// palette; no materials column.
interface ScalarBinding {
  // attribute name (see Attributes); advisory, unknown names ignored
  attribute: string;

  // index into RuntimeState.valuePools
  poolRef: number;

  // index into valuePools[poolRef].values
  valueRef: number;
}
```
