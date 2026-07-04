# Implementation decisions

_Part of the [Voxj Redesign Migration Plan](../README.md)._

Code-level decisions made while executing the
[checklist](../checklist.md), recorded as they land. The plan-level decisions and
their rationale live in the [README](../README.md#decisions); this log is for the
finer implementation choices a reviewer of the Rust would want explained, for
example how min/max bounds are modeled in serde, whether voxcore colors ride as a
dedicated value variant or a typed array, and how the vmax fold reconstructs
original color and material indices on write-back.

## Phase 1: `voxj` data model

### `VoxjValuePool` is a per-kind internally-tagged enum with typed values

The checklist offered a bounded-versus-unbounded enum split or a struct with
optional bounds. At the owner's direction the pool is one enum per kind, tagged
by `kind`, with every value type spelled out: `json` holds `Vec<VoxjValue>`,
`bool` holds `Vec<bool>`, `float` holds `Vec<f64>`, `int` holds `Vec<i64>`,
`string` and the hex kinds hold `Vec<String>`, and the vector color kinds hold
`Vec<[f64; 3]>` or `Vec<[f64; 4]>`. The bounded variants (`float`, `int`, and the
four vector color kinds) carry required `min`/`max: VoxjBound` fields; the
unbounded variants carry only `values`.

Serde renders it as `{ "kind", ... }` via `serde(tag = "kind")`, so the typed
fields do a lot of validation at parse: a malformed value rejects by type, a
missing bound on a bounded kind rejects as a missing field, a stray bound on an
unbounded kind rejects as an unknown field, and an unknown `kind` rejects as an
unknown variant. This moves value-shape checking (spec validation rule 9.2) to
parse time; the remaining value-BOUNDS check (each value within `min`/`max`)
stays a voxj-codec named check in Phase 2. A separate lightweight
[`VoxjValuePoolKind`] tag enum is kept, and `VoxjValuePool::kind()` maps a pool
to it, for code that needs the kind without matching the whole pool.

Consequence to note for goldens: `float` and vector-color values serialize
through `f64`, so an integral component writes as `1.0`, not `1`. `VoxjBound`
still writes integral bounds as `1`, so a float pool can read
`{ "min": 0, "max": 1, "values": [0.0, 0.5, 1.0] }`. Both forms are valid JSON;
the difference is cosmetic and deterministic.

### `VoxjBound` rejects JSON null at the serde layer

`VoxjBound` is `Number(f64) | None`, with a manual `Serialize` that writes a
number or the string `"none"` (integral numbers as JSON integers, mirroring
`VoxjValue`) and a manual `Deserialize` whose visitor accepts only a finite
number or `"none"`, so JSON `null`, booleans, and non-finite numbers reject at
parse. On the pool enum `min`/`max` are plain required `VoxjBound` fields on the
bounded variants, so a present `"min": null` reaches `VoxjBound` and rejects
directly; there is no `Option`, so no `deserialize_with` workaround is needed to
stop serde swallowing a null as an absent bound.

### Struct field order follows the owner's chosen layout

Serde serializes struct fields in declaration order, so field order sets the
wire key order. The owner fixed the layout: `VoxjRuntimeState` is `valuePools,
palettes, objects, hierarchyNodes, rootHierarchyNodes` (the pools and palettes
that objects reference come first); `VoxjObject` is `name, bounds, origin,
voxelPositions, layerPaletteRefs, voxelSamples`; `VoxjHierarchyNode` is `name,
transform, childNodes, childObjects`. The spec doc's examples and TypeScript
schema are reordered to match. JSON parsing is order-independent, so this only
fixes the write order for downstream goldens.

### `VoxjValue` is now only the `json` pool kind and `ext`

Because the pool enum types its values per kind, `VoxjValue` is no longer the
storage for every pool. It backs the `json` pool kind, which is arbitrary JSON
including `null`, and the opaque `ext` namespace. That is the type's original
purpose.

### Strictness: `deny_unknown_fields` on every closed struct AND on the tagged enums

`deny_unknown_fields` is on `VoxjFile`, `VoxjMain`, `VoxjRuntimeState`,
`VoxjEditState`, `VoxjObject`, `VoxjPalette`, `VoxjPaletteBinding`,
`VoxjTransform`, `VoxjHierarchyNode`, and `VoxjEditObject`. It is also on the
three tagged enums, `VoxjValuePool` (internally tagged) and `VoxjPositionBlock`
and `VoxjSampleBlock` (adjacently tagged). An earlier revision claimed the
adjacently-tagged block enums already reject stray sibling keys through serde's
tag representation; that was wrong. A probe showed a bare adjacently-tagged enum
silently ignores an unknown sibling key like `stray`, which violates spec
validation rule 5 (the encoding block is a closed structure). Adding
`deny_unknown_fields` to the adjacently- and internally-tagged enums fixes it:
serde 1.0.228 honors `deny_unknown_fields` on both tag styles, so every closed
structure in the file, including the encoding blocks and the value pool, now
rejects unknown keys at parse. `VoxjObject.origin` lost its `serde(default)`:
the redesign dropped the omitted-origin-defaults-to-zero rule, so origin is now
required and a missing `origin` rejects.

### No `serde_json` dependency; the document round-trip gate moves to Phase 2

`voxj` depends only on `serde`, per the checklist ground rule. The Phase 1 serde
behaviors, the kebab-case kind tags, the typed value shapes, `min`/`max` null
rejection, and unknown-key and unknown-kind rejection, were verified with a
throwaway probe crate rather than an in-crate test, so no `serde_json`
dev-dependency was added. The gate's "a hand-written new-shape document
round-trips through serde" check lands in Phase 2, in `voxj-codec`, which owns
`from_voxj_file_bytes` and already depends on `serde_json`; the strictness
negative cases move there with it.

### `VoxjValuePoolKind` uses `rename_all = "kebab-case"`

The eleven-kind enum renders each variant to its kebab-case tag
(`SrgbaHex` -> `srgba-hex`, `LinearRgbFloat` -> `linear-rgb-float`, and so on),
which matches every wire tag, rather than eleven explicit `rename` attributes.
A fieldless enum with no `serde(other)` rejects an unrecognized tag by default.

### Color kinds carry no `min`/`max`; the color space fixes the range

The owner removed bounds from all four vector color kinds. The bounded kinds are
now only `int` and `float`. The rationale: bounds earn their place when the range
is a per-attribute choice the format cannot know (`metallicFactor` is 0..1, `ior`
is 1..none), so the pool must state it. A color's range is not a choice; it is a
property of the color space, so it belongs to the kind, not the pool. The Q5
writer table had always emitted the same bounds per color kind (`0..1` for sRGB,
`0..none` for linear), confirming they carried no per-pool information. Dropping
them removes boilerplate from every color pool, removes the incoherent case of a
color pool with an odd declared range, and makes the whole color family uniformly
unbounded, matching the already-unbounded hex kinds.

In code the four color variants of `VoxjValuePool` lose their `min`/`max` fields
and become value-only, like the hex variants. The range is enforced intrinsically
by the kind: sRGB float components in `[0, 1]`, linear float components `>= 0`.
That per-kind range check is a voxj-codec named check in Phase 2; the wire types
carry `Vec<[f64; 3]>` / `Vec<[f64; 4]>`, so a non-numeric or wrong-length color
still rejects at parse. The spec doc, the Q5 table, and the TypeScript schema are
updated to match, and the TypeScript `ValuePool` is spelled out as a per-kind
union so each kind shows its own value type instead of `JsonValue[]`.

## Phase 2: `voxj-codec`

### Chunk boundary: codec path plus structural validation now, content rules later

Phase 2 is one compilation unit and cannot be split along a codec-versus-
validation seam: validation reads the palette shape, so it has to compile against
the new types the moment the codec does. This first chunk ports the codec path
(checklist items 1 to 4) and the STRUCTURAL palette, index, and geometry
validation (the palette-binding, materials-column, value-index-range, and
sample-material-index parts of the twenty rules, plus dropping the removed
no-duplicate-palette-ref rule). It defers the value-pool CONTENT validation (spec
rule 9: kind well-formedness, values non-empty, value within min/max,
integer-valued, color component ranges) and the stricter block-internal rules
(rules 11.2, 11.3, 13, and 14: rle counts, packed pad bits, bitmap/hilbert pad
and delta rules, base64 canonicality) to a follow-up chunk. Those are additive
named checks and tighter decode; they do not block compilation, so splitting them
off keeps each commit reviewable.

### `voxj_palette_material_counts`: M is the first materials column's length

The helper, renamed from `voxj_palette_cell_counts`, derives each layer's
material count M as `palette.materials.first().map_or(0, Vec::len)`. Column-major
materials store one column per binding, each of length M, so any column's length
is M and the first is authoritative. The `palettes` check separately guarantees
bindings are non-empty and every column shares the length M at least 1, so the
helper trusts the validator rather than asserting rectangularity, and it stays
panic-free when run pre-validation on a ragged or empty palette. encode and
decode both call this one helper, so packed-base64 widths derive identically on
both sides.

### Renames follow the cell-to-material and palette-to-layer model

Per Q7: `cell_counts` becomes `material_counts`, `num_palettes` becomes
`num_layers`, `VoxjDecodedObject.palette_refs` becomes `layer_palette_refs`, and
the sample docs reframe a sample from a cell index to a per-layer material index.
The internal raster helpers `cell_index` and `cell_to_position` keep their names:
they address grid cells of the dense occupancy bitmap, unrelated to palette
materials. `position_encoding.rs` needed no change; a position encoding carries
no palette or material framing to reframe.

### `VoxjValuePool::values_len()` added to voxj

The `palettes` check validates that every materials value-index falls in
`[0, pool.values.len())`, which needs the pooled value count independent of kind.
That accessor belongs on the pool, so `VoxjValuePool::values_len()` was added to
voxj alongside `kind()`, a small additive helper downstream crates will also
want, rather than re-matching all eleven variants inside voxj-codec.

### Named checks: `sample-cells` becomes `sample-materials`; poolRef range in `palettes`

`Check::SampleCells` becomes `Check::SampleMaterials` (report name
`sample-materials`). The check set otherwise keeps its shape and count for this
chunk; the new value-pool content check arrives with the deferred rule-9 work.
Binding `poolRef` range (spec rule 6.2) is validated in `check_palettes`, not
`check_indices`: `check_palettes` must resolve each binding's pool anyway to
range-check its value-indices, so co-locating the poolRef check keeps that logic
together. `check_indices` keeps the runtime-state-level refs (layer palette refs,
child nodes, child objects, roots) and drops the no-duplicate-palette-ref rule,
since two layers may share a palette. `check_geometry` reuses the per-layer M the
helper already produced instead of re-deriving a count, so the sample-range check
and the decode width agree by construction.
