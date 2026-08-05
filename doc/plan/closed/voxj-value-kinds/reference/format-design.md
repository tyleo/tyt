# voxj value kinds format design

The target spec text for every section of
[voxel-json-file-format.md](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md)
the value-kinds change touches, drafted from the [README](../README.md)
for the owner's review; iteration 2 lands it in the spec in one commit
once approved. Each `##` heading below names the spec section whose text
it carries, at the spec's own nesting. A blockquote under a heading
locates and scopes the replacement and is not spec text; everything else
lands in the spec verbatim, following the spec's line conventions.
Sections this page does not name keep their text, subject to iteration
2's sweep for stray wording.

## Value Pools

> Replaces the section's opening paragraph and its example block. The
> Value Pool Kinds subsection follows below.

Value pools live in `main.runtimeState.valuePools`, a shared array referenced by index, siblings of `objects` and `palettes`. A value pool is its `kind` and its `values`: `kind` names one JSON shape, and every entry of `values` is one value of that shape. Palettes reference value pools by index (see [Palettes](#palettes)).

```jsonc
// a value pool: shared values of one shape
{
  // value-shape tag
  "kind": "vec-4-float",

  // plain JSON literals, each well-formed for `kind`, indexed by value-index
  "values": [
    [1, 0, 0, 1],
    [0, 1, 0, 1],
    [0, 0, 1, 1],
  ],
}

// scalar value pools
{ "kind": "float", "values": [0, 0.5, 1] }
{ "kind": "int", "values": [0, 128, 255] }

// the same vector kind holds colors under one property name and plain
// numbers under another (see Properties)
{ "kind": "vec-2-int", "values": [[3, 7]] }
{ "kind": "vec-3-float", "values": [[0, 0, 1], [1, 0, 0]] }
```

## Value Pool Kinds

> Replaces the whole subsection: the intro, the kind table, and the
> notes. The domain examples at the end are new.

`kind` is a closed vocabulary naming the shape of a value pool's `values`. Every kind's `values` are plain readable JSON literals, and the declared kind types the whole array: a consumer reads an `int` value pool as integers and a `vec-3-float` value pool as three-float vectors; `json` is the kind whose values stay arbitrary JSON. A consumer must reject a file whose `kind` it does not recognize (see [Versioning and Extensibility](#versioning-and-extensibility)).

| `kind`        | JSON form                | Example `values`         | Typical properties                                                          |
| ------------- | ------------------------ | ------------------------ | --------------------------------------------------------------------------- |
| `bool`        | boolean                  | `[true, false]`          | flags                                                                       |
| `float`       | number                   | `[0, 0.5, "inf"]`        | emissiveStrength, ior, metallic, occlusionStrength, roughness, transmission |
| `int`         | number                   | `[0, 1, 2, 7]`           | counts, ids, indices                                                        |
| `json`        | any JSON, including null | `[{"k": 1}, "x", 3]`     | any custom property                                                         |
| `string`      | string                   | `["low", "high"]`        | enumerated tags                                                             |
| `vec-2-float` | number[2]                | `[[0.25, 0.75]]`         | custom float pairs                                                          |
| `vec-2-int`   | number[2]                | `[[3, 7]]`               | 2D grid coordinates                                                         |
| `vec-3-float` | number[3]                | `[[1, 0, 0], [0, 0, 1]]` | emissiveColor, normals                                                      |
| `vec-3-int`   | number[3]                | `[[3, 7, 1]]`            | 3D grid coordinates                                                         |
| `vec-4-float` | number[4]                | `[[1, 0, 0, 1]]`         | baseColor                                                                   |
| `vec-4-int`   | number[4]                | `[[3, 7, 1, 0]]`         | custom integer 4-tuples                                                     |

Notes:

1. A kind is one JSON shape, and the shape is the whole contract: a value pool is well-formed when every entry of `values` has its kind's shape. What a value means, a color, a normal, a count, is the binding property's concern (see [Properties](#properties)).
2. A float value is a finite JSON number, the string `"inf"`, or the string `"-inf"`. JSON has no infinity literal, so the sentinel strings spell the two infinities. `NaN` has no spelling and writers error on it.
3. An int value is a JSON number spelled as an integer, so `3.0` and `3e0` reject, with magnitude at most `2^53 - 1`, so a consumer reading numbers as doubles cannot silently lose one. `"inf"` and `"-inf"` reject as int values: an infinite integer means nothing.
4. A vector kind's value is an array of exactly the kind's length: float values for the `vec-*-float` kinds, int values for the `vec-*-int` kinds. A scalar is not a one-element vector: `0.5` and `[0.5]` are different JSON, so `int` and `float` stand apart from the vector kinds.
5. No kind carries a range. The format checks every value's shape and never its range; a range is a fact about the binding property and rides the property vocabulary (see [Properties](#properties)).
6. `kind` is required and has no default. A value pool has no optional fields.

```jsonc
{ "kind": "float", "values": [0, 0.5, "inf"] } // fine
{ "kind": "int", "values": [3.0] }             // rejects: 3.0 is not 3
{ "kind": "int", "values": ["inf"] }           // rejects: no infinite int
```

## Palettes

> Replaces the palette example block. The section's prose, including the
> resolution steps after the example, keeps its text.

```jsonc
{
  // ordered properties; each binds a property name to a value pool index.
  // Order fixes the value-index order in each `materials` row. No duplicate
  // name.
  "properties": [
    { "name": "baseColor", "valuePool": 0 },
    { "name": "metallic", "valuePool": 1 },
  ],

  // one row per material, a value-index per property, in property order.
  // `materials[m][b]` is a value-index into the value pool bound by
  // `properties[b]`. A voxel samples material `m` in `[0, M)`; resolve it
  // by reading across its row:
  //
  // material 0 = {
  //   baseColor: valuePools[0].values[0],
  //   metallic: valuePools[1].values[2]
  // }
  "materials": [
    [0, 2], // material 0
    [1, 0], // material 1
    [2, 1], // material 2
  ],
}
```

## Sharing Idioms

> Replaces the idiom 2 example block. The numbered idioms and the
> sentence introducing the example keep their text.

```jsonc
"valuePools": [
  // 0: lamp colors
  {
    "kind": "vec-4-float",
    "values": [
      [1, 0.7, 0.25, 1],
      [1, 0.04, 0, 1],
    ],
  },
  // 1: emissive strengths
  { "kind": "float", "values": [5, 40] },
],

"palettes": [
  // 0: the lamp variant; every row repeats strength cell 0
  {
    "properties": [
      { "name": "baseColor", "valuePool": 0 },
      { "name": "emissiveStrength", "valuePool": 1 },
    ],
    "materials": [[0, 0], [1, 0]],
  },

  // 1: the sign variant: the same value pools, the next strength cell
  {
    "properties": [
      { "name": "baseColor", "valuePool": 0 },
      { "name": "emissiveStrength", "valuePool": 1 },
    ],
    "materials": [[0, 1], [1, 1]],
  },
],

"objects": [
  // "Lamp A" glows at strength 5
  { /* ... */ "layers": [0] },

  // "Neon Sign": the same materials at strength 40
  { /* ... */ "layers": [1] },
]
```

## Properties

> Replaces the section's opening paragraph. The second paragraph, naming
> glTF as the recommended vocabulary, keeps its text.

A property is a named material parameter, listed in a palette's `properties[].name`. The format pairs each name with a value pool and leaves its meaning and value range as convention between producer and consumer; the tools that understand the vocabulary may check a range like `metallic`'s `[0, 1]`. A consumer ignores any property name it does not recognize, so any tool may bind names of its own (see [Versioning and Extensibility](#versioning-and-extensibility)).

## glTF conventions

> Replaces the whole subsection: the intro paragraph, the table, and the
> two paragraphs after it, of which the emissive-composition one folds
> into the table's strength row and does not return.

The recommended property vocabulary is glTF's metallic-roughness model. A material that follows it maps onto a glTF material, and the defaults below are glTF's own. Each property binds a value pool of the kind listed, and an unbound property renders at its default. The Range column restates the glTF schema's range for each property and binds as vocabulary convention (see [Properties](#properties)).

| Property            | Kind          | Range             | Default        | Meaning                                                                |
| ------------------- | ------------- | ----------------- | -------------- | ---------------------------------------------------------------------- |
| `baseColor`         | `vec-4-float` | each in `[0, 1]`  | `[1, 1, 1, 1]` | Base color, straight alpha = opacity (glTF `baseColorFactor`)          |
| `emissiveColor`     | `vec-3-float` | each in `[0, 1]`  | `[0, 0, 0]`    | Emissive color, black = none (glTF `emissiveFactor`)                   |
| `emissiveStrength`  | `float`       | `[0, inf)`        | `1`            | Multiplies emissive color (glTF `KHR_materials_emissive_strength`)     |
| `ior`               | `float`       | `0` or `[1, inf)` | `1.5`          | Index of refraction (glTF `KHR_materials_ior`)                         |
| `metallic`          | `float`       | `[0, 1]`          | `1`            | Metalness (glTF `metallicFactor`)                                      |
| `occlusionStrength` | `float`       | `[0, 1]`          | `1`            | Flat ambient occlusion, 1 = none (glTF `occlusionTexture.strength`)    |
| `roughness`         | `float`       | `[0, 1]`          | `1`            | Roughness (glTF `roughnessFactor`)                                     |
| `transmission`      | `float`       | `[0, 1]`          | `0`            | Light transmission through surface (glTF `KHR_materials_transmission`) |

The two color properties hold linear light with sRGB primaries and the D65 white point: `baseColor` is 4 components with straight alpha, `emissiveColor` is 3 with none. The binding name is what makes a value a color, so the color space is vocabulary convention like the ranges. A producer authoring from sRGB applies the sRGB transfer to the color components before writing.

## Versioning and Extensibility

> Replaces items 3 and 4. Items 1, 2, and 5 keep their text.

3. An unrecognized value pool `kind` must be rejected: its values cannot be safely decoded, exactly as an unknown `encoding`'s data cannot, and it must never be reinterpreted or downgraded. `kind` is required and has no default.
4. Unknown property **names** in `properties` are ignored: properties are advisory and convention-based, so one tool's names pass through another's reader untouched.

## Validation

> Replaces rules 3, 4, and 9. Rule 3 gains the sentinel exception, rule 4
> spells the full entity name, and rule 9 reduces to shape checks. Rule
> 10 and every other rule keep their text and their numbers.

3. Types are exact and nothing is coerced. A string where a number is expected, or the reverse, rejects. A number where an integer is expected is spelled as an integer. Every number is finite, so `NaN` and `+/-Infinity` reject; the sentinel strings `"inf"` and `"-inf"` are a float value's own spelling for the infinities (see [Value Pool Kinds](#value-pool-kinds)).
4. `null` rejects everywhere except in a `json` value pool's `values` and inside `main.ext`.

> Rules 5 through 8 keep their text.

9. **Value pools** (`runtimeState.valuePools`): an array, possibly empty. Each value pool has exactly the keys `kind` and `values`.
   1. `kind` is present and recognized.
   2. `values` is present, a non-empty array, with no `null` entry unless `kind` is `json`. Every entry is the shape `kind` declares (see [Value Pool Kinds](#value-pool-kinds)):
      1. `bool`: a JSON boolean.
      2. `float`: a float value.
      3. `int`: an int value.
      4. `json`: any JSON value, including `null`.
      5. `string`: a JSON string.
      6. `vec-2-float`, `vec-3-float`, `vec-4-float`: an array of exactly 2, 3, or 4 float values.
      7. `vec-2-int`, `vec-3-int`, `vec-4-int`: an array of exactly 2, 3, or 4 int values.

## File Example

> Replaces the whole example block. `nodes` and `rootNodes` are
> unchanged inside it; everything above them changes.

```jsonc
{
  "version": 1,
  "main": {
    "runtimeState": {
      "valuePools": [
        // 0: base colors
        {
          "kind": "vec-4-float",
          "values": [
            [1, 0, 0, 1],
            [0, 1, 0, 1],
            [0, 0, 1, 1],
          ],
        },

        // 1: one shared float value pool, bound by `metallic` and
        // `roughness`
        { "kind": "float", "values": [0, 0.5, 1] },

        // 2: emissive colors
        {
          "kind": "vec-3-float",
          "values": [
            [0, 0, 0],
            [1, 0.25, 0],
          ],
        },

        // 3: the base color layer 1 overrides with
        { "kind": "vec-4-float", "values": [[1, 1, 0, 1]] },

        // 4: emissive strengths, one cell per variant palette
        { "kind": "float", "values": [1, 5] },
      ],

      "palettes": [
        // value pool 1 is bound twice, to `metallic` and `roughness`
        {
          "properties": [
            { "name": "baseColor", "valuePool": 0 },
            { "name": "metallic", "valuePool": 1 },
            { "name": "roughness", "valuePool": 1 },
            { "name": "emissiveColor", "valuePool": 2 },
            { "name": "emissiveStrength", "valuePool": 4 },
          ],

          // one row per material, a value-index per property. Material 2
          // resolves to `baseColor` [0, 0, 1, 1], `metallic` 0.5,
          // `roughness` 0, `emissiveColor` [1, 0.25, 0],
          // `emissiveStrength` 1.
          "materials": [
            [0, 2, 1, 0, 0],
            [1, 0, 1, 0, 0],
            [2, 1, 0, 1, 0],
          ],
        },

        // a one-property palette; layered over palette 0 it overrides
        // `baseColor` only
        {
          "properties": [{ "name": "baseColor", "valuePool": 3 }],
          "materials": [[0]],
        },

        // the bright variant of palette 0: the same value pools and rows,
        // differing only in the `emissiveStrength` column
        {
          "properties": [
            { "name": "baseColor", "valuePool": 0 },
            { "name": "metallic", "valuePool": 1 },
            { "name": "roughness", "valuePool": 1 },
            { "name": "emissiveColor", "valuePool": 2 },
            { "name": "emissiveStrength", "valuePool": 4 },
          ],

          "materials": [
            [0, 2, 1, 0, 1],
            [1, 0, 1, 0, 1],
            [2, 1, 0, 1, 1],
          ],
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

          // two layers, back to front: palette 0, then palette 1. Both bind
          // `baseColor`, so the later layer supplies it; the other
          // properties come from layer 0.
          "layers": [0, 1],

          // one channel per layer, each a material index per voxel:
          //
          // layer 0 -> materials 0, 2 of palette 0
          // layer 1 -> materials 0, 0 of palette 1
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
          // the bright variant of Object A's base palette
          "layers": [2],
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

## TypeScript Schema

> Replaces the value pool comment, the `ValuePool` type, and the kind
> union. `FloatValue` and `IntValue` are new, and the kind union spells
> the full entity name, `ValuePoolKind`. The rest of the schema keeps its
> text.

```ts
// A shared value pool: `values` all of the one JSON shape `kind` names.

// A float value: a finite number, "inf", or "-inf"; NaN has no spelling.
type FloatValue = number | "inf" | "-inf";

// An int value: a number spelled as an integer, magnitude at most
// 2^53 - 1.
type IntValue = number;

type ValuePool =
  | { kind: "bool"; values: boolean[] }
  | { kind: "float"; values: FloatValue[] }
  | { kind: "int"; values: IntValue[] }
  | { kind: "json"; values: JsonValue[] }
  | { kind: "string"; values: string[] }
  | { kind: "vec-2-float"; values: [FloatValue, FloatValue][] }
  | { kind: "vec-2-int"; values: [IntValue, IntValue][] }
  | { kind: "vec-3-float"; values: [FloatValue, FloatValue, FloatValue][] }
  | { kind: "vec-3-int"; values: [IntValue, IntValue, IntValue][] }
  | {
      kind: "vec-4-float";
      values: [FloatValue, FloatValue, FloatValue, FloatValue][];
    }
  | { kind: "vec-4-int"; values: [IntValue, IntValue, IntValue, IntValue][] };

// Closed value-shape vocabulary (see Value Pool Kinds).
type ValuePoolKind =
  | "bool"
  | "float"
  | "int"
  | "json"
  | "string"
  | "vec-2-float"
  | "vec-2-int"
  | "vec-3-float"
  | "vec-3-int"
  | "vec-4-float"
  | "vec-4-int";
```
