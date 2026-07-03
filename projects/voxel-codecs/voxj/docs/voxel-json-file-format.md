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
2. `.voxjz` is the canonical shipping form. It is a standard zip archive holding exactly one `.voxj` member, conventionally named `main.voxj`, stored deflate-compressed. That deflate is the whole-file compression the encodings in [Voxel Encoding](#voxel-encoding) assume. The archive may carry additional resource members; consumers ignore members they do not recognize. Reading a `.voxjz` means opening the archive and parsing its `.voxj` member, which is byte-identical to a standalone `.voxj`. Tools that key off the `.zip` suffix may also accept `.voxjz.zip`.
3. Packaging is a transport concern and never changes the document. Recognize the form by leading bytes, not extension. An uncompressed file's first non-whitespace byte is `{`. A zip archive begins with `PK`, the bytes `0x50 0x4B`. Consumers accept either form wherever a voxel json file is expected.

## Structure

```jsonc
{
  "version": 1,

  "main": {
    "runtimeState": {
      "objects": [
        /* ... */
      ],

      "valuePools": [
        /* ... */
      ],

      "palettes": [
        /* ... */
      ],

      "hierarchyNodes": [
        /* ... */
      ],

      "rootHierarchyNodes": [
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

The runtime scene lives under `main.runtimeState`: objects, value pools, palettes, and hierarchy nodes are referenced by their array indices, and `rootHierarchyNodes` lists indices into `hierarchyNodes`. `editState` (optional) and `ext` are siblings of `runtimeState`. `editState` carries per-object editor margin aligned to the runtime objects (see [Edit State](#edit-state)); `ext` is an optional namespace for user-defined data that the core format ignores (see [Extensions](#extensions)).

## Coordinate System

The coordinate system is Z-up, right-handed. Voxel coordinates are unsigned integers and one unit = one voxel. A voxel at integer coordinate `(x, y, z)` occupies the unit cube whose minimum corner is that coordinate, spanning `[x, x + 1)` on each axis. An object carries no transform of its own beyond its grid `origin`; the hierarchy node that references it supplies rotation, scale, and placement. Seating the grid with `origin` near `-bounds / 2` makes the node's position the object's pivot, so rotating or scaling the node turns the object about its center rather than its min corner.

## Objects

An object is one voxel volume of pure geometry. Aside from its grid `origin`, it carries no transform of its own: rotation, scale, and placement come from the hierarchy node that references it.

```jsonc
{
  "name": "Object A",

  // [X, Y, Z] size in voxels
  "bounds": [1, 1, 1],

  // [X, Y, Z] translation from the placing node to the grid's min corner
  "origin": [0, 0, 0],
  "voxelPositions": { "encoding": "raw-json", "data": [[0, 0, 0]] },

  // palette references, one per layer
  "layerPaletteRefs": [0],
  // one channel per layer; each channel is one material index per voxel
  "voxelSamples": { "encoding": "raw-json", "data": [[0]] },
}
```

An object carries any number of layers, listed in `layerPaletteRefs` as palette indices. Each layer maps every voxel to one material in its palette. Two layers may reference the same palette; what the overlap means is left to the consuming application.

`voxelPositions` and `voxelSamples` are encoded blocks (see [Voxel Encoding](#voxel-encoding)). Each voxel has a position `(x, y, z)` and one material sample per layer. `voxelSamples` carries one channel per layer, in `layerPaletteRefs` order, and the sample in channel `c` is a material index into the palette `layerPaletteRefs[c]`. The number of voxels is implicit: it is the number of positions decoded from `voxelPositions`. Positions within an object must be unique, and every voxel samples every layer.

`bounds` is `[X, Y, Z]`, the runtime grid's size in voxels along each axis. Voxel positions are 0-based, so every voxel lies in `[0, X) x [0, Y) x [0, Z)`. `bounds` is exactly tight: the grid fits the voxels with no empty margin on any face, so on every axis some voxel reaches `0` and some voxel reaches the bound minus one. An empty object has `bounds = [0, 0, 0]`. Build-volume margin around the geometry is not allowed here; it lives in [`editState`](#edit-state), whose edit grid may be larger than the runtime grid. `bounds` is needed to decode `bitmap-base64` and `hilbert-delta-varint-base64`, where it sets the canonical voxel order and the Hilbert `bits = max(1, bitLength(max(X, Y, Z) - 1))` (see [Voxel Order](#voxel-order)).

`origin` is `[X, Y, Z]`, the translation in voxels from the placing hierarchy node to the grid's min corner; `[0, 0, 0]` puts the min corner at the node's local origin. It shifts where the grid sits relative to its node but does not change the voxel encodings, which stay 0-based within `bounds`. Setting it to about `-bounds / 2` centers the grid on the node, making the node's position the object's pivot so rotation and scale turn the object about its center.

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
// raw-json (listing order):
{ "encoding": "raw-json", "data": [[0, 0, 0], [1, 0, 0], [0, 1, 0], [1, 1, 0]] }

// bitmap-base64: cells in k-order (0, 0, 0), (0, 1, 0), (1, 0, 0), (1, 1, 0)
// are all occupied -> bits 1111 + 4 zero pad -> byte 0xF0.
{ "encoding": "bitmap-base64", "data": "8A==" }

// hilbert-delta-varint-base64:
//
// bits = 1
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

A sample block holds one channel per layer, in `layerPaletteRefs` order. Each channel gives, for every voxel in the position block's voxel order, a material index into that layer's palette.

1. `raw-json`: one channel per layer, each a plain array of that layer's material index for every voxel: `[[l0v0, l0v1, ...], [l1v0, l1v1, ...], ...]`.
2. `rle-json`: one channel per layer; each channel is a flat run-length encoding `[value1, count1, value2, count2, ...]`. Counts are positive integers and, in every channel, sum to the number of voxels.
3. `packed-base64`: one bit-packed channel per layer. For the channel of a layer whose palette has `M` materials, each voxel's material index is packed at fixed width `b = max(1, bitLength(M - 1))` bits, MSB-first, 8 per byte, with the final byte zero-padded; the width is derived from `M` and not stored. `data` is one base64 string per layer, in `layerPaletteRefs` order, each encoding exactly `ceil(voxelCount * b / 8)` bytes. This is the same packing scheme as the `bitmap-base64` position encoding, which is its `b = 1` special case. An empty object has one `""` per layer. Best for incoherent or many-material objects, where `rle-json` would approach one run per voxel.

#### Example: two layers over four voxels; layer 0 material indices `0, 0, 0, 1` (palette `M = 2`) and layer 1 material indices `2, 2, 3, 3` (palette `M = 4`), in the position block's voxel order

```jsonc
// raw-json: one array per layer, a material index per voxel.
{ "encoding": "raw-json", "data": [[0, 0, 0, 1], [2, 2, 3, 3]] }

// rle-json: one flat [value, count, ...] run stream per layer.
{ "encoding": "rle-json", "data": [[0, 3, 1, 1], [2, 2, 3, 2]] }

// packed-base64: one packed channel per layer.
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

Value pools live in `main.runtimeState.valuePools`, a shared array referenced by index, siblings of `objects` and `palettes`. A value pool holds `values`, all of one value-shape given by `kind`. `kind` tags the shape of the values. Palettes reference pools by index (see [Palettes](#palettes)).

```jsonc
// a value pool: shared values of one shape
{
  // value-shape tag (see Value Pool Kinds)
  "kind": "srgba-hex",

  // plain JSON literals, each well-formed for `kind`, indexed by value-index
  "values": ["#FF0000FF", "#00FF00FF", "#0000FFFF"],
}

// int, float, and vector color pools require min/max; each a number or "none"
{ "kind": "float", "min": 0, "max": 1, "values": [0, 0.5, 1] }
{ "kind": "float", "min": 1, "max": "none", "values": [1.5, 1.33] } // >= 1
{ "kind": "int", "min": 0, "max": 255, "values": [0, 128, 255] }
{ "kind": "srgba-int", "min": 0, "max": 255, "values": [[255, 0, 0, 255]] } // 8-bit
{ "kind": "srgb-float", "min": 0, "max": "none", "values": [[2, 0, 0]] } // HDR
```

### Value Pool Kinds

`kind` is a closed vocabulary tagging the shape of a pool's `values`. Every kind's `values` are plain readable JSON literals; declaring a kind enables validation: a consumer must understand a file's kinds to validate it, and must reject a file whose `kind` it does not recognize (see [Versioning and Extensibility](#versioning-and-extensibility)).

| `kind`              | JSON form            | Constraint                                           | Example `values`               | Typical attributes                                                                            |
| ------------------- | -------------------- | ---------------------------------------------------- | ------------------------------ | --------------------------------------------------------------------------------------------- |
| `json`              | any JSON, incl. null | none                                                 | `[{"k": 1}, "x", 3]`           | any custom attribute                                                                          |
| `bool`              | boolean              | `true` / `false`                                     | `[true, false]`                | flags                                                                                         |
| `float`             | number               | floating-point-valued; within `min`/`max`            | `[0, 0.5, 1]`                  | metallicFactor, roughnessFactor, occlusionStrength, transmissionFactor, emissiveStrength, ior |
| `int`               | number               | integer-valued, within `min`/`max`                   | `[0, 1, 2, 7]`                 | ids, counts, indices                                                                          |
| `string`            | string               | must be a string                                     | `["low", "high"]`              | enumerated tags                                                                               |
| `srgb-float`        | number[3]            | 3 finite numbers; within `min`/`max`, sRGB           | `[[1, 0, 0], [0.5, 0.5, 0]]`   | emissiveFactor, sRGB float / HDR                                                              |
| `srgb-hex`          | string               | matches `^#[0-9A-F]{6}$`                             | `["#FF0000", "#204080"]`       | emissiveFactor; opaque custom colors                                                          |
| `srgb-int`          | number[3]            | 3 integer-valued numbers; within `min`/`max`, sRGB   | `[[255, 0, 0], [128, 128, 0]]` | emissiveFactor, sRGB integer                                                                  |
| `srgba-float`       | number[4]            | 4 finite numbers; within `min`/`max`, sRGB           | `[[1, 0, 0, 1]]`               | baseColorFactor, sRGB float                                                                   |
| `srgba-hex`         | string               | matches `^#[0-9A-F]{8}$`                             | `["#FF0000FF"]`                | baseColorFactor, the default color kind                                                       |
| `srgba-int`         | number[4]            | 4 integer-valued numbers; within `min`/`max`, sRGB   | `[[255, 0, 0, 255]]`           | baseColorFactor, sRGB integer                                                                 |
| `linear-rgb-float`  | number[3]            | 3 finite numbers; within `min`/`max`, linear         | `[[1, 0, 0], [0, 0.5, 1]]`     | emissiveFactor, linear                                                                        |
| `linear-rgb-int`    | number[3]            | 3 integer-valued numbers; within `min`/`max`, linear | `[[255, 0, 0], [0, 128, 255]]` | emissiveFactor, integer linear                                                                |
| `linear-rgba-float` | number[4]            | 4 finite numbers; within `min`/`max`, linear         | `[[1, 0, 0, 1]]`               | baseColorFactor, linear                                                                       |
| `linear-rgba-int`   | number[4]            | 4 integer-valued numbers; within `min`/`max`, linear | `[[255, 0, 0, 255]]`           | baseColorFactor, integer linear                                                               |

Notes:

1. `float` is a continuous, finite number; `int` is its integer-valued sibling. `min` and `max` bounds apply only to `int`, `float`, and the eight vector color kinds (`-int` and `-float` components); on those kinds both are required, and no other kind may carry them.
2. `min` and `max` each take a finite number or the string `"none"` for unbounded on that side, and both are always written out. A numeric bound must be integer-valued on `int` and the `-int` color kinds; on the vector color kinds a bound applies per component; when both bounds are finite numbers, `min <= max`.
3. Colors come in hex, integer, and float forms across two color spaces. `srgb-hex` / `srgba-hex` are `#RRGGBB` / `#RRGGBBAA` sRGB strings, the human-editable default. `srgb-int` / `srgba-int` / `srgb-float` / `srgba-float` are sRGB integer or float components; `linear-rgb-int` / `linear-rgba-int` / `linear-rgb-float` / `linear-rgba-float` are the linear components.
4. `kind` is required and has no default; the bounded kinds require both `min` and `max`. A value pool has no optional fields.

## Palettes

A palette binds attribute names to shared [value pools](#value-pools), then lists the distinct materials it uses as rows over those pools. A voxel samples a material in each layer by its index in that layer's palette. A palette may be referenced by any number of layers and objects (see [Objects](#objects)).

Materials are stored column-major, one column per binding:

```jsonc
{
  // ordered bindings; each binds an attribute name to a value pool index. Order
  // fixes the column order in `materials`. No duplicate attribute.
  "bindings": [
    { "attribute": "baseColorFactor", "poolRef": 0 },
    { "attribute": "metallicFactor", "poolRef": 1 },
  ],

  // `materials` is column-major: one inner array per binding (a column), in
  // binding order, so materials.length == bindings.length. Every column has the
  // same length, the material count M. materials[b][m] is a value-index into the
  // pool bound by column b. A voxel samples material m in [0, M); resolve it by
  // reading down the columns:
  //   material 0 = { baseColorFactor: pool0.values[0], metallicFactor: pool1.values[2] }
  "materials": [
    [0, 1, 2], // baseColorFactor value-index for materials 0, 1, 2
    [2, 0, 1], // metallicFactor value-index for materials 0, 1, 2
  ],
}
```

`materials` is column-major: each inner array is one binding's column of value-indices into a single pool.

To resolve a voxel's material:

1. Read the voxel's sample `m`, a material index.
2. For each binding `b`, read `materials[b][m]`, a value-index into the pool `bindings[b].poolRef`.
3. The attribute `bindings[b].attribute` takes `valuePools[bindings[b].poolRef].values[materials[b][m]]`.
4. Unbound attributes take their default from the [Attributes](#attributes) table.

For the palette above, a voxel sampling material `0` resolves to `baseColorFactor = valuePools[0].values[0]` and `metallicFactor = valuePools[1].values[2]`.

### Attributes

An attribute is a named material property, listed in `palette.bindings[].attribute`. The format wires attributes without defining them: the name carries the meaning and the pool carries the values; that pairing is all the format defines. An attribute's meaning and value range are convention between producer and consumer, not part of the wire format. A consumer ignores any attribute name it does not recognize, so extending the vocabulary never breaks an older reader (see [Versioning and Extensibility](#versioning-and-extensibility)).

voxj's recommended vocabulary is glTF's, below. The format neither requires nor privileges it; it is the convention voxj tools target.

#### glTF conventions

The recommended attribute vocabulary is glTF's metallic-roughness model, so a voxj material maps one-to-one onto a glTF material and the defaults below are glTF's own. Each attribute binds a value pool of one of the kinds listed, and an unbound attribute renders at its default:

| Attribute            | Kind                                                                                         | Range | Default     | Meaning                                                                            |
| -------------------- | -------------------------------------------------------------------------------------------- | ----- | ----------- | ---------------------------------------------------------------------------------- |
| `baseColorFactor`    | `srgba-hex` (default), `srgba-int`, `srgba-float`, `linear-rgba-int`, or `linear-rgba-float` |       | `#FFFFFFFF` | Base color, straight alpha = opacity (glTF `baseColorFactor`)                      |
| `metallicFactor`     | `float`, `min: 0`, `max: 1`                                                                  | 0-1   | `1`         | Metalness (glTF `metallicFactor`)                                                  |
| `roughnessFactor`    | `float`, `min: 0`, `max: 1`                                                                  | 0-1   | `1`         | Roughness (glTF `roughnessFactor`)                                                 |
| `occlusionStrength`  | `float`, `min: 0`, `max: 1`                                                                  | 0-1   | `1`         | Flat ambient occlusion, 1 = none (glTF `occlusionTexture.strength`)                |
| `emissiveFactor`     | `srgb-hex` (default), `srgb-int`, `srgb-float`, `linear-rgb-int`, or `linear-rgb-float`      |       | `#000000`   | Emissive color, black = none (glTF `emissiveFactor`)                               |
| `emissiveStrength`   | `float`, `min: 0`                                                                            | 0+    | `1`         | Multiplies emissive color in linear space (glTF `KHR_materials_emissive_strength`) |
| `ior`                | `float`, `min: 1`                                                                            | 1+    | `1.5`       | Index of refraction (glTF `KHR_materials_ior`)                                     |
| `transmissionFactor` | `float`, `min: 0`, `max: 1`                                                                  | 0-1   | `0`         | Light transmission through surface (glTF `KHR_materials_transmission`)             |

A color attribute binds a hex, integer-component, or float-component color kind in either the sRGB or linear space (see [Value Pool Kinds](#value-pool-kinds)); hex is the authoring default. Base color takes an alpha-carrying kind and emission an alpha-less one, and all forms carry the same color.

Emission is two attributes. `emissiveFactor` is the emitted color, `srgb-hex` by default or a vector form, no alpha, default `#000000` for no emission, authored and linearized like `baseColorFactor`. `emissiveStrength` is a numeric multiplier over that color, `float` with `min: 0`, default `1`; values above `1` push emission into HDR/bloom range. Rendered emission is `linearize(emissiveFactor) * emissiveStrength`. The defaults compose: a black color emits nothing at any strength, and a color left at the default strength `1` emits at face value, so a material that sets only `emissiveFactor` emits that color at strength 1.

## Hierarchy Nodes

Nodes form a DAG: a node may have multiple parents but no cycles. Each references child nodes and child objects by index and carries a transform.

```jsonc
{
  "name": "parent-1",
  "childNodes": [1],
  "childObjects": [0],
  "transform": {
    "position": [0, 0, 0],
    "rotation": [0, 0, 0, 1],
    "scale": [1, 1, 1],
  },
}
```

A transform has three fields: `position` is a possibly-fractional `[x, y, z]`, `rotation` is a unit quaternion `[x, y, z, w]`, and `scale` is `[x, y, z]`.

1. A transform composes as `Translation * Rotation * Scale`.
2. A node's world transform is `parentWorld * nodeLocal`; a root, listed in `rootHierarchyNodes`, has world = local. Reached through multiple parents, a node is placed once per path; this is instancing.
3. An object is placed at the world transform of the node referencing it. A voxel at grid position `p` sits at node-local position `origin + p`, so its world position is the node's world transform applied to `origin + p`.
4. `rotation` must be a unit quaternion; consumers may renormalize within a small tolerance (see [Validation](#validation)).
5. `scale` is per-axis; a negative component mirrors that axis and flips winding/handedness. A zero component is degenerate and invalid (see [Validation](#validation)).

The scene's roots are exactly the nodes listed in `rootHierarchyNodes`. A node that is neither listed as a root nor referenced as a child is unplaced and does not render, so a file may hold library nodes that are defined without being placed. The format describes placement only; how overlapping or sub-voxel placements are resolved, merged, or rasterized is consumer-defined.

## Edit State

`main.editState` is optional editor state, kept separate from the runtime scene. When present, its `objects` array has one entry per runtime object, aligned by index, giving the author's edit grid (the build volume) for that object.

```jsonc
{
  "objects": [
    {
      // [X, Y, Z] size of the edit grid in voxels
      "bounds": [6, 6, 6],

      // [X, Y, Z] translation from the placing node to the edit grid's min corner
      "origin": [-1, -1, -1],
    },
  ],
}
```

Each edit grid must contain its object's runtime grid: on every axis the edit `origin` is `<=` the runtime `origin`, and the edit `origin + bounds` is `>=` the runtime `origin + bounds`. An edit grid equal to the runtime grid carries no margin. `editState` is omitted when no object has a distinct edit grid; a consumer that ignores it loses only editor margin, never geometry.

## Extensions

`main.ext` is a reserved namespace for user-defined extensions, conventionally keyed by vendor. The core format assigns it no meaning and makes no compatibility guarantees about its contents; consumers ignore extensions they do not recognize. For example, Voxel Max's camera:

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
4. Unknown **attribute** names in bindings are ignored, since attributes are advisory and convention-based, so adding attributes is backward compatible.
5. Ignore vs reject: unknown attribute names are ignored; unknown `kind`, `encoding`, and `version`, and unknown object keys in any core structure, are rejected. Each is a contract a consumer must understand to make its guarantees.

## Validation

Validation is a hard contract, not best-effort. A validator rejects any file that violates a rule below; it never repairs, coerces, or fills in bad or missing input. Every field is required except the two on the optional allowlist: `main.editState`, whose absence means no editor margin, and `main.ext`, whose absence means no extensions. The closed vocabularies `version`, `encoding`, and value pool `kind` reject any unrecognized value and are never reinterpreted or downgraded.

### Rules

1. `version` is recognized.
2. Every `encoding`, on both `voxelPositions` and `voxelSamples`, is recognized.
3. Types are exact and nothing is coerced. A string where a number is expected, or the reverse, rejects. Every integer-valued number has no fractional part, and every number is finite, so `NaN` and `+/-Infinity` reject.
4. `null` rejects everywhere except in a `json`-kind pool's `values`: in every structural field, every non-`json` pool's `values`, and every block's `data`.
5. Unknown keys reject in every closed structure: file, `main`, `runtimeState`, `editState`, object, encoding block, palette, binding, value pool, transform, hierarchy node, and edit object. The only open points are `main.ext` and binding attribute names.
6. All indices are in range:
    1. each object `layerPaletteRefs` entry indexes `runtimeState.palettes`.
    2. each binding `poolRef` indexes `runtimeState.valuePools`.
    3. each `childNodes` entry indexes `runtimeState.hierarchyNodes`.
    4. each `childObjects` entry indexes `runtimeState.objects`.
    5. each `rootHierarchyNodes` entry indexes `runtimeState.hierarchyNodes`.
7. References are unique: no hierarchy node lists the same child node or the same child object twice, and no node appears in `rootHierarchyNodes` twice.
8. **Objects**, per object:
    1. `layerPaletteRefs` is present, an array of integers, possibly empty.
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
        8. `srgb-int` / `linear-rgb-int`: an array of exactly 3 integer-valued numbers, each within `min`/`max`.
        9. `srgba-int` / `linear-rgba-int`: an array of exactly 4 integer-valued numbers, each within `min`/`max`.
        10. `srgb-float` / `linear-rgb-float`: an array of exactly 3 finite numbers, each within `min`/`max`.
        11. `srgba-float` / `linear-rgba-float`: an array of exactly 4 finite numbers, each within `min`/`max`.
    3. `min` and `max`:
        1. both present when `kind` is `int`, `float`, or one of the eight vector color kinds, and both absent for every other kind.
        2. each is a finite number or the string `none`, meaning unbounded on that side.
        3. a numeric bound is integer-valued when `kind` is `int` or an `-int` color kind.
        4. `min <= max` when both are finite numbers.
10. **Palettes** (`runtimeState.palettes`): an array, possibly empty. Each palette's keys are drawn only from { `bindings`, `materials` }.
    1. `bindings` is a non-empty array; each binding has exactly the keys `attribute`, a non-empty string, and `poolRef`, an integer.
    2. no two bindings share an `attribute`.
    3. `materials` has exactly `bindings.length` columns, one per binding in binding order.
    4. every column is an array of the same length `M >= 1`, the material count.
    5. every `materials[b][m]` is an integer in `[0, valuePools[bindings[b].poolRef].values.length)`.
11. **Samples**: let `V` be the voxel count from the position block. `voxelSamples.data` has exactly `layerPaletteRefs.length` channels, one per layer in `layerPaletteRefs` order. For channel `c`, let `M` be the material count of palette `layerPaletteRefs[c]`, and by encoding:
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
16. `bounds` is three non-negative integers, exactly tight around the decoded positions: on each axis the minimum voxel coordinate is `0` and `bounds` is the maximum plus one. An empty object has `bounds = [0, 0, 0]`.
17. The hierarchy is acyclic.
18. No transform `scale` component is zero.
19. Every transform `rotation` has length-squared within `1e-6` of `1`; consumers may renormalize within this tolerance.
20. When `editState` is present, its `objects` has exactly one entry per runtime object. Each edit object's `bounds` is three non-negative integers and its `origin` is three integers, and the edit grid contains the runtime grid: on every axis edit `origin` is `<=` runtime `origin`, and edit `origin + bounds` is `>=` runtime `origin + bounds`.

## Examples

### File Example

```jsonc
{
  "version": 1,
  "main": {
    "runtimeState": {
      "objects": [
        {
          "name": "Object A",

          // two layers: palette 0, then palette 1. Layers do not merge; the app
          // decides what two baseColorFactor layers mean.
          "layerPaletteRefs": [0, 1],

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

          // one channel per layer, each a material index per voxel:
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
          // single layer
          "layerPaletteRefs": [1],
          "bounds": [1, 1, 1],
          "origin": [0, 0, 0],
          "voxelPositions": { "encoding": "raw-json", "data": [[0, 0, 0]] },
          "voxelSamples": { "encoding": "raw-json", "data": [[0]] },
        },
      ],
      "palettes": [
        // value pool 1 is bound twice here, to metallicFactor and roughnessFactor
        {
          "bindings": [
            { "attribute": "baseColorFactor", "poolRef": 0 },
            { "attribute": "metallicFactor", "poolRef": 1 },
            { "attribute": "roughnessFactor", "poolRef": 1 },
            { "attribute": "emissiveFactor", "poolRef": 2 },
          ],

          // column-major, one column per binding. Material 2 resolves to
          // baseColorFactor #0000FFFF, metallicFactor 0.5, roughnessFactor 0,
          // emissiveFactor #FF6600.
          "materials": [
            [0, 1, 2],
            [2, 0, 1],
            [1, 1, 0],
            [0, 0, 1],
          ],
        },

        // base color authored in linear form instead of hex
        {
          "bindings": [{ "attribute": "baseColorFactor", "poolRef": 3 }],
          "materials": [[0]],
        },
      ],
      "valuePools": [
        {
          "kind": "srgba-hex",
          "values": ["#FF0000FF", "#00FF00FF", "#0000FFFF"],
        },

        // one shared float pool, bound by both metallicFactor and roughnessFactor
        { "kind": "float", "min": 0, "max": 1, "values": [0, 0.5, 1] },

        { "kind": "srgb-hex", "values": ["#000000", "#FF6600"] },

        {
          "kind": "linear-rgba-float",
          "min": 0,
          "max": 1,
          "values": [[1, 0, 0, 1]],
        },
      ],

      "hierarchyNodes": [
        {
          "name": "parent-1",

          "childNodes": [1],

          "childObjects": [0],

          "transform": {
            "position": [0, 0, 0],
            "rotation": [0, 0, 0, 1],
            "scale": [1, 1, 1],
          },
        },

        {
          "name": "parent-2",

          "childNodes": [],
          "childObjects": [1],

          "transform": {
            "position": [0, 0, 0],
            "rotation": [0, 0, 0, 1],
            "scale": [1, 1, 1],
          },
        },
      ],

      "rootHierarchyNodes": [0],
    },
  },
}
```

### TypeScript Schema

```typescript
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
  objects: VoxelObject[];

  palettes: Palette[];

  // shared value pools, referenced by index from palette bindings
  valuePools: ValuePool[];

  hierarchyNodes: HierarchyNode[];

  // indices into hierarchyNodes; the scene's roots
  rootHierarchyNodes: number[];
}

// Optional editor state: one edit grid per runtime object, aligned by index.
interface EditState {
  objects: EditObject[];
}

// One object's edit grid (build volume), which must contain its runtime grid.
interface EditObject {
  // [X, Y, Z] size of the edit grid in voxels
  bounds: Vec3;

  // [X, Y, Z] translation from the placing node to the edit grid's min corner
  origin: Vec3;
}

// Pure geometry; placed only by a hierarchy node that references it.
interface VoxelObject {
  name: string;

  // palette indices, one per layer (see Objects). Layers are independent
  // material channels; the format defines no merge across them.
  layerPaletteRefs: number[];

  // [X, Y, Z] size in voxels; voxels occupy [0, X) x [0, Y) x [0, Z). Exactly
  // tight: per-axis the min voxel coordinate is 0 and the bound is the max plus
  // one; [0, 0, 0] when empty. No margin here (that is editState). Required to
  // decode bitmap-base64 and hilbert-delta-varint-base64.
  bounds: Vec3;

  // [X, Y, Z] translation from the placing node to the grid's min corner. Does
  // not affect the voxel encodings.
  origin: Vec3;

  voxelPositions: PositionBlock;

  voxelSamples: SampleBlock;
}

// ## Voxel Encoding

// Both blocks share one voxel order, fixed by the position encoding (see Voxel
// Order); every sample channel is in that order. The match is an authoring
// invariant that validation cannot verify.

type PositionBlock =
  // One [x, y, z] per voxel, in listing order.
  | { encoding: "raw-json"; data: Vec3[] }
  // Dense occupancy bitmap over `bounds` (required to decode): one bit per cell
  // k = x * Y * Z + y * Z + z, packed 8 per byte MSB-first, base64-encoded.
  // Canonical order is ascending k.
  | { encoding: "bitmap-base64"; data: string }
  // Prefix-sum deltas of each voxel's 3D Hilbert-curve index (see
  // hilbertEncode/hilbertDecode), voxels sorted by ascending index; deltas as
  // an unsigned-LEB128 varint stream, base64-encoded. Requires bits <= 17
  // (every bounds dimension <= 131072).
  | { encoding: "hilbert-delta-varint-base64"; data: string };

type SampleBlock =
  // One channel per layer (in `layerPaletteRefs` order): that layer's material index for
  // every voxel, in voxel order.
  | { encoding: "raw-json"; data: number[][] }
  // One channel per layer: a flat run stream [value1, count1, value2, count2, ...].
  | { encoding: "rle-json"; data: number[][] }
  // One channel per layer: each voxel's material index bit-packed at width
  // b = max(1, bitLength(M - 1)) for that layer's palette material count M,
  // MSB-first, base64-encoded (same packing as the bitmap-base64 position encoding).
  | { encoding: "packed-base64"; data: string[] };

// ## Palettes

// A palette binds attribute names to value pools, then stores its materials
// column-major: one inner array per binding, in binding order, each of length
// M, the material count. A voxel samples material m; attribute
// bindings[b].attribute takes
// valuePools[bindings[b].poolRef].values[materials[b][m]].
interface Palette {
  bindings: Binding[];

  materials: number[][];
}

// One attribute-to-pool binding; fixes one column of materials.
interface Binding {
  // attribute name (see Attributes); advisory, unknown names ignored
  attribute: string;

  // index into RuntimeState.valuePools
  poolRef: number;
}

// A shared pool of values, all of one shape given by kind. The bounded kinds
// (int, float, and the vector color kinds) require both min and max; every other
// kind carries neither. Each bound is a finite number or "none" for unbounded on
// that side.
type ValuePool =
  | {
      kind: BoundedKind;

      min: number | "none";

      max: number | "none";

      values: JsonValue[];
    }
  | {
      kind: Exclude<PoolKind, BoundedKind>;
      values: JsonValue[]
    };

// Closed value-shape vocabulary (see Value Pool Kinds).
type PoolKind =
  | "json"
  | "bool"
  | "float"
  | "int"
  | "string"
  | "srgb-float"
  | "srgb-hex"
  | "srgb-int"
  | "srgba-float"
  | "srgba-hex"
  | "srgba-int"
  | "linear-rgb-float"
  | "linear-rgb-int"
  | "linear-rgba-float"
  | "linear-rgba-int";

// The kinds that carry min/max bounds; see ValuePool.
type BoundedKind =
  | "float"
  | "int"
  | "srgb-float"
  | "srgb-int"
  | "srgba-float"
  | "srgba-int"
  | "linear-rgb-float"
  | "linear-rgb-int"
  | "linear-rgba-float"
  | "linear-rgba-int";

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

  // indices into Main.hierarchyNodes (DAG, no cycles)
  childNodes: number[];

  // indices into Main.objects
  childObjects: number[];

  transform: Transform;
}

interface Transform {
  // [x, y, z]
  position: Vec3;

  // unit quaternion [x, y, z, w]
  rotation: Quat;

  // [x, y, z]
  scale: Vec3;
}

type Vec3 = [number, number, number];

type Quat = [number, number, number, number];
```

### Reference Code

Reference implementations of the binary encodings, as small independent codecs to port directly. `raw-json` positions and samples are plain JSON and need none. `base64` / `unbase64` are standard RFC 4648 (`btoa`/`atob` or `Buffer` in JS; see [Voxel Encoding](#voxel-encoding)). `Vec3` is `[number, number, number]`. Each block's `data` is one composition below.

#### Bit Widths

```ts
// Binary digits in a non-negative integer, bitLength(0) = 0. The width formulas
// call bitLength(x - 1): the bits to index x distinct values, integer-exact
// with no floating point. Never use Math.log2 for these.
function bitLength(n: number): number {
  let len = 0;
  while (n > 0) {
    n = Math.floor(n / 2);
    len++;
  }
  return len;
}

// Hilbert `bits` per axis from bounds, and packed-base64 channel width from a
// palette material count. Both are max(1, ceil(log2(.))) via bitLength.
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
// zero-padded. bitmap-base64 is the width = 1 case.
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

// Inverse of packBits. Bytes past the end read as zero.
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

// bitmap-base64: one occupancy bit per cell of `bounds`, packed at width 1. The
// canonical voxel order is ascending cell index, so reorder each sample channel
// to match (sort voxel indices by cellIndex for the same remap that
// encodeHilbertBlockWithRemap returns).
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

// packed-base64: one layer's channel, each voxel's material index packed at
// packedWidth(materialCount). `samples` is in the position block's voxel order.
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
// rle-json: one layer's channel as a flat [value, count, ...] run stream; counts
// are positive and sum to the voxel count.
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
// base64(varintEncode(deltaEncode(sortedHilbertIndices))).
//
// NOTE: indices are assembled with arithmetic, not `<<`, because JS bitwise
// operators are 32-bit and an index can exceed 31 bits on large grids. The
// index is exact in a JS `number` only while 3 * bits <= 53, i.e. bits <= 17;
// the format caps bits at 17 (every bounds dimension <= 131072) for this
// reason.

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

  // Interleave into a single index (axes[0] most significant).
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
//    deltaDecode is the prefix sum; deltaEncode assumes ascending input, which
//    keeps every delta after the first strictly positive.
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
//    Uses arithmetic (not `<<` / `>>`) so values above 2^31 stay exact.
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

// Compose. base64 / unbase64 are standard (btoa + atob in the browser, Buffer
// in Node). Voxels are sorted by ascending Hilbert index so the deltas stay
// positive. `bits` is hilbertBits(object.bounds); packed-base64 uses
// packedWidth(materialCount).
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

// Like encodeHilbertBlock, but also returns `remap`, the permutation from input
// order to the block's canonical order: remap[oldIndex] = newIndex. Reorder each
// sample array `s` to match with `out[remap[i]] = s[i]`.
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

  // order[newIndex] = oldIndex, sorted by ascending Hilbert index.
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
