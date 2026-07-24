# Voxel Json File Format

A JSON format for voxel models: geometry, materials, and scene hierarchy. Examples below are JSONC for readability; real files are plain JSON.

## Design Objectives

1. **Human-understandable:** the format is plain JSON that a person can open, read, and reason about directly.
2. **Small file size:** models stay compact on disk, especially under the whole-file gzip/deflate the encodings assume downstream.
3. **Fast to parse:** files decode quickly enough to fit into game asset pipelines that ingest extracted 3D models (e.g. `.glb`) for mainstream engines.

## File Extensions

A voxel json file is a single JSON document. It is stored in one of two interchangeable forms with identical content, differing only in whether the document is packaged in a zip archive.

| Form         | Extension | Contents                                                  | MIME type                   |
| ------------ | --------- | --------------------------------------------------------- | --------------------------- |
| Uncompressed | `.voxj`   | The JSON document as UTF-8 text, read and edited directly | `application/vnd.voxj+json` |
| Compressed   | `.voxjz`  | A zip archive containing the document                     | `application/vnd.voxj+zip`  |

1. `.voxj` is the canonical authoring form. It is plain UTF-8 JSON whose top-level value is the object in [Structure](#structure), openable in any JSON-aware editor. Tools that key off the `.json` suffix may also accept `.voxj.json`.
2. `.voxjz` is the canonical shipping form. It is a standard zip archive holding exactly one `.voxj` member, conventionally named `main.voxj`, compressed with the deflate method. That deflate is the whole-file compression the encodings in [Voxel Encoding](#voxel-encoding) assume. The archive may carry additional resource members; consumers ignore members they do not recognize. Reading a `.voxjz` means opening the archive and parsing its `.voxj` member, which is byte-identical to a standalone `.voxj`. Tools that key off the `.zip` suffix may also accept `.voxjz.zip`.
3. Packaging is a transport concern and never changes the document. Recognize the form by leading bytes, not extension. An uncompressed file's first non-whitespace byte is `{`. A zip archive begins with `PK`, the bytes `0x50 0x4B`. Consumers accept either form wherever a voxel json file is expected.

## Structure

```jsonc
{
  "version": 1,

  "main": {
    "runtimeState": {
      "valuePools": [
        /* ... */
      ],

      "palettes": [
        /* ... */
      ],

      "objects": [
        /* ... */
      ],

      "nodes": [
        /* ... */
      ],

      "rootNodes": [
        /* ... */
      ],
    },

    "editState": {
      "objects": [
        /* ... */
      ],
    },

    "ext": {
      /* ... */
    },
  },
}
```

The runtime scene lives under `main.runtimeState`: value pools, palettes, objects, and hierarchy nodes are referenced by their array indices, and `rootNodes` lists indices into `nodes`. `editState` (optional) and `ext` are siblings of `runtimeState`. `editState` carries per-object editor margin aligned to the runtime objects (see [Edit State](#edit-state)); `ext` is an optional namespace for user-defined data that the core format ignores (see [Extensions](#extensions)).

## Coordinate System

The coordinate system is Z-up, right-handed. Voxel coordinates are unsigned integers and one unit = one voxel. A voxel at integer coordinate `(x, y, z)` occupies the unit cube whose minimum corner is that coordinate, spanning `[x, x + 1)` on each axis. An object carries no transform of its own beyond its grid `origin`; the hierarchy node that references it supplies rotation, scale, and placement. Seating the grid with `origin` near `-bounds / 2` makes the node's position the object's pivot, so rotating or scaling the node turns the object about its center rather than its min corner.

## Objects

An object is one voxel volume of pure geometry. Aside from its grid `origin`, it carries no transform of its own: rotation, scale, and placement come from the hierarchy node that references it.

```jsonc
{
  "name": "Object A",

  // `[X, Y, Z]` size in voxels
  "bounds": [1, 1, 1],

  // `[X, Y, Z]` integer translation from the placing node to the grid's
  // min corner
  "origin": [0, 0, 0],
  "voxelPositions": { "encoding": "raw-json", "data": [[0, 0, 0]] },

  // `layers`: palette references, ordered back to front
  "layers": [0],
  // one channel per sampled layer; each is one material index per voxel
  "voxelSamples": { "encoding": "raw-json", "data": [[0]] },
}
```

An object carries one layer list, `layers`, an array of palette indices ordered back to front. Each layer supplies all of its palette's properties: scalar properties one value for the whole object, array properties one value per voxel through a sample channel. A palette may appear in `layers` any number of times. Layers combine by overriding: contributions apply in `layers` order and each property takes its value from the last layer that supplies it, so later layers override earlier ones (see [Palettes](#palettes) for the full resolution).

A layer is **sampled** when its palette has at least one material, `M > 0` (see [Palettes](#palettes)). A palette with no materials, `materials: []`, is never sampled, so a scalar-only palette carries no per-voxel data. `voxelSamples` carries exactly one channel per sampled layer, in `layers` order: channel `c` belongs to the `c`-th sampled layer, and its samples are material indices into that layer's palette.

`voxelPositions` and `voxelSamples` are encoded blocks (see [Voxel Encoding](#voxel-encoding)). Each voxel has a position `(x, y, z)` and one material sample per sampled layer. The number of voxels is implicit: it is the number of positions decoded from `voxelPositions`. Positions within an object must be unique, and every voxel samples every sampled layer.

`bounds` is `[X, Y, Z]`, the runtime grid's size in voxels along each axis. Voxel positions are 0-based, so every voxel lies in `[0, X) x [0, Y) x [0, Z)`. `bounds` is exactly tight: the grid fits the voxels with no empty margin on any face, so on every axis some voxel reaches `0` and some voxel reaches the bound minus one. An empty object has `bounds = [0, 0, 0]`: a point at its `origin`. Build-volume margin around the geometry is not allowed here; it lives in [`editState`](#edit-state), whose edit grid may be larger than the runtime grid. `bounds` is needed to decode `bitmap-base64` and `hilbert-delta-varint-base64`, where it sets the canonical voxel order and the Hilbert `bits = max(1, bitLength(max(X, Y, Z) - 1))` (see [Voxel Order](#voxel-order)).

`origin` is `[X, Y, Z]`, three integers: the translation in voxels from the placing hierarchy node to the grid's min corner; `[0, 0, 0]` puts the min corner at the node's local origin. It shifts where the grid sits relative to its node but does not change the voxel encodings, which stay 0-based within `bounds`. Setting it to about `-bounds / 2` centers the grid on the node, making the node's position the object's pivot so rotation and scale turn the object about its center.

The position block fixes a single voxel order for the object, and every sample channel follows it voxel-for-voxel.

## Voxel Encoding

Each block is `{ "encoding", "data" }`. New `encoding` values may be added in future versions (see [Versioning](#versioning-and-extensibility)). Every binary encoding below has a reference implementation in [Reference Code](#reference-code).

All base64 in this format uses the standard RFC 4648 alphabet, not base64url, with `=` padding and no line breaks.

### Position Encodings

1. `raw-json`: one `[x, y, z]` triple per voxel, in listing order: `[[x0, y0, z0], [x1, y1, z1], ...]`. An empty object has `data = []`.
2. `bitmap-base64`: a dense occupancy bitmap. `data` is a standard base64 string with no line breaks. It encodes one occupancy bit per cell of the object's `bounds = [X, Y, Z]`, so positions are implicit and this encoding requires `bounds` to decode. The cell index is `k = x * Y * Z + y * Z + z`, iterating x outermost and z innermost over `0 <= x < X`, `0 <= y < Y`, `0 <= z < Z` with `X * Y * Z` cells total. Bit `k` is `1` if cell `k` is occupied. Bits are packed 8 per byte, MSB-first: cell `k` is bit `(7 - (k mod 8))` of byte `floor(k / 8)`. The last byte is zero-padded when `X * Y * Z` is not a multiple of 8; pad bits must be `0`. The base64 encodes exactly `ceil(X * Y * Z / 8)` bytes. The number of voxels is the number of set bits. An empty object has `bounds = [0, 0, 0]`, so its `data` is the empty string. Best for dense objects, roughly >= 50% filled; valid at any density.
3. `hilbert-delta-varint-base64`: a Hilbert-index delta list. `data` is a standard base64 string with no line breaks, encoding the deltas as an unsigned LEB128 varint stream (see the reference code in [Reference Code](#reference-code)). Each position `(x, y, z)` maps to one Hilbert index via the standard 3D Hilbert curve with `bits = max(1, bitLength(max(X, Y, Z) - 1))` taken from `bounds`. Axes map to Hilbert dimensions `(x, y, z) = (0, 1, 2)`, and the curve covers a `2 ^ bits` cube containing the bounds. Voxels are sorted by ascending index, and the encoded deltas are `[h0, h1 - h0, h2 - h1, ...]`; decode by base64-decoding to the varint stream, reading the deltas, prefix-summing to recover the indices, then Hilbert-decoding each. Every delta after the first is strictly positive. An empty object has `data = ""`. A good general-purpose encoding; strongest from sparse up through moderate density, and compact at any density because each delta is a small varint rather than a full index.

   Because the reference algorithm assembles and decodes a Hilbert index in a JS `number` (a double, exact only to `2 ^ 53`), this encoding requires `bits <= 17`, equivalently every `bounds` dimension `<= 131072`; a validator must reject larger grids, which must instead use `bitmap-base64` or `raw-json`.

#### Example: a 2 x 2 x 1 square in the `z = 0` plane - voxels `(0, 0, 0)`, `(1, 0, 0)`, `(0, 1, 0)`, `(1, 1, 0)` with `bounds = [2, 2, 1]`

```jsonc
// `raw-json` (listing order):
{ "encoding": "raw-json", "data": [[0, 0, 0], [1, 0, 0], [0, 1, 0], [1, 1, 0]] }

// `bitmap-base64`: cells in `k`-order (0, 0, 0), (0, 1, 0), (1, 0, 0),
// (1, 1, 0) are all occupied -> bits 1111 + 4 zero pad -> byte 0xF0.
{ "encoding": "bitmap-base64", "data": "8A==" }

// `hilbert-delta-varint-base64`:
//
// `bits = 1`
//
// sorted Hilbert indices [0, 3, 4, 7] ->
// deltas [0, 3, 1, 3] ->
// varint bytes 00 03 01 03 ->
// base64 "AAMBAw==".
// Those indices decode (in order) to
// (0, 0, 0), (0, 1, 0), (1, 1, 0), (1, 0, 0). This is a different voxel order
// than the bitmap's raster order, so the two encodings need the sample channels
// in different orders.
{ "encoding": "hilbert-delta-varint-base64", "data": "AAMBAw==" }
```

### Sample Encodings

A sample block holds one channel per sampled layer, in `layers` order. Each channel gives, for every voxel in the position block's voxel order, a material index into that layer's palette.

1. `raw-json`: one channel per sampled layer, each a plain array of that layer's material index for every voxel: `[[l0v0, l0v1, ...], [l1v0, l1v1, ...], ...]`.
2. `rle-json`: one channel per sampled layer; each channel is a flat run-length encoding `[value1, count1, value2, count2, ...]`. Counts are positive integers and, in every channel, sum to the number of voxels.
3. `packed-base64`: one bit-packed channel per sampled layer. For the channel of a layer whose palette has `M` materials, each voxel's material index is packed at fixed width `b = max(1, bitLength(M - 1))` bits, MSB-first, 8 per byte, with the final byte zero-padded; the width is derived from `M` and not stored. `data` is one base64 string per sampled layer, in `layers` order, each encoding exactly `ceil(voxelCount * b / 8)` bytes. This is the same packing scheme as the `bitmap-base64` position encoding, which is its `b = 1` special case. An empty object has one `""` per sampled layer. Best for incoherent or many-material objects, where `rle-json` would approach one run per voxel.

#### Example: two sampled layers over four voxels; layer 0 material indices `0, 0, 0, 1` (palette `M = 2`) and layer 1 material indices `2, 2, 3, 3` (palette `M = 4`), in the position block's voxel order

```jsonc
// `raw-json`: one array per layer, a material index per voxel.
{ "encoding": "raw-json", "data": [[0, 0, 0, 1], [2, 2, 3, 3]] }

// `rle-json`: one flat `[value, count, ...]` run stream per layer.
{ "encoding": "rle-json", "data": [[0, 3, 1, 1], [2, 2, 3, 2]] }

// `packed-base64`: one packed channel per layer.
//
// layer 0: M = 2 -> b = 1
// 0,0,0,1 -> byte 0b0001_0000 = 0x10 -> "EA=="
//
// layer 1: M = 4 -> b = 2
// 2,2,3,3 -> bits 10 10 11 11 -> byte 0b1010_1111 = 0xAF -> "rw=="
{ "encoding": "packed-base64", "data": ["EA==", "rw=="] }
```

### Voxel Order

The position block defines the object's single canonical voxel order, and every sample channel, one per sampled layer, is in that same order, voxel-for-voxel, for every combination of position and sample encoding:

1. `raw-json` positions: listing order.
2. `bitmap-base64` positions: ascending cell index `k` (raster order, z fastest).
3. `hilbert-delta-varint-base64` positions: ascending Hilbert index.

The same geometry generally orders differently under different position encodings, so re-encoding the position block changes the order and the sample channels must be regenerated to match.

### Choosing an Encoding

Position:

1. `bitmap-base64`: dense objects. Smallest geometry when filled, and the fastest to decode.
2. `hilbert-delta-varint-base64`: sparse objects, and any object with spatially coherent color that you want as small as possible. Hilbert order places neighboring voxels next to each other in the stream, which also lengthens the sample channels' runs and improves their compression. It costs more to decode.
3. `raw-json`: hand-authored or tiny objects, where readability matters more than size.

Sample:

1. `rle-json`: coherent, few-color objects with large regions of one material. The common case, and human-readable.
2. `packed-base64`: incoherent or many-color objects like noise, or color that changes almost every voxel, where `rle-json` would approach one run per voxel and balloon.
3. `raw-json`: hand-authored or tiny objects.

Favored pairs:

1. `bitmap-base64` + `rle-json`: coherent color, dense or speed-sensitive. The fast default.
2. `hilbert-delta-varint-base64` + `rle-json`: coherent color at the smallest size, for larger or sparser models; slower to decode.
3. `bitmap-base64` + `packed-base64`: incoherent or many-color.

Avoid pairing Hilbert positions with `packed-base64`: Hilbert order only helps by lengthening runs, which `packed-base64` does not use, so it costs decode time for no gain.

Positions and samples interact, so choose them as a pair. Encoding is offline, so you need not trust these rules: build the candidate pairs, compress each the way the file ships, and keep the smallest. All blocks assume whole-file gzip or deflate downstream.

## Value Pools

Value pools live in `main.runtimeState.valuePools`, a shared array referenced by index, siblings of `objects` and `palettes`. A value pool holds `values`, all of one value-shape given by `kind`. `kind` tags the shape of the values. Palettes reference pools by index: an array property references a whole pool and a scalar property a single cell of one (see [Palettes](#palettes)).

```jsonc
// a value pool: shared values of one shape
{
  // value-shape tag (see Value Pool Kinds)
  "kind": "srgba-hex",

  // plain JSON literals, each well-formed for `kind`, indexed by value-index
  "values": ["#FF0000FF", "#00FF00FF", "#0000FFFF"],
}

// only `int` and `float` pools carry `min`/`max`; each a number or "none"
{ "kind": "float", "min": 0, "max": 1, "values": [0, 0.5, 1] }
{ "kind": "float", "min": 1, "max": "none", "values": [1.5, 1.33] } // >= 1
{ "kind": "int", "min": 0, "max": 255, "values": [0, 128, 255] }

// color pools carry no `min`/`max`; the color space fixes the range
{ "kind": "srgba-float", "values": [[1, 0, 0, 1]] } // sRGB, each in [0, 1]
{ "kind": "linear-rgb-float", "values": [[2, 0, 0]] } // linear, each >= 0 (HDR)
```

### Value Pool Kinds

`kind` is a closed vocabulary tagging the shape of a pool's `values`. Every kind's `values` are plain readable JSON literals; declaring a kind enables validation: a consumer must understand a file's kinds to validate it, and must reject a file whose `kind` it does not recognize (see [Versioning and Extensibility](#versioning-and-extensibility)).

| `kind`              | JSON form            | Constraint                                | Example `values`             | Typical properties                                                                            |
| ------------------- | -------------------- | ----------------------------------------- | ---------------------------- | --------------------------------------------------------------------------------------------- |
| `json`              | any JSON, incl. null | none                                      | `[{"k": 1}, "x", 3]`         | any custom property                                                                           |
| `bool`              | boolean              | `true` / `false`                          | `[true, false]`              | flags                                                                                         |
| `float`             | number               | floating-point-valued; within `min`/`max` | `[0, 0.5, 1]`                | metallicFactor, roughnessFactor, occlusionStrength, transmissionFactor, emissiveStrength, ior |
| `int`               | number               | integer-valued, within `min`/`max`        | `[0, 1, 2, 7]`               | ids, counts, indices                                                                          |
| `string`            | string               | must be a string                          | `["low", "high"]`            | enumerated tags                                                                               |
| `srgb-float`        | number[3]            | 3 finite numbers in `[0, 1]`, sRGB        | `[[1, 0, 0], [0.5, 0.5, 0]]` | emissiveFactor, sRGB float                                                                    |
| `srgb-hex`          | string               | matches `^#[0-9A-F]{6}$`                  | `["#FF0000", "#204080"]`     | emissiveFactor; opaque custom colors                                                          |
| `srgba-float`       | number[4]            | 4 finite numbers in `[0, 1]`, sRGB        | `[[1, 0, 0, 1]]`             | baseColorFactor, sRGB float                                                                   |
| `srgba-hex`         | string               | matches `^#[0-9A-F]{8}$`                  | `["#FF0000FF"]`              | baseColorFactor, the default color kind                                                       |
| `linear-rgb-float`  | number[3]            | 3 finite numbers `>= 0`, linear           | `[[1, 0, 0], [0, 0.5, 1]]`   | emissiveFactor, linear                                                                        |
| `linear-rgba-float` | number[4]            | 4 finite numbers `>= 0`, linear           | `[[1, 0, 0, 1]]`             | baseColorFactor, linear                                                                       |

Notes:

1. `float` is a continuous, finite number; `int` is its integer-valued sibling. `min` and `max` bounds apply only to `int` and `float`; on those two both are required, and no other kind may carry them.
2. `min` and `max` each take a finite number or the string `"none"` for unbounded on that side, and both are always written out. A numeric bound must be integer-valued on `int`; when both bounds are finite numbers, `min <= max`.
3. Colors come in hex and float forms across two color spaces. `srgb-hex` / `srgba-hex` are `#RRGGBB` / `#RRGGBBAA` sRGB strings, the human-editable default. `srgb-float` / `srgba-float` are sRGB float components in `[0, 1]`; `linear-rgb-float` / `linear-rgba-float` are the linear components `>= 0`, so linear carries HDR. Colors carry no `min`/`max`; each color kind's range is fixed by its color space. Hex is sRGB only; a linear color is always written in float form.
4. `kind` is required and has no default; the bounded kinds require both `min` and `max`. A value pool has no optional fields.

## Palettes

A palette binds property names to shared [value pools](#value-pools), then lists the distinct materials it uses as rows over those pools. Properties come in two arities, and a palette may carry either, both, or neither. An **array property** binds to a whole pool and takes one value-index per material, so its value varies per material. A **scalar property** is pinned to a single pool cell of any kind, `valuePools[valuePool].values[valueIndex]`, one value for the whole palette. Scalar properties may stand alone, but an array property's values live in the material rows, so a palette with array properties has at least one material. A voxel samples a material in each sampled layer by its index in that layer's palette. A palette may be referenced by any number of layers and objects (see [Objects](#objects)).

A scalar property wires a name to a value; any arithmetic, such as `emissiveStrength` multiplying `emissiveFactor`, comes from the property vocabulary. Within one palette a name appears in `arrayProperties` or `scalarProperties`, so a single layer never conflicts with itself.

A material is one row of value-indices, one per array property, so the material count `M` is `materials.length`; with no array properties every row is empty:

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
  // order. `materials[m][b]` is a value-index into the pool bound by
  // `arrayProperties[b]`. A voxel samples material `m` in `[0, M)`; resolve
  // it by reading across its row:
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

1. Each layer supplies its palette's properties. A scalar property supplies its `name` as `valuePools[valuePool].values[valueIndex]`, one value for the whole object. An array property supplies its `name` per voxel: read the voxel's sample `m` from the layer's channel, a material index; array property `b` supplies `arrayProperties[b].name` as `valuePools[arrayProperties[b].valuePool].values[materials[m][b]]`.
2. Layers override: contributions apply in `layers` order, back to front, and each property takes its value from the last layer that supplies it. Three layers supplying `{a, b, c}`, then `{a}`, then `{c}` resolve to `b` from the first, `a` from the second, and `c` from the third.
3. Unbound properties are left to the vocabulary; the recommended glTF conventions supply a default for each (see [Properties](#properties)).

### Sharing Idioms

One pool cell can supply a property at every scope without cloning anything:

1. All materials of one palette share a value: put a scalar property on that palette. One `layers` entry supplies both arities; nothing is listed twice.
2. Per-object variation over a shared palette: make small palettes of one scalar property each, with `materials: []` so they are never sampled, and list one after the shared palette. Switching an object's knob is a one-integer edit.
3. Single source of truth: the pool cell. Editing it updates every palette that references it.
4. Per-voxel variation: move the property from `scalarProperties` to `arrayProperties`, giving it a per-material value-index and a channel.
5. Whole-object override: list a scalar-property palette after the layer it overrides; the object-wide value replaces the per-voxel values for that property.

Idiom 2, two lamp objects sharing one base palette but glowing at different strengths:

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
  // no channel, so `voxelSamples` has one channel, for palette 0
  { /* ... */ "layers": [0, 1] },

  // "Neon Sign": the same base palette; switching knobs is a one-integer
  // edit in `layers`
  { /* ... */ "layers": [0, 2] },
]
```

### Properties

A property is a named material parameter, listed in a palette's `arrayProperties[].name` and `scalarProperties[].name`. The format wires properties without defining them: the name carries the meaning and the pool carries the values; that pairing is all the format defines. A property's meaning and value range are convention between producer and consumer, not part of the wire format. A consumer ignores any property name it does not recognize, so extending the vocabulary never breaks an older reader (see [Versioning and Extensibility](#versioning-and-extensibility)).

voxj's recommended vocabulary is glTF's, below. The format neither requires nor privileges it; it is the convention voxj tools target.

#### glTF conventions

The recommended property vocabulary is glTF's metallic-roughness model, so a voxj material maps one-to-one onto a glTF material and the defaults below are glTF's own. Each property binds a value pool of one of the kinds listed, and an unbound property renders at its default:

| Property             | Kind                                                         | Range | Default     | Meaning                                                                            |
| -------------------- | ------------------------------------------------------------ | ----- | ----------- | ---------------------------------------------------------------------------------- |
| `baseColorFactor`    | `srgba-hex` (default), `srgba-float`, or `linear-rgba-float` |       | `#FFFFFFFF` | Base color, straight alpha = opacity (glTF `baseColorFactor`)                      |
| `metallicFactor`     | `float`, `min: 0`, `max: 1`                                  | 0-1   | `1`         | Metalness (glTF `metallicFactor`)                                                  |
| `roughnessFactor`    | `float`, `min: 0`, `max: 1`                                  | 0-1   | `1`         | Roughness (glTF `roughnessFactor`)                                                 |
| `occlusionStrength`  | `float`, `min: 0`, `max: 1`                                  | 0-1   | `1`         | Flat ambient occlusion, 1 = none (glTF `occlusionTexture.strength`)                |
| `emissiveFactor`     | `srgb-hex` (default), `srgb-float`, or `linear-rgb-float`    |       | `#000000`   | Emissive color, black = none (glTF `emissiveFactor`)                               |
| `emissiveStrength`   | `float`, `min: 0`                                            | 0+    | `1`         | Multiplies emissive color in linear space (glTF `KHR_materials_emissive_strength`) |
| `ior`                | `float`, `min: 1`                                            | 1+    | `1.5`       | Index of refraction (glTF `KHR_materials_ior`)                                     |
| `transmissionFactor` | `float`, `min: 0`, `max: 1`                                  | 0-1   | `0`         | Light transmission through surface (glTF `KHR_materials_transmission`)             |

A color property binds a hex or float-component color kind in either the sRGB or linear space (see [Value Pool Kinds](#value-pool-kinds)); hex, which is sRGB only, is the authoring default. Base color takes an alpha-carrying kind and emission an alpha-less one, and all forms carry the same color.

Emission is two properties. `emissiveFactor` is the emitted color, `srgb-hex` by default or a vector form, no alpha, default `#000000` for no emission, authored and linearized like `baseColorFactor`. `emissiveStrength` is a numeric multiplier over that color, `float` with `min: 0`, default `1`; values above `1` push emission into HDR/bloom range. Rendered emission is `linearize(emissiveFactor) * emissiveStrength`. The defaults compose: a black color emits nothing at any strength, and a color left at the default strength `1` emits at face value, so a material that sets only `emissiveFactor` emits that color at strength 1. A strength shared by a whole palette or object is typically wired as a scalar property (see [Palettes](#palettes)).

## Hierarchy Nodes

Nodes form a DAG: a node may have multiple parents but no cycles. Each references child nodes and child objects by index and carries a transform.

```jsonc
{
  "name": "parent-1",
  "transform": {
    "position": [0, 0, 0],
    "rotation": [0, 0, 0, 1],
    "scale": [1, 1, 1],
  },
  "childNodes": [1],
  "childObjects": [0],
}
```

A transform has three fields: `position` is a possibly-fractional `[x, y, z]`, `rotation` is a unit quaternion `[x, y, z, w]`, and `scale` is `[x, y, z]`.

1. A transform composes as `Translation * Rotation * Scale`.
2. A node's world transform is `parentWorld * nodeLocal`; a root, listed in `rootNodes`, has world = local. Reached through multiple parents, a node is placed once per path; this is instancing.
3. An object is placed at the world transform of the node referencing it. A voxel at grid position `p` sits at node-local position `origin + p`, so its world position is the node's world transform applied to `origin + p`.
4. `rotation` must be a unit quaternion; consumers may renormalize within a small tolerance (see [Validation](#validation)).
5. `scale` is per-axis; a negative component mirrors that axis and flips winding/handedness. A zero component is degenerate and invalid (see [Validation](#validation)).

The scene's roots are exactly the nodes listed in `rootNodes`. A node that is neither listed as a root nor referenced as a child is unplaced and does not render, so a file may hold library nodes that are defined without being placed. The format describes placement only; how overlapping or sub-voxel placements are resolved, merged, or rasterized is consumer-defined.

## Edit State

`main.editState` is optional editor state, kept separate from the runtime scene. When present, its `objects` array has one entry per runtime object, aligned by index, giving the author's edit grid (the build volume) for that object.

```jsonc
{
  "objects": [
    {
      // `[X, Y, Z]` size of the edit grid in voxels
      "bounds": [6, 6, 6],

      // `[X, Y, Z]` integer translation from the placing node to the edit
      // grid's min corner
      "origin": [-1, -1, -1],
    },
  ],
}
```

Each edit grid must contain its object's runtime grid: on every axis the edit `origin` is `<=` the runtime `origin`, and the edit `origin + bounds` is `>=` the runtime `origin + bounds`. An empty object is a point at its `origin`, and containment reduces to that point lying inside the edit grid. An edit grid equal to the runtime grid carries no margin. `editState` is omitted when no object has a distinct edit grid; a consumer that ignores it loses only editor margin, never geometry.

## Extensions

`main.ext` is a reserved namespace for user-defined extensions, conventionally keyed by vendor. The core format assigns it no meaning and makes no compatibility guarantees about its contents; consumers ignore extensions they do not recognize. Its contents are arbitrary JSON, `null` included. For example, Voxel Max's camera:

```jsonc
{
  "ext": {
    "voxel-max": {
      "version": 4,
      "camera": {
        "angles": [
          /* ... */
        ],
        "light": [
          /* ... */
        ],
        "pan": [0, 0],
        "zoom": 1,
        "origin": [
          /* ... */
        ],
      },
    },
  },
}
```

## Versioning and Extensibility

1. An unrecognized `version` must be rejected.
2. An unknown `encoding` (positions or samples) must be rejected; the block cannot be safely decoded.
3. An unrecognized value pool `kind` must be rejected: the pool's values cannot be safely validated, exactly as an unknown `encoding` cannot be safely decoded, and it must never be reinterpreted or downgraded. `kind` is required and has no default.
4. Unknown property **names** in `arrayProperties` and `scalarProperties` are ignored, since properties are advisory and convention-based, so adding properties is backward compatible.
5. Ignore vs reject: unknown property names are ignored; unknown `kind`, `encoding`, and `version`, and unknown object keys in any core structure, are rejected. Each is a contract a consumer must understand to make its guarantees.

## Validation

Validation is a hard contract, not best-effort. A validator rejects any file that violates a rule below; it never repairs, coerces, or fills in bad or missing input. Every field is required except the two on the optional allowlist: `main.editState`, whose absence means no editor margin, and `main.ext`, whose absence means no extensions. The closed vocabularies `version`, `encoding`, and value pool `kind` reject any unrecognized value and are never reinterpreted or downgraded.

### Rules

1. `version` is recognized.
2. Every `encoding`, on both `voxelPositions` and `voxelSamples`, is recognized.
3. Types are exact and nothing is coerced. A string where a number is expected, or the reverse, rejects. Every integer-valued number has no fractional part, and every number is finite, so `NaN` and `+/-Infinity` reject.
4. `null` rejects everywhere except in a `json`-kind pool's `values` and inside `main.ext`: in every structural field, every non-`json` pool's `values`, and every block's `data`.
5. Unknown keys reject in every closed structure: file, `main`, `runtimeState`, `editState`, object, encoding block, palette, array property, scalar property, value pool, transform, hierarchy node, and edit object. The only open points are `main.ext` and property names.
6. All indices are in range:
   1. each object `layers` entry indexes `runtimeState.palettes`.
   2. each array and scalar property `valuePool` indexes `runtimeState.valuePools`.
   3. each `childNodes` entry indexes `runtimeState.nodes`.
   4. each `childObjects` entry indexes `runtimeState.objects`.
   5. each `rootNodes` entry indexes `runtimeState.nodes`.
7. References are unique: no hierarchy node lists the same child node or the same child object twice, and no node appears in `rootNodes` twice.
8. **Objects**, per object:
   1. `layers` is present, an array of integers, possibly empty.
   2. `voxelPositions` and `voxelSamples` are present; the Positions and Samples rules check their structure.
9. **Value pools** (`runtimeState.valuePools`): an array, possibly empty. Each pool's keys are drawn only from { `kind`, `values`, `min`, `max` }.
   1. `kind` is present and recognized.
   2. `values` is present, a non-empty array, with no `null` entry unless `kind` is `json`. Every entry is well-formed for `kind`:
      1. `json`: any JSON value, including `null`.
      2. `string`: a JSON string.
      3. `bool`: a JSON boolean.
      4. `int`: an integer-valued finite number within `min`/`max`.
      5. `float`: a finite number within `min`/`max`.
      6. `srgb-hex`: matches `^#[0-9A-F]{6}$`.
      7. `srgba-hex`: matches `^#[0-9A-F]{8}$`.
      8. `srgb-float`: an array of exactly 3 finite numbers, each in `[0, 1]`; `linear-rgb-float`: exactly 3 finite numbers, each `>= 0`.
      9. `srgba-float`: an array of exactly 4 finite numbers, each in `[0, 1]`; `linear-rgba-float`: exactly 4 finite numbers, each `>= 0`.
   3. `min` and `max`:
      1. both present when `kind` is `int` or `float`, and both absent for every other kind.
      2. each is a finite number or the string `none`, meaning unbounded on that side.
      3. a numeric bound is integer-valued when `kind` is `int`.
      4. `min <= max` when both are finite numbers.
10. **Palettes** (`runtimeState.palettes`): an array, possibly empty. Each palette's keys are drawn only from { `arrayProperties`, `scalarProperties`, `materials` }.
    1. `arrayProperties` is an array, possibly empty; each array property has exactly the keys `name`, a non-empty string, and `valuePool`, an integer. `scalarProperties` is an array, possibly empty; each scalar property has exactly the keys `name`, a non-empty string, `valuePool`, an integer, and `valueIndex`, an integer.
    2. no two properties share a `name`, across `arrayProperties` and `scalarProperties` together.
    3. `materials` is an array of `M >= 0` rows, the material count; every row is an array of exactly `arrayProperties.length` integers, one value-index per array property in property order.
    4. every `materials[m][b]` is an integer in `[0, valuePools[arrayProperties[b].valuePool].values.length)`.
    5. every scalar property's `valueIndex` is an integer in `[0, valuePools[valuePool].values.length)`.
    6. a palette with a non-empty `arrayProperties` has a non-empty `materials`.
11. **Samples**: let `V` be the voxel count from the position block. A layer is sampled iff the material count `M` of its palette is greater than zero. `voxelSamples.data` has exactly one channel per sampled layer, in `layers` order, so channel `c` belongs to the `c`-th sampled layer. For channel `c`, let `M` be the material count of its layer's palette, and by encoding:
    1. `raw-json`: each channel is a `number[]` of length exactly `V`, every entry an integer in `[0, M)`.
    2. `rle-json`: each channel is a flat even-length `[value, count, ...]` stream whose values are integers in `[0, M)`, whose counts are positive integers, and whose counts sum to exactly `V`.
    3. `packed-base64`: each channel is a base64 string decoding to exactly `ceil(V * b / 8)` bytes for `b = max(1, bitLength(M - 1))`, its pad bits zero, every decoded value `< M`.
12. Sample order matches the position block's voxel order (see [Voxel Order](#voxel-order)). This is an authoring invariant a validator cannot confirm.
13. **Positions**: `voxelPositions.data` is well-formed for its encoding:
    1. `raw-json`: `[x, y, z]` integer triples.
    2. `bitmap-base64`: decodes to exactly `ceil(X * Y * Z / 8)` bytes, its pad bits zero, and the voxel count equals the number of set bits.
    3. `hilbert-delta-varint-base64`:
       1. `data` decodes to an unsigned LEB128 varint stream of non-negative deltas, every delta after the first strictly positive.
       2. `bits` derived from `bounds` is `<= 17`, equivalently every `bounds` dimension is `<= 131072`.
       3. every decoded position lies in `[0, X) x [0, Y) x [0, Z)`.
14. Every base64 field is canonical RFC 4648: the standard alphabet, not base64url, with correct `=` padding and no whitespace or line breaks.
15. Voxel positions within an object are unique after decoding.
16. `bounds` is three non-negative integers, exactly tight around the decoded positions: on each axis the minimum voxel coordinate is `0` and `bounds` is the maximum plus one. An empty object has `bounds = [0, 0, 0]`. `origin` is three integers.
17. The hierarchy is acyclic.
18. No transform `scale` component is zero.
19. Every transform `rotation` has length-squared within `1e-6` of `1`; consumers may renormalize within this tolerance.
20. When `editState` is present, its `objects` has exactly one entry per runtime object. Each edit object's `bounds` is three non-negative integers and its `origin` is three integers, and the edit grid contains the runtime grid: on every axis edit `origin` is `<=` runtime `origin`, and edit `origin + bounds` is `>=` runtime `origin + bounds`. For an empty object this reduces to its point lying inside the edit grid.

## Examples

### File Example

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

        // one shared float pool, bound by `metallicFactor` and
        // `roughnessFactor`
        { "kind": "float", "min": 0, "max": 1, "values": [0, 0.5, 1] },

        { "kind": "srgb-hex", "values": ["#000000", "#FF6600"] },

        { "kind": "linear-rgba-float", "values": [[1, 0, 0, 1]] },

        // emissive strengths, referenced by cell from a scalar property
        { "kind": "float", "min": 0, "max": "none", "values": [1, 5] },
      ],

      "palettes": [
        // value pool 1 is bound twice, to `metallicFactor` and
        // `roughnessFactor`
        {
          "arrayProperties": [
            { "name": "baseColorFactor", "valuePool": 0 },
            { "name": "metallicFactor", "valuePool": 1 },
            { "name": "roughnessFactor", "valuePool": 1 },
            { "name": "emissiveFactor", "valuePool": 2 },
          ],

          "scalarProperties": [],

          // one row per material, a value-index per array property. Material
          // 2 resolves to `baseColorFactor` #0000FFFF, `metallicFactor` 0.5,
          // `roughnessFactor` 0, `emissiveFactor` #FF6600.
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
          // sampled and both bind `baseColorFactor`, so the later layer
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
          // sampled and supplies `emissiveStrength` 5 to the whole object.
          // `voxelSamples` has one channel, for palette 0.
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

### TypeScript Schema

```ts
interface VoxelJsonFile {
  version: 1;

  main: Main;
}

interface Main {
  runtimeState: RuntimeState;

  // optional editor state, aligned by index with the runtime objects
  editState?: EditState;

  // user-defined extensions, conventionally vendor-keyed; the core format
  // assigns no meaning and guarantees nothing about its contents
  ext?: { [key: string]: JsonValue };
}

// The runtime scene.
interface RuntimeState {
  // shared value pools, referenced by index from palette properties
  valuePools: ValuePool[];

  palettes: Palette[];

  objects: VoxelObject[];

  nodes: HierarchyNode[];

  // indices into `nodes`; the scene's roots
  rootNodes: number[];
}

// Optional editor state: one edit grid per runtime object, aligned by index.
interface EditState {
  objects: EditObject[];
}

// One object's edit grid (build volume), which must contain its runtime grid.
interface EditObject {
  // `[X, Y, Z]` size of the edit grid in voxels
  bounds: Vec3;

  // `[X, Y, Z]` integer translation from the placing node to the edit grid's
  // min corner
  origin: Vec3;
}

// Pure geometry; placed only by a hierarchy node that references it.
interface VoxelObject {
  name: string;

  // `[X, Y, Z]` size in voxels; voxels occupy `[0, X) x [0, Y) x [0, Z)`.
  // Exactly tight: per-axis the min voxel coordinate is 0 and the bound is
  // the max plus one; `[0, 0, 0]` when empty, a point at origin. No margin
  // here (that is `editState`). Required to decode `bitmap-base64` and
  // `hilbert-delta-varint-base64`.
  bounds: Vec3;

  // `[X, Y, Z]` integer translation from the placing node to the grid's min
  // corner. Does not affect the voxel encodings.
  origin: Vec3;

  voxelPositions: PositionBlock;

  // palette indices, ordered back to front; each layer supplies all of its
  // palette's properties and later layers override earlier ones. A layer is
  // sampled iff its palette has materials (`M > 0`); each sampled layer
  // carries one `voxelSamples` channel (see Objects)
  layers: number[];

  voxelSamples: SampleBlock;
}

// ## Voxel Encoding

// Both blocks share one voxel order, fixed by the position encoding (see Voxel
// Order); every sample channel is in that order. The match is an authoring
// invariant that validation cannot verify.

type PositionBlock =
  // One `[x, y, z]` per voxel, in listing order.
  | { encoding: "raw-json"; data: Vec3[] }
  // Dense occupancy bitmap over `bounds` (required to decode): one bit per
  // cell `k = x * Y * Z + y * Z + z`, packed 8 per byte MSB-first,
  // base64-encoded. Canonical order is ascending `k`.
  | { encoding: "bitmap-base64"; data: string }
  // Prefix-sum deltas of each voxel's 3D Hilbert-curve index (see
  // `hilbertEncode`/`hilbertDecode`), voxels sorted by ascending index;
  // deltas as an unsigned-LEB128 varint stream, base64-encoded. Requires
  // `bits <= 17` (every `bounds` dimension <= 131072).
  | { encoding: "hilbert-delta-varint-base64"; data: string };

type SampleBlock =
  // One channel per sampled layer (in `layers` order): that layer's material
  // index for every voxel, in voxel order.
  | { encoding: "raw-json"; data: number[][] }
  // One channel per sampled layer: a flat run stream
  // `[value1, count1, value2, count2, ...]`.
  | { encoding: "rle-json"; data: number[][] }
  // One channel per sampled layer: each voxel's material index bit-packed at
  // width `b = max(1, bitLength(M - 1))` for that layer's palette material
  // count `M`, MSB-first, base64-encoded (same packing as the
  // `bitmap-base64` position encoding).
  | { encoding: "packed-base64"; data: string[] };

// ## Palettes

// A palette binds property names to value pools, then lists its materials:
// one row per material, a value-index per array property in property order,
// so the material count `M` is `materials.length`. A voxel samples material
// `m`; property `arrayProperties[b].name` takes
// `valuePools[arrayProperties[b].valuePool].values[materials[m][b]]`, and
// each scalar property takes `valuePools[valuePool].values[valueIndex]`, one
// value for the whole palette. Layers apply in `layers` order; each property
// takes its value from the last layer that supplies it.
interface Palette {
  arrayProperties: ArrayProperty[];

  scalarProperties: ScalarProperty[];

  materials: number[][];
}

// One property bound to a whole pool, one value-index per material.
interface ArrayProperty {
  // property name (see Properties); advisory, unknown names ignored
  name: string;

  // index into `RuntimeState.valuePools`
  valuePool: number;
}

// One property pinned to a single pool cell, one value for the whole
// palette.
interface ScalarProperty {
  // property name (see Properties); advisory, unknown names ignored
  name: string;

  // index into `RuntimeState.valuePools`
  valuePool: number;

  // index into `valuePools[valuePool].values`
  valueIndex: number;
}

// A shared pool of values, all of one shape given by `kind`, each kind's
// values typed to its shape. Only `int` and `float` carry `min`/`max`, each
// a finite number or "none"; a color kind's range is fixed by its color
// space, so no color kind carries bounds.
type ValuePool =
  // arbitrary JSON, including `null`
  | { kind: "json"; values: JsonValue[] }
  // booleans
  | { kind: "bool"; values: boolean[] }
  // finite floats within `min`/`max`
  | {
      kind: "float";
      min: number | "none";
      max: number | "none";
      values: number[];
    }
  // integers within `min`/`max`
  | {
      kind: "int";
      min: number | "none";
      max: number | "none";
      values: number[];
    }
  // strings
  | { kind: "string"; values: string[] }
  // sRGB float colors, each component in `[0, 1]`
  | { kind: "srgb-float"; values: [number, number, number][] }
  // `#RRGGBB` sRGB hex strings
  | { kind: "srgb-hex"; values: string[] }
  // sRGB float colors with alpha, each component in [0, 1]
  | { kind: "srgba-float"; values: [number, number, number, number][] }
  // `#RRGGBBAA` sRGB hex strings
  | { kind: "srgba-hex"; values: string[] }
  // linear float colors, each component `>= 0`
  | { kind: "linear-rgb-float"; values: [number, number, number][] }
  // linear float colors with alpha, each component >= 0
  | { kind: "linear-rgba-float"; values: [number, number, number, number][] };

// Closed value-shape vocabulary (see Value Pool Kinds).
type PoolKind =
  | "json"
  | "bool"
  | "float"
  | "int"
  | "string"
  | "srgb-float"
  | "srgb-hex"
  | "srgba-float"
  | "srgba-hex"
  | "linear-rgb-float"
  | "linear-rgba-float";

type JsonValue =
  | string
  | number
  | boolean
  | null
  | JsonValue[]
  | { [key: string]: JsonValue };

// ## Hierarchy

interface HierarchyNode {
  name: string;

  transform: Transform;

  // indices into `RuntimeState.nodes` (DAG, no cycles)
  childNodes: number[];

  // indices into `RuntimeState.objects`
  childObjects: number[];
}

interface Transform {
  // `[x, y, z]`
  position: Vec3;

  // unit quaternion `[x, y, z, w]`
  rotation: Quat;

  // `[x, y, z]`
  scale: Vec3;
}

type Vec3 = [number, number, number];

type Quat = [number, number, number, number];
```

### Reference Code

Reference implementations of the binary encodings, as small independent codecs to port directly. `raw-json` positions and samples are plain JSON and need none. `base64` / `unbase64` are standard RFC 4648 (`btoa`/`atob` or `Buffer` in JS; see [Voxel Encoding](#voxel-encoding)). `Vec3` is `[number, number, number]`. Each block's `data` is one composition below.

#### Bit Widths

```ts
// Binary digits in a non-negative integer, `bitLength(0) = 0`. The width
// formulas call `bitLength(x - 1)`: the bits to index `x` distinct values,
// integer-exact with no floating point. Never use `Math.log2` for these.
function bitLength(n: number): number {
  let len = 0;
  while (n > 0) {
    n = Math.floor(n / 2);
    len++;
  }
  return len;
}

// Hilbert `bits` per axis from `bounds`, and `packed-base64` channel width
// from a palette material count. Both are `max(1, ceil(log2(.)))` via
// `bitLength`.
function hilbertBits(bounds: Vec3): number {
  return Math.max(1, bitLength(Math.max(bounds[0], bounds[1], bounds[2]) - 1));
}

function packedWidth(materialCount: number): number {
  return Math.max(1, bitLength(materialCount - 1));
}
```

#### Bit-Packing: `bitmap-base64` positions and `packed-base64` samples

```ts
// Pack `values` at a fixed `width` bits each, MSB-first, 8 per byte, final byte
// zero-padded. `bitmap-base64` is the `width = 1` case.
function packBits(values: number[], width: number): Uint8Array {
  const out = new Uint8Array(Math.ceil((values.length * width) / 8));
  let bit = 0;
  for (const value of values) {
    for (let b = width - 1; b >= 0; b--) {
      if ((value >> b) & 1) out[bit >> 3] |= 1 << (7 - (bit & 7));
      bit++;
    }
  }
  return out;
}

// Inverse of `packBits`. Bytes past the end read as zero.
function unpackBits(bytes: Uint8Array, width: number, count: number): number[] {
  const out: number[] = [];
  let bit = 0;
  for (let i = 0; i < count; i++) {
    let value = 0;
    for (let w = 0; w < width; w++) {
      const byte = bytes[bit >> 3] ?? 0;
      value = (value << 1) | ((byte >> (7 - (bit & 7))) & 1);
      bit++;
    }
    out.push(value);
  }
  return out;
}

// Raster cell index of a position within `bounds`: x outermost, z innermost.
function cellIndex([x, y, z]: Vec3, [, Y, Z]: Vec3): number {
  return x * Y * Z + y * Z + z;
}

// `bitmap-base64`: one occupancy bit per cell of `bounds`, packed at width
// 1. The canonical voxel order is ascending cell index, so reorder each
// sample channel to match (sort voxel indices by `cellIndex` for the same
// `remap` that `encodeHilbertBlockWithRemap` returns).
function encodeBitmapBlock(positions: Vec3[], bounds: Vec3): string {
  const occupancy = new Array<number>(bounds[0] * bounds[1] * bounds[2]).fill(
    0,
  );
  for (const p of positions) occupancy[cellIndex(p, bounds)] = 1;
  return base64(packBits(occupancy, 1));
}

function decodeBitmapBlock(data: string, bounds: Vec3): Vec3[] {
  const [X, Y, Z] = bounds;
  const cells = X * Y * Z;
  const occupancy = unpackBits(unbase64(data), 1, cells);
  const out: Vec3[] = [];
  for (let k = 0; k < cells; k++) {
    if (occupancy[k]) {
      out.push([Math.floor(k / (Y * Z)), Math.floor((k % (Y * Z)) / Z), k % Z]);
    }
  }
  return out;
}

// `packed-base64`: one layer's channel, each voxel's material index packed
// at `packedWidth(materialCount)`. `samples` is in the position block's
// voxel order.
function encodePackedChannel(samples: number[], materialCount: number): string {
  return base64(packBits(samples, packedWidth(materialCount)));
}

function decodePackedChannel(
  data: string,
  materialCount: number,
  voxelCount: number,
): number[] {
  return unpackBits(unbase64(data), packedWidth(materialCount), voxelCount);
}
```

#### Run-Length: `rle-json` samples

```ts
// `rle-json`: one layer's channel as a flat `[value, count, ...]` run
// stream; counts are positive and sum to the voxel count.
function rleEncode(samples: number[]): number[] {
  const out: number[] = [];
  for (let i = 0; i < samples.length; ) {
    const value = samples[i];
    let count = 1;
    while (samples[i + count] === value) count++;
    out.push(value, count);
    i += count;
  }
  return out;
}

function rleDecode(rle: number[]): number[] {
  const out: number[] = [];
  for (let i = 0; i < rle.length; i += 2) {
    for (let c = 0; c < rle[i + 1]; c++) out.push(rle[i]);
  }
  return out;
}
```

#### Hilbert: `hilbert-delta-varint-base64` positions

```ts
// Three independent encode/decode codecs (hilbert, delta, varint) plus a
// composition. The block `data` is
// `base64(varintEncode(deltaEncode(sortedHilbertIndices)))`.
//
// NOTE: indices are assembled with arithmetic, not `<<`, because JS bitwise
// operators are 32-bit and an index can exceed 31 bits on large grids. The
// index is exact in a JS `number` only while `3 * bits <= 53`, i.e.
// `bits <= 17`; the format caps `bits` at 17 (every `bounds` dimension
// <= 131072) for this reason.

// 1. Hilbert: a position <-> its 3D Hilbert-curve index (Skilling's transform),
//    `bits` bits per axis.
function hilbertEncode(x: number, y: number, z: number, bits: number): number {
  const axes = [x, y, z];
  const topBit = 1 << (bits - 1);

  for (let mask = topBit; mask > 1; mask >>= 1) {
    const lower = mask - 1;
    for (let i = 0; i < 3; i++) {
      if (axes[i] & mask) {
        axes[0] ^= lower;
      } else {
        const t = (axes[0] ^ axes[i]) & lower;
        axes[0] ^= t;
        axes[i] ^= t;
      }
    }
  }

  for (let i = 1; i < 3; i++) axes[i] ^= axes[i - 1];
  let t = 0;
  for (let mask = topBit; mask > 1; mask >>= 1) {
    if (axes[2] & mask) t ^= mask - 1;
  }
  for (let i = 0; i < 3; i++) axes[i] ^= t;

  // Interleave into a single index (`axes[0]` most significant).
  let index = 0;
  for (let k = bits - 1; k >= 0; k--) {
    for (let d = 0; d < 3; d++) {
      index = index * 2 + ((axes[d] >> k) & 1);
    }
  }
  return index;
}

function hilbertDecode(index: number, bits: number): [number, number, number] {
  const totalBits = 3 * bits;
  const axes = [0, 0, 0];

  // De-interleave the index back into the three axes.
  for (let p = 0; p < totalBits; p++) {
    const bitValue = Math.floor(index / 2 ** (totalBits - 1 - p)) % 2;
    const k = bits - 1 - Math.floor(p / 3);
    axes[p % 3] |= bitValue << k;
  }

  // Invert the encode transform.
  const size = 2 << (bits - 1);
  const gray = axes[2] >> 1;
  for (let i = 2; i > 0; i--) axes[i] ^= axes[i - 1];
  axes[0] ^= gray;
  for (let mask = 2; mask !== size; mask <<= 1) {
    const lower = mask - 1;
    for (let i = 2; i >= 0; i--) {
      if (axes[i] & mask) {
        axes[0] ^= lower;
      } else {
        const t = (axes[0] ^ axes[i]) & lower;
        axes[0] ^= t;
        axes[i] ^= t;
      }
    }
  }

  return [axes[0], axes[1], axes[2]];
}

// 2. Delta: an ascending integer sequence <-> its successive differences.
//    `deltaDecode` is the prefix sum; `deltaEncode` assumes ascending
//    input, which keeps every delta after the first strictly positive.
function deltaEncode(values: number[]): number[] {
  return values.map((v, i) => (i === 0 ? v : v - values[i - 1]));
}

function deltaDecode(deltas: number[]): number[] {
  const out: number[] = [];
  let acc = 0;
  for (const d of deltas) {
    acc += d;
    out.push(acc);
  }
  return out;
}

// 3. Varint: a non-negative integer array <-> an unsigned-LEB128 byte stream.
//    Uses arithmetic (not `<<` / `>>`) so values above `2^31` stay exact.
function varintEncode(values: number[]): Uint8Array {
  const out: number[] = [];
  for (let v of values) {
    while (v >= 0x80) {
      out.push((v & 0x7f) | 0x80);
      v = Math.floor(v / 128);
    }
    out.push(v);
  }
  return Uint8Array.from(out);
}

function varintDecode(bytes: Uint8Array): number[] {
  const out: number[] = [];
  let i = 0;
  while (i < bytes.length) {
    let v = 0;
    let scale = 1;
    let b: number;
    do {
      b = bytes[i++];
      v += (b & 0x7f) * scale;
      scale *= 128;
    } while (b & 0x80);
    out.push(v);
  }
  return out;
}

// Compose. `base64` / `unbase64` are standard (`btoa` + `atob` in the
// browser, `Buffer` in Node). Voxels are sorted by ascending Hilbert index
// so the deltas stay positive. `bits` is `hilbertBits(object.bounds)`;
// `packed-base64` uses `packedWidth(materialCount)`.
function encodeHilbertBlock(positions: Vec3[], bits: number): string {
  const idx = positions
    .map((p) => hilbertEncode(p[0], p[1], p[2], bits))
    .sort((a, b) => a - b);
  return base64(varintEncode(deltaEncode(idx)));
}

function decodeHilbertBlock(data: string, bits: number): Vec3[] {
  const idx = deltaDecode(varintDecode(unbase64(data)));
  return idx.map((h) => hilbertDecode(h, bits));
}

// Like `encodeHilbertBlock`, but also returns `remap`, the permutation from
// input order to the block's canonical order: `remap[oldIndex] = newIndex`.
// Reorder each sample array `s` to match with `out[remap[i]] = s[i]`.
function encodeHilbertBlockWithRemap(
  positions: Vec3[],
  bits: number,
): { data: string; remap: number[] } {
  const n = positions.length;

  const indices = new Array<number>(n);
  for (let i = 0; i < n; i++) {
    const p = positions[i];
    indices[i] = hilbertEncode(p[0], p[1], p[2], bits);
  }

  // `order[newIndex] = oldIndex`, sorted by ascending Hilbert index.
  const order = new Array<number>(n);
  for (let i = 0; i < n; i++) order[i] = i;
  order.sort((a, b) => indices[a] - indices[b]);

  const remap = new Array<number>(n);
  const sortedIndices = new Array<number>(n);
  for (let newIndex = 0; newIndex < n; newIndex++) {
    const oldIndex = order[newIndex];
    remap[oldIndex] = newIndex;
    sortedIndices[newIndex] = indices[oldIndex];
  }

  return { data: base64(varintEncode(deltaEncode(sortedIndices))), remap };
}
```
