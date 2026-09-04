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

  // palette references, ordered back to front
  "layers": [0],
  // one channel per layer; each is one material index per voxel
  "voxelSamples": { "encoding": "raw-json", "data": [[0]] },
}
```

An object carries any number of layers, listed in `layers` as palette indices, back to front; a palette may appear more than once. Each layer maps every voxel to one material in its palette and supplies that material's properties; later layers override earlier ones (see [Palettes](#palettes)).

`voxelPositions` and `voxelSamples` are encoded blocks (see [Voxel Encoding](#voxel-encoding)). Each voxel has a position `(x, y, z)` and one material sample per layer. `voxelSamples` carries one channel per layer, in `layers` order, and the sample in channel `c` is a material index into the palette `layers[c]`. The number of voxels is implicit: it is the number of positions decoded from `voxelPositions`. Positions within an object must be unique.

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

A sample block holds one channel per layer, in `layers` order. Each channel gives, for every voxel in the position block's voxel order, a material index into that layer's palette.

1. `raw-json`: one channel per layer, each a plain array of that layer's material index for every voxel: `[[l0v0, l0v1, ...], [l1v0, l1v1, ...], ...]`.
2. `rle-json`: one channel per layer; each channel is a flat run-length encoding `[value1, count1, value2, count2, ...]`. Counts are positive integers and, in every channel, sum to the number of voxels.
3. `packed-base64`: one bit-packed channel per layer. For the channel of a layer whose palette has `M` materials, each voxel's material index is packed at fixed width `b = max(1, bitLength(max(1, M) - 1))` bits, MSB-first, 8 per byte, with the final byte zero-padded; the width is derived from `M` and not stored. At `M = 0` no sample can exist, so the layer's object has no voxels and its channel is `""`. `data` is one base64 string per layer, in `layers` order, each encoding exactly `ceil(voxelCount * b / 8)` bytes. This is the same packing scheme as the `bitmap-base64` position encoding, which is its `b = 1` special case. An empty object has one `""` per layer. Best for incoherent or many-material objects, where `rle-json` would approach one run per voxel.

#### Example: two layers over four voxels; layer 0 material indices `0, 0, 0, 1` (palette `M = 2`) and layer 1 material indices `2, 2, 3, 3` (palette `M = 4`), in the position block's voxel order

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

The position block defines the object's single canonical voxel order, and every sample channel is in that same order, voxel-for-voxel, for every combination of position and sample encoding:

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

Value pools live in `main.runtimeState.valuePools`, a shared array referenced by index, siblings of `objects` and `palettes`. A value pool holds `values`, all of one value-shape given by `kind`. Palettes reference value pools by index (see [Palettes](#palettes)).

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

### Value Pool Kinds

`kind` is a closed vocabulary naming the shape of a value pool's `values`. Every kind's `values` are plain readable JSON literals, and the declared kind types the whole array. A consumer must reject a file whose `kind` it does not recognize (see [Versioning and Extensibility](#versioning-and-extensibility)).

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

1. A kind is one value-shape, and the shape is the whole contract. What a value means, a color, a normal, a count, is the binding property's concern (see [Properties](#properties)).
2. A float value is a finite JSON number, the string `"inf"`, or the string `"-inf"`. JSON has no infinity literal, so the sentinel strings stand in for the two infinities. `NaN` has no sentinel and writers error on it.
3. An int value is a JSON number spelled as an integer, so `3.0` and `3e0` reject. Its magnitude is at most `2^53 - 1`, so a consumer reading numbers as doubles cannot silently lose one. `"inf"` and `"-inf"` reject as int values: an infinite integer means nothing.
4. A vector kind's value is an array of exactly the kind's length: float values for the `vec-*-float` kinds, int values for the `vec-*-int` kinds. A scalar is not a one-element vector: `0.5` and `[0.5]` are different JSON, so `int` and `float` stand apart from the vector kinds.
5. A kind carries no range. A range is a fact about the binding property and rides the property vocabulary (see [Properties](#properties)).
6. `kind` is required and has no default. A value pool has no optional fields.

```jsonc
{ "kind": "float", "values": [0, 0.5, "inf"] } // fine
{ "kind": "int", "values": [3.0] }             // rejects: 3.0 is not 3
{ "kind": "int", "values": ["inf"] }           // rejects: no infinite int
```

## Palettes

A palette binds property names to shared [value pools](#value-pools), then lists the distinct materials it uses as rows over those value pools. A voxel samples a material in each layer by its index in that layer's palette. A palette may be referenced by any number of layers and objects (see [Objects](#objects)).

A material is one row of value-indices, one per property, and with no properties every row is empty. The material count `M` is `materials.length`. A palette with no materials offers nothing to sample, so only a layer of an empty object can reference it:

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

To resolve a voxel's properties:

1. In each layer, read the voxel's sample `m` from the layer's channel, a material index.
2. Property `b` supplies `properties[b].name` as `valuePools[properties[b].valuePool].values[materials[m][b]]`.
3. Each property takes its value from the last layer that supplies it, in `layers` order. Three layers supplying `{a, b, c}`, then `{a}`, then `{c}` resolve to `b` from the first, `a` from the second, and `c` from the third.
4. Unbound properties are left to the vocabulary; the recommended glTF conventions supply a default for each (see [Properties](#properties)).

### Sharing Idioms

One value pool cell can feed any number of materials, and editing it updates them all:

1. All materials of one palette share a value: every row repeats the same value-index.
2. Per-object variation over a shared base: sibling palettes that share the same value pools and differ in one column. Switching an object between variants is a one-integer edit in `layers`; the channel data is identical across variants.

Idiom 2, two lamp objects sharing their value pools but glowing at different strengths:

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

### Properties

A property is a named material parameter, listed in a palette's `properties[].name`. The format pairs each name with a value pool and leaves its meaning and value range as convention between producer and consumer; tools that understand the vocabulary may check a range like `metallic`'s `[0, 1]`. A consumer ignores any property name it does not recognize, so any tool may bind names of its own (see [Versioning and Extensibility](#versioning-and-extensibility)).

voxj's recommended vocabulary is glTF's, below. The format neither requires nor privileges it; it is the convention voxj tools target.

#### glTF conventions

The recommended property vocabulary is glTF's metallic-roughness model. A material that follows it maps onto a glTF material, and the defaults below are glTF's own. Each property binds a value pool of the kind listed, and an unbound property renders at its default.

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

`main.ext` is a reserved namespace for user-defined extensions, conventionally keyed by vendor. The core format assigns it no meaning and makes no compatibility guarantees about its contents; consumers ignore extensions they do not recognize. Its contents are arbitrary JSON, `null` included, though `ext` itself is never `null`. For example, Voxel Max's camera:

```jsonc
{
  "ext": {
    "vmax": {
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
3. An unrecognized value pool `kind` must be rejected: its values cannot be safely decoded, exactly as an unknown `encoding`'s data cannot, and it must never be reinterpreted or downgraded. `kind` is required and has no default.
4. Unknown property **names** in `properties` are ignored: properties are advisory and convention-based, so one tool's names pass through another's reader untouched.
5. Ignore vs reject: unknown property names are ignored; unknown `kind`, `encoding`, and `version`, and unknown object keys in any core structure, are rejected. Each is a contract a consumer must understand to make its guarantees.

## Validation

Validation is a hard contract, not best-effort. A validator rejects any file that violates a rule below; it never repairs, coerces, or fills in bad or missing input. Every field is required except the two on the optional allowlist: `main.editState`, whose absence means no editor margin, and `main.ext`, whose absence means no extensions. The closed vocabularies `version`, `encoding`, and value pool `kind` reject any unrecognized value and are never reinterpreted or downgraded.

### Rules

1. `version` is recognized.
2. Every `encoding`, on both `voxelPositions` and `voxelSamples`, is recognized.
3. Types are exact and nothing is coerced. A string where a number is expected, or the reverse, rejects. A number where an integer is expected is spelled as an integer. Every number is finite, so `NaN` and `+/-Infinity` reject; a float value writes the infinities as the sentinel strings `"inf"` and `"-inf"` (see [Value Pool Kinds](#value-pool-kinds)).
4. `null` rejects everywhere except in a `json` value pool's `values` and inside `main.ext`.
5. Unknown keys reject in every closed structure: file, `main`, `runtimeState`, `editState`, object, encoding block, palette, property, value pool, transform, hierarchy node, and edit object. The only open points are `main.ext` and property names. Keys are unique in every object: a repeated key rejects rather than resolving last-wins, `main.ext` and `json` pool values included.
6. All indices are in range:
   1. each object `layers` entry indexes `runtimeState.palettes`.
   2. each property `valuePool` indexes `runtimeState.valuePools`.
   3. each `childNodes` entry indexes `runtimeState.nodes`.
   4. each `childObjects` entry indexes `runtimeState.objects`.
   5. each `rootNodes` entry indexes `runtimeState.nodes`.
7. References are unique: no hierarchy node lists the same child node or the same child object twice, and no node appears in `rootNodes` twice.
8. **Objects**, per object:
   1. `layers` is present, an array of integers, possibly empty.
   2. `voxelPositions` and `voxelSamples` are present; the Positions and Samples rules check their structure.
9. **Value pools** (`runtimeState.valuePools`): an array, possibly empty. Each value pool has exactly the keys `kind` and `values`.
   1. `kind` is present and recognized.
   2. `values` is present, an array, possibly empty, with no `null` entry unless `kind` is `json`. Every entry is the shape `kind` declares (see [Value Pool Kinds](#value-pool-kinds)):
      1. `bool`: a JSON boolean.
      2. `float`: a float value.
      3. `int`: an int value.
      4. `json`: any JSON value, including `null`.
      5. `string`: a JSON string.
      6. `vec-2-float`, `vec-3-float`, `vec-4-float`: an array of exactly 2, 3, or 4 float values.
      7. `vec-2-int`, `vec-3-int`, `vec-4-int`: an array of exactly 2, 3, or 4 int values.
10. **Palettes** (`runtimeState.palettes`): an array, possibly empty. Each palette's keys are drawn only from { `properties`, `materials` }.
    1. `properties` is an array, possibly empty; each property has exactly the keys `name`, a non-empty string, and `valuePool`, an integer.
    2. no two properties share a `name`.
    3. `materials` is an array, possibly empty, of `M` rows, the material count; every row is an array of exactly `properties.length` integers, one value-index per property in property order.
    4. every `materials[m][b]` is an integer in `[0, valuePools[properties[b].valuePool].values.length)`.
11. **Samples**: let `V` be the voxel count from the position block. `voxelSamples.data` has exactly `layers.length` channels, one per layer in `layers` order. For channel `c`, let `M` be the material count of palette `layers[c]`, and by encoding:
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

  // palette indices, back to front, one `voxelSamples` channel per layer;
  // later layers override earlier ones (see Objects)
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
  // One channel per layer (in `layers` order): that layer's material index
  // for every voxel, in voxel order.
  | { encoding: "raw-json"; data: number[][] }
  // One channel per layer: a flat run stream
  // `[value1, count1, value2, count2, ...]`.
  | { encoding: "rle-json"; data: number[][] }
  // One channel per layer: each voxel's material index bit-packed at width
  // `b = max(1, bitLength(max(1, M) - 1))` for that layer's palette material
  // count `M`, MSB-first, base64-encoded (same packing as the
  // `bitmap-base64` position encoding).
  | { encoding: "packed-base64"; data: string[] };

// ## Palettes

// A palette binds property names to value pools, then lists its materials:
// one row per material, a value-index per property in property order, so
// the material count `M` is `materials.length`. A voxel
// samples material `m`; property `properties[b].name` takes
// `valuePools[properties[b].valuePool].values[materials[m][b]]`.
interface Palette {
  properties: Property[];

  materials: number[][];
}

// One property bound to a whole value pool, one value-index per material.
interface Property {
  // property name (see Properties); advisory, unknown names ignored
  name: string;

  // index into `RuntimeState.valuePools`
  valuePool: number;
}

// A shared value pool: `values` all of the one value-shape `kind` names.

// A float value: a finite number, "inf", or "-inf"; NaN has no sentinel.
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
// `bits` bits per axis.
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
// `deltaDecode` is the prefix sum; `deltaEncode` assumes ascending input, which
// keeps every delta after the first strictly positive.
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
// Uses arithmetic (not `<<` / `>>`) so values above `2^31` stay exact.
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
