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
    "objects": [
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
    "ext": {
      /* ... */
    },
  },
}
```

Objects, palettes, and hierarchy nodes are referenced by their array index, and `rootHierarchyNodes` lists indices into `hierarchyNodes`. `ext` is an optional namespace for user-defined data that the core format ignores (see [Extensions](#extensions)).

## Coordinate System

The coordinate system is Z-up, right-handed. Voxel coordinates are unsigned integers and one unit = one voxel. A voxel at integer coordinate `(x, y, z)` occupies the unit cube whose minimum corner is that coordinate, i.e., spanning `[x, x + 1)` on each axis. Enclosing hierarchy nodes must be used to recenter or scale objects about a pivot.

## Objects

An object is one voxel volume. It is pure geometry with no transform: it is placed only by a hierarchy node that references it.

```jsonc
{
  "name": "Object A",

  // [X, Y, Z] size in voxels
  "bounds": [1, 1, 1],
  "voxelPositions": { "encoding": "raw-json", "data": [[0, 0, 0]] },

  // palette indices, in resolution order
  "paletteRefs": [0],
  "voxelSamples": { "encoding": "raw-json", "data": [[0]] },
}
```

`voxelPositions` and `voxelSamples` are encoded blocks (see [Voxel Encoding](#voxel-encoding)). Each voxel has a position `(x, y, z)` and one sample per referenced palette. The sample for palette `n` is a cell index into the object's `n`-th palette. The number of voxels is implicit: it is the number of positions decoded from `voxelPositions`. Positions within an object must be unique, and every voxel samples every palette.

`bounds` is `[X, Y, Z]`, the object's size in voxels along each axis. Objects are authored from the origin, so every voxel position lies in `[0, X) x [0, Y) x [0, Z)`. `bounds` must contain the object but need not fit it tightly: each component is at least the maximum coordinate on that axis plus one, and may be larger to leave empty margin around the geometry (see [Validation](#validation)). `bounds` is required to decode `bitmap-base64` and `hilbert_index-delta-varint-base64`, so margin is not always free under those encodings: for `bitmap-base64`, changing `bounds` changes the canonical voxel order and forces re-encoding; for `hilbert_index-delta-varint-base64`, `bounds` enters only through `bits = max(1, bitLength(max(X, Y, Z) - 1))`, so margin that does not change `bits` is free but margin that does forces re-encoding (see [Voxel Order](#voxel-order)).

The position block fixes a single voxel order for the object, and the sample channels follow it voxel-for-voxel.

## Voxel Encoding

Each block is `{ "encoding", "data" }`. New `encoding` values may be added in future versions (see [Versioning](#versioning-and-extensibility)).

All base64 in this format uses the standard RFC 4648 alphabet, not base64url, with `=` padding and no line breaks.

### Position Encodings

1. `raw-json`: one `[x, y, z]` triple per voxel, in listing order: `[[x0, y0, z0], [x1, y1, z1], ...]`. An empty object has `data = []`.
2. `bitmap-base64`: a dense occupancy bitmap. `data` is a standard base64 string with no line breaks. It encodes one occupancy bit per cell of the object's `bounds = [X, Y, Z]`, so positions are implicit and this encoding requires `bounds` to decode. The cell index is `k = x * Y * Z + y * Z + z`, iterating x outermost and z innermost over `0 <= x < X`, `0 <= y < Y`, `0 <= z < Z` with `X * Y * Z` cells total. Bit `k` is `1` if cell `k` is occupied. Bits are packed 8 per byte, MSB-first: cell `k` is bit `(7 - (k mod 8))` of byte `floor(k / 8)`. The last byte is zero-padded when `X * Y * Z` is not a multiple of 8; pad bits must be `0`. The base64 encodes exactly `ceil(X * Y * Z / 8)` bytes. The number of voxels is the number of set bits. An object with no set bits is empty; its `data` is `ceil(X * Y * Z / 8)` zero bytes for the given `bounds` (`""` when `bounds = [0, 0, 0]`). Best for dense objects, roughly >= 50% filled; valid at any density.
3. `hilbert_index-delta-varint-base64`: a Hilbert-index delta list. `data` is a standard base64 string with no line breaks, encoding the deltas as an unsigned LEB128 varint stream (see the reference code in [Hilbert Reference Code](#hilbert-reference-code)). Each position `(x, y, z)` maps to one Hilbert index via the standard 3D Hilbert curve with `bits = max(1, bitLength(max(X, Y, Z) - 1))` taken from `bounds`. Axes map to Hilbert dimensions `(x, y, z) = (0, 1, 2)`, and the curve covers a `2 ^ bits` cube containing the bounds. Voxels are sorted by ascending index, and the encoded deltas are `[h0, h1 - h0, h2 - h1, ...]`; decode by base64-decoding to the varint stream, reading the deltas, prefix-summing to recover the indices, then Hilbert-decoding each. Every delta after the first is strictly positive. An empty object has `data = ""`. A good general-purpose encoding; strongest from sparse up through moderate density, and compact at any density because each delta is a small varint rather than a full index.

   Because the reference algorithm assembles and decodes a Hilbert index in a JS `number` (a double, exact only to `2 ^ 53`), this encoding requires `bits <= 17`, equivalently every `bounds` dimension `<= 131072`; a validator must reject larger grids, which must instead use `bitmap-base64` or `raw-json`.

#### Example: a 2 x 2 x 1 square in the `z = 0` plane - voxels `(0, 0, 0)`, `(1, 0, 0)`, `(0, 1, 0)`, `(1, 1, 0)` with `bounds = [2, 2, 1]`

```jsonc
// raw-json (listing order):
{ "encoding": "raw-json", "data": [[0, 0, 0], [1, 0, 0], [0, 1, 0], [1, 1, 0]] }

// bitmap-base64: cells in k-order (0, 0, 0), (0, 1, 0), (1, 0, 0), (1, 1, 0)
// are all occupied -> bits 1111 + 4 zero pad -> byte 0xF0.
{ "encoding": "bitmap-base64", "data": "8A==" }

// hilbert_index-delta-varint-base64:
//
// bits = 1
//
// sorted Hilbert indices [0, 3, 4, 7] ->
// deltas [0, 3, 1, 3] ->
// varint bytes 00 03 01 03 ->
// base64 "AAMBAw==".
// Those indices decode (in order) to
// (0, 0, 0), (0, 1, 0), (1, 1, 0), (1, 0, 0). This is a different voxel order
// than the bitmap's raster order, so the two encodings need their sample
// channels in different orders.
{ "encoding": "hilbert_index-delta-varint-base64", "data": "AAMBAw==" }
```

### Sample Encodings

1. `raw-json`: one entry per voxel, each a plain array of that voxel's samples, one cell index per palette, in order: `[[v0p0, v0p1], [v1p0, v1p1], ...]`.
2. `rle-json`: one channel per palette; each channel is a flat run-length encoding `[value1, count1, value2, count2, ...]`. Counts are positive integers and, in every channel, sum to the number of voxels.
3. `packed-base64`: one bit-packed channel per palette. For the channel sampling a palette with `c` cells, each voxel's cell index is packed at fixed width `b = max(1, bitLength(c - 1))` bits, MSB-first, 8 per byte, with the final byte zero-padded; the width is derived from `c` and not stored. `data` is one base64 string per palette, in `paletteRefs` order, each encoding exactly `ceil(voxelCount * b / 8)` bytes. This is the same packing scheme as the `bitmap-base64` position encoding, which is its `b = 1` special case. An empty object has one `""` per palette. Best for incoherent or many-color objects, where `rle-json` would approach one run per voxel.

#### Example: four voxels in an object whose `paletteRefs` has length 2, with per-voxel samples in the position block's voxel order `[0, 0]`, `[0, 1]`, `[0, 1]`, `[1, 1]`

```jsonc
// raw-json: one [palette-0 cell, palette-1 cell] row per voxel
{ "encoding": "raw-json", "data": [[0, 0], [0, 1], [0, 1], [1, 1]] }

// rle-json: one channel per palette, each a flat [value, count, ...] run stream
{ "encoding": "rle-json", "data": [[0, 3, 1, 1], [0, 1, 1, 3]] }

// packed-base64: one packed channel per palette.
//
// Channel 0 =
// 0,0,0,1 ->
// byte 0b0001_0000 = 0x10 ->
// "EA=="
//
// channel 1 =
// 0,1,1,1 ->
// byte 0b0111_0000 = 0x70 ->
// "cA=="
{ "encoding": "packed-base64", "data": ["EA==", "cA=="] }
```

### Voxel Order

The position block defines the object's single canonical voxel order, and the sample channels are in that same order, voxel-for-voxel, for every combination of position and sample encodings:

1. `raw-json` positions: listing order.
2. `bitmap-base64` positions: ascending cell index `k` (raster order, z fastest).
3. `hilbert_index-delta-varint-base64` positions: ascending Hilbert index.

The same geometry generally orders differently under different position encodings, so re-encoding the position block changes the order and the sample channels must be regenerated to match.

### Choosing an Encoding

Position:

1. `bitmap-base64`: dense objects. Smallest geometry when filled, and the fastest to decode.
2. `hilbert_index-delta-varint-base64`: sparse objects, and any object whose color is spatially coherent that you want as small as possible. Hilbert order places neighboring voxels next to each other in the stream, which also lengthens the sample channel's runs and improves its compression. It costs more to decode.
3. `raw-json`: hand-authored or tiny objects, where readability matters more than size.

Sample:

1. `rle-json`: coherent, few-color objects with large regions of one material. The common case, and human-readable.
2. `packed-base64`: incoherent or many-color objects like noise, or color that changes almost every voxel, where `rle-json` would approach one run per voxel and balloon.
3. `raw-json`: hand-authored or tiny objects.

Favored pairs:

1. `bitmap-base64` + `rle-json`: coherent color, dense or speed-sensitive. The fast default.
2. `hilbert_index-delta-varint-base64` + `rle-json`: coherent color when you want the smallest (larger or sparser models); slower to decode.
3. `bitmap-base64` + `packed-base64`: incoherent or many-color.

Avoid pairing Hilbert positions with `packed-base64`: Hilbert order only helps by lengthening runs, which `packed-base64` does not use, so it costs decode time for no gain.

Positions and samples interact, so choose them as a pair. Encoding is offline, so you need not trust these rules: build the candidate pairs, compress each the way the file ships, and keep the smallest. All blocks assume whole-file gzip or deflate downstream.

## Palettes

A palette declares an attribute set once, then lists its cells as rows of values. `attributes` is the ordered list of attribute keys shared by every cell; `data` holds one row per cell, each row carrying that cell's values positionally aligned to `attributes`. So cell `c`'s value for `attributes[i]` is `data[c][i]`, every row has exactly `attributes.length` values, and a cell is referenced by its row index in `data`.

```jsonc
[
  // color + metallic
  { "attributes": ["rgba", "metallic"], "data": [["#FF0000FF", 1]] },

  // color only
  { "attributes": ["rgba"], "data": [["#00FF00FF"], ["#0000FFFF"]] },

  // PBR only
  { "attributes": ["metallic", "roughness"], "data": [[1, 0.2]] },
]
```

A voxel's material is the ordered merge of its sampled cells across the object's palettes; a later palette overrides an earlier one on shared attributes. This allows a shared base palette plus override layers.

### Example: an object with two palette layers, a base layer and a finish layer that share `roughness`, for a voxel sampling cell `0` of each.

Base layer:

```json
{ "attributes": ["rgba", "roughness"], "data": [["#FF0000FF", 0.9]] }
```

Finish layer:

```json
{ "attributes": ["metallic", "roughness"], "data": [[1, 0.2]] }
```

Cell `0` of each pairs its row with the palette's `attributes`, giving `{ rgba: "#FF0000FF", roughness: 0.9 }` and `{ metallic: 1, roughness: 0.2 }`. A later palette overrides an earlier one on shared keys, so the finish layer wins `roughness` (`0.9` -> `0.2`) and adds `metallic`:

```json
{ "rgba": "#FF0000FF", "roughness": 0.2, "metallic": 1 }
```

### Attributes

Recommended attributes (the format stores attributes generically; meaning is by convention). Omitted attributes use their default. Consumers ignore attributes they do not recognize. Names follow the glTF metallic-roughness vocabulary.

| Attribute      | Type            | Range | Default     | Meaning                                          |
| -------------- | --------------- | ----- | ----------- | ------------------------------------------------ |
| `rgba`         | hex `#RRGGBBAA` |       | `#FFFFFFFF` | sRGB color, straight alpha = opacity             |
| `metallic`     | number          | 0-1   | 0           | Metalness                                        |
| `roughness`    | number          | 0-1   | 1           | Roughness                                        |
| `occlusion`    | number          | 0-1   | 1           | Flat ambient occlusion (1 = none)                |
| `emissive`     | number          | 0+    | 0           | Emissive strength, scales `rgba` in linear space |
| `ior`          | number          | 1+    | 1.5         | Index of refraction                              |
| `transmission` | number          | 0-1   | 0           | Light transmission through surface               |

The `rgba` value is a hex string with a leading `#`, uppercase digits, and all eight `RRGGBBAA` digits present (no shorthand); it must match `^#[0-9A-F]{8}$`.

## Hierarchy Nodes

Nodes form a DAG (a node may have multiple parents; no cycles). Each references child nodes and child objects by index and carries a transform.

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

A transform has three fields: `position` is `[x, y, z]` (may be fractional), `rotation` is a unit quaternion `[x, y, z, w]`, and `scale` is `[x, y, z]`.

1. A transform composes as `Translation * Rotation * Scale`.
2. A node's world transform is `parentWorld * nodeLocal`; a root, listed in `rootHierarchyNodes`, has world = local. Reached through multiple parents, a node is placed once per path; this is instancing.
3. An object is placed at the world transform of the node referencing it; its voxels are in local voxel space.
4. `rotation` must be a unit quaternion; consumers may renormalize within a small tolerance (see [Validation](#validation)).
5. `scale` is per-axis; a negative component mirrors that axis and flips winding/handedness. A zero component is degenerate and invalid (see [Validation](#validation)).

The scene's roots are exactly the nodes listed in `rootHierarchyNodes`. A node that is neither listed as a root nor referenced as a child is unplaced and does not render, so a file may hold library nodes that are defined without being placed. The format describes placement only; how overlapping or sub-voxel placements are resolved, merged, or rasterized is consumer-defined.

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
2. Unknown **attribute** keys are ignored (attributes are advisory and convention-based), so adding attributes is backward compatible.
3. An unknown `encoding` (positions or samples) must be rejected; the block cannot be safely decoded.

## Validation

1. `version` is recognized.
2. All indices are in range:
   1. object `paletteRefs` -> `main.palettes`, and one object references each palette at most once
   2. each sample cell index -> the cell count of the palette it indexes
   3. each `childNodes` entry -> `main.hierarchyNodes` and each `childObjects` entry -> `main.objects`, and one hierarchy node lists each child node and each child object at most once
   4. `rootHierarchyNodes` -> `main.hierarchyNodes`, and each node appears as a root at most once
3. Position `data` is well-formed:
   1. `raw-json` is `[x, y, z]` triples
   2. `bitmap-base64` base64-decodes to exactly `ceil(X * Y * Z / 8)` bytes, its pad bits are zero, and the decoded number of voxels equals the number of set bits
   3. `hilbert_index-delta-varint-base64` `data` base64-decodes to an unsigned LEB128 varint stream of non-negative deltas, with every delta after the first strictly positive; `bits` derived from `bounds` is `<= 17` and equivalently every `bounds` dimension `<= 131072`; after decoding, every position lies in `[0, X) x [0, Y) x [0, Z)` and `bounds` is consistent with them
4. After decoding, voxel positions within an object are unique.
5. `bounds` is three non-negative integers and contains the decoded positions: for a non-empty object each component is at least that axis's maximum coordinate plus one; an empty object may carry any such `bounds`, canonically `[0, 0, 0]`.
6. Sample arity matches `paletteRefs.length`:
   1. `raw-json` has exactly one row per voxel, each row holding exactly that many cell indices
   2. `rle-json` has exactly that many channels
   3. `packed-base64` has exactly that many channels (base64 strings)
7. Each `rle-json` channel's run counts are positive and sum to the number of voxels. Each `packed-base64` channel base64-decodes to exactly `ceil(voxelCount * b / 8)` bytes for that channel's width `b = max(1, bitLength(c - 1))`, where `c` is the indexed palette's cell count, and its pad bits are zero.
8. Sample order matches the position block's voxel order (see [Voxel Order](#voxel-order)). This is an authoring invariant a validator cannot confirm.
9. In every palette, each `data` row has exactly `attributes.length` values, and `attributes` has no duplicate keys. Where a cell carries `rgba`, its value matches `^#[0-9A-F]{8}$`.
10. The hierarchy is acyclic.
11. No transform `scale` component is zero.
12. Every transform `rotation` has length-squared within `1e-6` of `1`; consumers may renormalize within this tolerance.

## Examples

### File Example

```jsonc
{
  "version": 1,
  "main": {
    "objects": [
      {
        "name": "Object A",
        "paletteRefs": [0],
        "bounds": [1, 1, 1],
        "voxelPositions": { "encoding": "raw-json", "data": [[0, 0, 0]] },
        "voxelSamples": { "encoding": "raw-json", "data": [[0]] },
      },
      {
        "name": "Object B",
        "paletteRefs": [1, 2],
        // Two voxels at (0, 0, 0) and (1, 0, 0).
        "bounds": [2, 1, 1],
        "voxelPositions": {
          "encoding": "raw-json",
          "data": [
            [0, 0, 0],
            [1, 0, 0],
          ],
        },
        // Per voxel: [palette-1 cell, palette-2 cell].
        "voxelSamples": {
          "encoding": "raw-json",
          "data": [
            [0, 0],
            [1, 0],
          ],
        },
      },
    ],
    "palettes": [
      { "attributes": ["rgba", "metallic"], "data": [["#FF0000FF", 1]] },
      { "attributes": ["rgba"], "data": [["#00FF00FF"], ["#0000FFFF"]] },
      { "attributes": ["metallic", "roughness"], "data": [[1, 0.2]] },
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
}
```

### TypeScript Schema

```typescript
interface VoxelJsonFile {
  version: 1;
  main: Main;
}

interface Main {
  objects: VoxelObject[];
  palettes: Palette[];
  hierarchyNodes: HierarchyNode[];
  // indices into hierarchyNodes; the scene's roots
  rootHierarchyNodes: number[];
  // user-defined extensions, conventionally vendor-keyed; the core format
  // assigns no meaning and guarantees nothing about its contents
  ext?: { [key: string]: JsonValue };
}

// Pure geometry; placed only by a hierarchy node that references it.
interface VoxelObject {
  name: string;
  // indices into Main.palettes, in resolution order
  paletteRefs: number[];
  // [X, Y, Z] size in voxels; voxels occupy [0, X) x [0, Y) x [0, Z). Must
  // contain every voxel (per-axis >= max + 1). Required to decode bitmap-base64
  // and hilbert_index-delta-varint-base64; margin is not always free under
  // those.
  bounds: Vec3;
  voxelPositions: PositionBlock;
  voxelSamples: SampleBlock;
}

// ## Voxel Encoding

// Both blocks share one voxel order, fixed by the position encoding (see Voxel
// Order); sample channels are always in that order. The match is an authoring
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
  | { encoding: "hilbert_index-delta-varint-base64"; data: string };

type SampleBlock =
  // One entry per voxel: that voxel's samples, one cell index per palette in
  // order.
  | { encoding: "raw-json"; data: number[][] }
  // One channel per palette: a flat run stream
  // [value1, count1, value2, count2, ...].
  | { encoding: "rle-json"; data: number[][] }
  // One channel per palette: each voxel's cell index bit-packed at width
  // b = max(1, bitLength(c - 1)) for that palette's cell count
  // c, MSB-first, base64-encoded (the same packing as the bitmap-base64
  // position encoding).
  | { encoding: "packed-base64"; data: string[] };

// ## Palettes

// A palette declares its attribute keys once, then lists cells as rows of
// values aligned to those keys: cell c's value for attributes[i] is data[c][i].
// Every row has attributes.length values; a cell is referenced by its row
// index.
interface Palette {
  attributes: string[];
  data: AttributeValue[][];
}

// An attribute value is any valid JSON value. The recommended attributes use
// strings (rgba) and numbers, but the format stores values generically;
// attribute meaning is by convention and unknown keys are ignored by consumers.
type AttributeValue = JsonValue;

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

### Hilbert Reference Code

```ts
// Reference algorithm for `hilbert_index-delta-varint-base64`, as three
// independent encode/decode codecs (hilbert, delta, varint) plus a composition.
// The block `data` is base64(varintEncode(deltaEncode(sortedHilbertIndices))).
//
// NOTE: indices are assembled with arithmetic, not `<<`, because JS bitwise
// operators are 32-bit and an index can exceed 31 bits on large grids. The
// index is exact in a JS `number` only while 3 * bits <= 53, i.e. bits <= 17;
// the format caps bits at 17 (every bounds dimension <= 131072) for this
// reason.

// 0. Bit length: number of binary digits in a non-negative integer, with
//    bitLength(0) = 0 (so bitLength(n) = floor(log2(n)) + 1 for n >= 1). The
//    width formulas below call bitLength(x - 1), which is the number of bits
//    needed to index x distinct values (= ceil(log2(x)) for x >= 1) and is
//    integer-exact with no floating-point rounding. Used to derive Hilbert
//    `bits` and the packed-base64 width `b`; never use Math.log2 for these.
function bitLength(n: number): number {
  let len = 0;
  while (n > 0) {
    n = Math.floor(n / 2);
    len++;
  }
  return len;
}

// Hilbert `bits` per axis from bounds, and packed-base64 channel width `b` from
// a palette cell count. Both are max(1, ceil(log2(.))) computed via bitLength.
function hilbertBits(bounds: Vec3): number {
  const maxDim = Math.max(bounds[0], bounds[1], bounds[2]);
  return Math.max(1, bitLength(maxDim - 1));
}

function packedWidth(cellCount: number): number {
  return Math.max(1, bitLength(cellCount - 1));
}

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
// positive. `bits` is hilbertBits(object.bounds); packed-base64 channels use
// packedWidth(cellCount).
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
