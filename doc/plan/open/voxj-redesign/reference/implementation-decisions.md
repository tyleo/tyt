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

### Second chunk: value-pool content validation is one `value-pools` check

The deferred rule-9 work lands as a single named check, `value-pools`, in
`check_value_pools.rs`, reported between `version` and `palettes` to mirror the
spec's rule order (9 before 10). It runs before `palettes` in the driver, so a
fail-fast run surfaces a malformed pool before the palette that reads it. The
report list grows from twelve checks to thirteen.

The check covers only the content rules that the typed pool model cannot enforce
at parse. Parse already guarantees the value SHAPE: a bound is present only on
`int` and `float` (the enum variants), `null` is rejected outside a `json` pool
(the typed `Vec`s), a color array has exactly its kind's length (`[f64; N]`), a
number is finite (JSON has no NaN or Infinity), and an unknown key or kind
rejects (`deny_unknown_fields`, tagged enum). Finiteness (spec rule 3) is a
parse-boundary guarantee the whole validator leans on, the same way
`check_transforms` never re-guards a non-finite scale, so `value-pools` does not
re-check it either; a `VoxjFile` built in memory with a non-finite float on an
unbounded side is out of the parse-then-validate contract these checks assume.
So the check verifies four things parse leaves open:

1. `values` is non-empty, via `VoxjValuePool::values_len()`, for every kind.
2. int/float numeric bounds are well-formed: an int pool's numeric bounds are
   integer-valued (rule 9.3.3), and `min <= max` when both are finite (9.3.4).
3. every int/float value lies within `min`/`max` (9.2.4, 9.2.5). Both kinds
   compare through `f64`; the format's numeric ranges are small, so the `i64`
   widening is exact in practice. A `"none"` bound is unbounded on that side.
4. hex colors match `#` plus exactly six or eight UPPERCASE hex digits (9.2.6,
   9.2.7), hand-rolled rather than pulling in a regex dependency; and float color
   components lie in their space's range, sRGB in `[0, 1]` and linear `>= 0`
   (9.2.8, 9.2.9), over both the 3- and 4-component arrays via one const-generic
   helper.

The block-internal rules (11.2, 11.3, 13, 14: rle counts, packed and bitmap pad
bits, hilbert deltas and bit width, base64 canonicality) remain deferred to the
final Phase 2 chunk; they tighten the decode path rather than adding a pool
check, so they stay a separate reviewable unit.

### Final chunk: block-internal rules tighten the decode, no new check name

The last Phase 2 chunk enforces the block-internal rules by hardening
`decode_voxj_object` and `decode_varint` so a malformed encoded block fails to
decode. `check_geometry` already decodes every object and reports a decode
failure as the `blocks` check, so these rules surface there with no new
`Check` variant, and the report list stays at thirteen. The specific tightenings:

1. rle-json (rule 11.2): `rle_decode` now returns a `Result` and rejects an
   odd-length stream (a value with no count) and a zero count. The remaining
   clauses were already covered: value-in-`[0, M)` by the `sample-materials`
   check over the decoded samples, and counts-sum-to-`V` by the existing
   per-channel length check in `decode_samples`.
2. packed-base64 (rule 11.3) and bitmap-base64 (rule 13.2): one shared helper,
   `check_packed_bytes`, requires the decoded byte length to equal exactly
   `ceil(bits / 8)` (the old packed check only rejected a short channel, not a
   long one) and requires the final byte's unused low bits to be zero. Because
   the packing is MSB-first, the pad bits are the last byte's low `pad` bits, so
   the mask is `(1 << pad) - 1`; the exact length rules out padding anywhere
   else. Decoded-value-in-range stays with `sample-materials` (packed) and set
   bits define the voxel count (bitmap).
3. hilbert (rule 13.3): the decode caps `bits` at `MAX_HILBERT_BITS = 17`
   (13.3.2) and errors before touching the data, and `decode_varint` now returns
   a `Result` that rejects a stream ending mid-value or a value past 64 bits,
   which previously panicked or wrapped on malformed input (13.3.1, well-formed
   varint). The strictly-positive-delta clause is enforced implicitly: a
   non-positive delta after the first repeats a Hilbert index, so it decodes to a
   duplicate position that `unique-positions` rejects, and Hilbert decode is a
   bijection so a duplicate position can arise no other way. In-bounds (13.3.3)
   stays with the `bounds` check.
4. base64 canonicality (rule 14): needed no code. base64 0.22's
   `general_purpose::STANDARD` decodes with `RequireCanonical` padding,
   `decode_allow_trailing_bits = false`, and the standard (non-url) alphabet, so
   a base64url character, wrong `=` padding, non-zero trailing bits, or embedded
   whitespace already rejects at decode. Every base64 block field routes through
   this one engine.

`decode_varint`'s overflow guard checks `shift >= 64` before shifting, so a
legal `<< 63` never panics and an 11th continuation byte rejects; a real hilbert
delta needs at most `3 * 17 = 51` bits, well inside the guard. These are decode
changes, so they harden the actual decoder used by `from_voxj_file_bytes`, not
just the validator.
