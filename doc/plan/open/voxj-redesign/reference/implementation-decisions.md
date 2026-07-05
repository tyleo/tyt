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

## Phase 3: `voxcore`

### Chunk boundary: additive leaf pool types first; brand rename deferred

Phase 3 is the crux and the largest blast radius: `VoxPalette` is an unsafe
struct-of-arrays with hand-proved invariants, and the rewrite ripples into
`VoxObject`, `VoxRuntimeState`, `VoxMain::validate`/`gc`, `error.rs`, and
`VoxGcRemap`. The first chunk is the smallest thing that compiles and passes on
its own: the additive leaf types the rest of the phase builds on. It adds
`VoxValuePoolKind`, `VoxBound`, `VoxValuePool`, `VoxPaletteBinding`, and the
`BVoxValuePool` brand, and registers them in `lib.rs`. It touches no existing
code beyond `lib.rs`, so every existing test passes unchanged.

Checklist item 1 also lists "rename the cell brand to a material brand." That is
deferred to the `VoxPalette`-rewrite chunk, where `BVoxPaletteCell` becomes a
material brand at the same time the type stops being a rectangular cell grid and
becomes column-major materials. Renaming the brand now, while the grid semantics
are unchanged, would churn 30+ downstream files that name `BVoxPaletteCell`
(across `voxsmith` and `vxl`) and leave a misleading "material" brand describing
a cell grid. Item 1 stays unchecked until that rename lands with the rewrite.

### `VoxValuePool` is a per-kind discriminated union with exact typed values

At the owner's direction `voxcore`'s pool mirrors the wire's `VoxjValuePool`: a
per-kind enum, one variant per kind, with every value type spelled out rather
than a uniform `(kind, Vec<VoxValue>, min, max)` struct. `json` holds
`Vec<VoxValue>`, `bool` holds `Vec<bool>`, `float` holds `Vec<f64>`, `int` holds
`Vec<i64>`, `string` holds `Vec<String>`, and the four color kinds hold
`Vec<[f64; 3]>` or `Vec<[f64; 4]>`. The value shapes are exact at the type level,
so a color is a fixed-length float array rather than an untyped structure a
reader must re-check. `min`/`max` live only on the `float` and `int` variants,
so the discriminated union makes the bounds optional by construction, the same
shape the wire type uses. A lightweight `VoxValuePoolKind` tag enum is kept and
`VoxValuePool::kind()` maps a pool to it, for code that needs the kind without
matching the whole pool, mirroring `voxj`. The only methods are `values_len()`
and `kind()`, matching the wire's surface; a pool is built by variant literal,
and the value-well-formed-for-kind and within-bounds checks are
`VoxMain::validate`'s job (item 6).

### Nine canonical kinds; colors are exact typed float arrays

`VoxValuePoolKind` is the nine canonical kinds `json, bool, float, int, string,
srgb, srgba, linear-rgb, linear-rgba`. The wire's six color kinds (hex and float
per space) collapse onto the four color kinds here, since `voxcore` stores a
color as float components in the space's natural range and does not carry the
on-wire encoding (per Q1). This settles checklist item 5 a third way: neither a
`VoxValue` color variant nor colors riding as `VoxValue::Array`, but exact
`Vec<[f64; N]>` in the color variants themselves, so `VoxValue` is unused for
colors and backs only the `json` kind and `ext`, exactly as the wire's
`VoxjValue` does. `int` and `float` are the only kinds that carry bounds, since
the discriminated union puts `min`/`max` only on those two variants.

### `VoxBound` has no serde; `VoxPaletteBinding.pool` is a branded id

`VoxBound` is `Number(f64) | None`, mirroring the wire's number-or-none bound but
without the wire type's manual `Serialize`/`Deserialize` (no serde in `voxcore`).
It does not enforce finiteness at construction; a non-finite numeric bound is a
`VoxMain::validate` concern (item 6/7), consistent with how `check_transforms`
leaves finiteness to validation. `VoxPaletteBinding` holds `pool:
U32Id<BVoxValuePool>`, a branded id, not the wire's plain `usize` `poolRef`, so a
pool is a first-class cross-reference that gc relabels alongside objects,
palettes, and nodes (item 8). `branded_id`'s `scalar_id!` hand-implements
`Clone`, `Copy`, `Debug`, `Eq`/`PartialEq`, `Hash`, and `Ord` for any
`TBrand: ?Sized` with no bound leakage, so `VoxPaletteBinding` derives `Eq`
even though `BVoxValuePool` is a bare marker struct with no derives.

### Chunk 2: the value-pool store lands additively before the palette rewrite

Chunk 2 adds the shared value-pool store to `VoxRuntimeState` and, pulled
forward from checklist item 6, the additive `VoxMain` accessors that reach it.
Like chunk 1, the chunk is purely additive: no existing signature or behavior
changes, so all of `voxcore` stays green and every prior test passes unchanged.
This keeps the crux (the `VoxPalette` rewrite, which cannot compile in halves)
as its own reviewable chunk rather than smearing it across this one.

The store is `value_pool_ids: IdStruct<BVoxValuePool>` plus
`value_pools: IdField<BVoxValuePool, VoxValuePool>`, the same id-pool-plus-column
shape as the object, palette, and hierarchy-node stores. The checklist offered a
plain indexed vec instead; the id-pool shape wins because `VoxPaletteBinding.pool`
is a `U32Id<BVoxValuePool>`, so a pool must be a first-class pooled entity that
`gc` relabels alongside objects, palettes, and nodes once the rewrite lands. The
fields lead the struct, so `VoxRuntimeState` now mirrors the wire's
`valuePools`-first layout (`valuePools, palettes, objects, hierarchyNodes,
rootHierarchyNodes`, per Phase 1's field-order note); field order has no wire
effect in `voxcore` (no serde), so this is only for readability.

`clone_runtime_state` clones each pool with a plain `.clone()`, not the
`.clone_object()`/`.clone_palette()` rebuild the SoA-backed columns need, because
`VoxValuePool` is plain data and derives `Clone`. `Drop` releases the
`value_pools` column against `value_pool_ids` like the others, so the pooled
`Vec`s are freed rather than leaked.

Pulled forward from item 6: `add_value_pool`, `value_pool_count`, `value_pool`,
and `iter_value_pools`, mirroring the palette accessors, so the store is
reachable and a round-trip plus a deep-copy test exercise it. Deferred to the
palette-rewrite chunk (items 3, 6, 8): `validate` gains no pool checks yet, `gc`
does not compact the pool store yet, and there is no `remove_value_pool`. All
three are safe to defer here. With no bindings referencing pools and no removals
possible, the pool ids stay contiguous from zero, so an untouched `gc` leaves
them correctly numbered; and `validate` ignoring pools cannot admit a state a
later reader would reject, since nothing reads a pool yet.

### Chunk 3: the `VoxPalette` rewrite lands the whole palette blast radius at once

Phase 3's crux cannot compile in halves: `VoxObject` samples reference palette
entries, `VoxMain::validate`/`gc` coordinate the palette/object/pool relabelings,
`error.rs` and `VoxGcRemap` name the palette entities, so the palette rewrite and
its ripple (checklist items 1 brand rename, 3, 4, 5, 6, 7, 8, 9) land together.
Every `voxcore` test is rebuilt in the same change. The result is verified
leak- and UB-clean under Miri as well as green under normal `cargo test`, since
the rewrite is dense unsafe struct-of-arrays code.

### Material-major SoA mirrors the old cell grid; Copy value-indices simplify it

`VoxPalette` stores `bindings: IdField<BVoxPaletteBinding, VoxPaletteBinding>`
and `materials: IdField<BVoxMaterial, IdField<BVoxPaletteBinding, u32>>`, the
same material-major (row-per-material) shape the old grid used, with the inner
`VoxValue` swapped for a `u32` value-index into the binding's pool. The wire is
column-major, but the in-memory SoA layout is an implementation choice and the
voxj seam (Phase 4) will transpose on read/write; keeping the old layout reuses
the proven unsafe `Drop`/`gc`/`clone`/`remove` logic with type swaps, which is
the lowest-risk transform. Because the inner value is now `Copy` `u32` rather
than heap-owning `VoxValue`, three paths simplify: `Drop` and `remove_material`
skip the per-binding inner release (dropping the inner `IdField` frees its buffer
and the `u32`s need no drop), `clone_palette` clones each material's inner column
bytewise via `IdField: Clone` (available only for `TValue: Copy`) instead of
rebuilding it value by value, and `remove_binding` leaves each material's stale
`u32` slot in place (a no-op to release) for `gc` to compact, only freeing the
binding's attribute `String`. Reads of a removed binding are guarded by binding
retention, and `gc`'s per-material `inner.gc(binding_remap)` never reads a
removed binding's slot (its remap entry is `None`), so the stale slot is
invisible and harmless.

### Bindings stay a branded id pool; materials and pools are the cross-referenced ids

Three entities could be branded: bindings, materials, value pools. Materials are
referenced by object voxel samples and value pools by palette bindings, so both
must be branded ids that `gc` relabels and other entities hold across the call;
they are `BVoxMaterial` (renamed from `BVoxPaletteCell`) and the existing
`BVoxValuePool`. Bindings are referenced only within their palette (they key the
per-material value-index columns), so they need no cross-reference brand, but
keeping them a branded id pool (`BVoxPaletteBinding`, renamed from
`BVoxAttribute`) reuses the old attribute pool's `gc`/`Drop` code verbatim and
matches the crate's uniform SoA style. The object's per-layer entries become
`BVoxLayer` (renamed from `BVoxPaletteRef`). The three brand renames ripple into
`voxsmith` and `vxl`, but those crates are rewritten in later phases and would
not compile against the new API regardless, so the renames add no net churn and
keep the brand names honest about the new model.

### `value_index` on the palette, `material_value` resolution on `VoxMain`

The pool a binding draws from lives in `VoxRuntimeState`, not the palette, so
`VoxPalette` cannot resolve a value on its own. `VoxPalette::value_index(material,
binding)` returns the raw `u32` index, and `VoxMain::material_value(palette,
material, binding)` hops palette -> binding pool-ref -> pool, returning
`(&VoxValuePool, u32)` for the caller to read by kind. This is the checklist's
"resolve-material-and-attribute-to-value read," split across the two owners of
the data.

### `gc` compacts the pool store and relabels binding pool-refs

`gc` now compacts `value_pool_ids` first, then calls
`VoxPalette::relabel_value_pools` on every palette to translate each binding's
`pool` id through the pool remap, before compacting palettes and objects.
`VoxGcRemap` gains `value_pools` and renames `cells` to `materials`. With no
`remove_value_pool` path the pool ids are already contiguous, so the relabel is
currently the identity; it is implemented in full anyway so `gc` uniformly
compacts every pool and a future pool removal is covered. Value-indices inside
materials point at pool *contents*, which `gc` never reorders, so they stay valid
across a `gc` untouched.

### `validate` value-pool content mirrors the voxj-codec rule-9 check

`VoxMain::validate` runs a value-pool content check first (so a palette that
reads a malformed pool is reported after the pool), mirroring voxj-codec's
`value-pools` check as the in-memory types allow: pool non-empty (`EmptyPool`);
`int`/`float` bounds finite, integer-valued for `int`, and `min <= max`
(`PoolBound`); each `int`/`float` value finite and within bounds, and each color
component in its space's range, sRGB `[0, 1]` and linear `>= 0` (`PoolValue`).
Unlike the wire, `voxcore` has no parse boundary, so finiteness is checked here
rather than assumed: numeric bounds explicitly, `float` values explicitly, sRGB
color components by their `[0, 1]` range test (which rejects any non-finite),
and linear color components by an explicit finiteness guard alongside the `>= 0`
test, since a bare `>= 0` would let `+Infinity` through. Then per
palette: binding pool-refs resolve (`BindingPool`), attribute keys are unique
(`DuplicateBindingAttribute`), and every material value-index is within its
binding's pool (`MaterialValue`). Material *column arity* needs no runtime check:
`add_material` retains exactly one value-index per binding, so a material
structurally has a value-index for every binding, which `value_index`'s
`expect` documents. The duplicate-palette-ref rule is dropped, since two layers
may share a palette.

### `VoxPalette` carries a stored attribute-name index

`VoxPalette` keeps a `by_attribute: HashMap<String, U32Id<BVoxPaletteBinding>>`
alongside the SoA columns, exposed as `binding_by_attribute(&str)`, so callers
resolve a binding by its attribute name in O(1) rather than scanning
`iter_bindings` (the pattern every converter uses). It is maintained rather than
scanned by choice: `add_binding` inserts, `clone_palette` clones it, and `gc`
rebuilds it from the relabeled bindings (a full rebuild is O(bindings) and
simpler than relabeling each value through the binding remap). `remove_binding`
drops the entry only when it still points at the removed binding, so a duplicate
attribute (which `validate` rejects, but which can exist transiently) that
overwrote the entry with a later binding leaves that later binding's lookup
intact. The index is therefore exact for a validated palette, whose attribute
keys are unique, and best-effort otherwise; `validate` still catches duplicates
by iterating the bindings, not the index. The added `HashMap` is plain data
outside the unsafe SoA machinery: it drops on its own after `Drop` runs and needs
no manual release, and Miri confirms the palette stays leak-clean.

## Phase 4: `voxsmith` voxj seam

### The whole seam is one chunk; verified against a scoped `voxj`-only build

Phase 4 lands as a single chunk covering the whole read and write seam. voxsmith
cannot compile until phases 5 to 7 port the remaining voxcore consumers (the
converters, the glTF pipeline, the mesh code, and `reduce_palette`), so no
smaller sub-chunk builds any better, and the read-versus-write round-trip
symmetry is the reviewable property that a half-ported seam would hide. On
default features `cargo check -p voxsmith` leaves its errors entirely in the
phase-5/6 modules and none in `internal/voxj` or `convert/voxj`. To actually run
the seam tests, the crate was built with `--no-default-features --features voxj`,
which cfg-gates out every other codec module; the one unconditional straggler,
`reduce_palette`, was temporarily gated out and then reverted. All 25 seam tests
pass under that build. `from_voxj_file` runs voxcore's `validate`, not
voxj-codec's, so the seam validates the palette shape it must build (duplicate
attribute, column arity, ragged columns, hex form) and defers every range and
bound check to voxcore.

### Kind-directed value conversion lives at the pool level, not on `VoxValue`

The checklist framed the kind-directed decode and encode as a rework of
`vox_value_from_voxj_value` and `voxj_value_from_vox_value`. In the full-pool
model those two functions convert only `json`-pool values and the `ext` block,
because voxcore's `VoxValue` and the wire's `VoxjValue` now back only `json` and
`ext`; every other kind stores typed values on its pool variant. So the
kind-directed conversion is a pool-level concern and landed as two new functions,
`vox_value_pool_from_voxj_value_pool` and `voxj_value_pool_from_vox_value_pool`,
one match arm per kind. The named `VoxValue` converters are unchanged and the
`json` arm calls them per value. Per-value bound and range validation stays
voxcore's `validate`; the seam only reshapes.

### The palette converter reshapes bindings and materials; pools convert separately

Value pools are shared `VoxRuntimeState` entities, not palette-local, so
`vox_palette_from_voxj_palette` handles only the bindings and the material
transpose, and `from_voxj_file` converts each pool once at the `VoxMain`-assembly
level, adding pools before palettes so bindings resolve. The wire stores
materials column-major, one column per binding; voxcore stores per-material rows
of value-indices. The reader transposes columns into rows and the writer reads
each binding's column back down the materials, so the two are exact inverses. The
reader rejects a duplicate binding attribute (replacing the old last-wins dedup),
a `materials` column count that disagrees with the bindings, and a ragged column;
`add_material`'s arity is then always satisfied, so its `None` path is defensive.

### The write seam is a faithful 1:1 mirror; dedup and the bounds table are converter work

The Q5/Q6 writer duties, one pool per `(kind, bounds)` deduped document-wide, the
per-attribute min/max default table, and identical-material dedup, were written
before Q1 chose the full-pool voxcore model. With that model voxcore already
holds pools as first-class shared entities carrying their own bounds, and the
converters build the distinct pool and material sets. So the voxj write seam
mirrors voxcore one to one in id order, exactly as the old `voxj_palette_from_
vox_palette` mirrored the old cell grid, and emits wire index == voxcore id so
cross-references map by `id.to_u32()`. Deduping pools by `(kind, bounds)`, the
per-attribute bounds table, and material dedup move to the converters and the
voxelizer in phases 5 and 6, where a voxcore pool is first constructed and its
bounds first chosen. This keeps the seam a pure reshape and avoids re-deduping
data voxcore already deduplicated.

### `ColorFormat`: float default, hex round-trips exactly, linear is always float

`ColorFormat` is a two-variant enum, `Float` (the default, per Q5) and `Hex`,
added under `convert/voxj` and threaded through `write_voxj` and
`VoxjFileBuilder`; the `--color-format` CLI flag that drives it lands in phase 7.
Because voxcore stores every color as float components, the format only picks the
on-wire encoding for the two sRGB kinds: `Hex` emits `srgb-hex`/`srgba-hex`,
`Float` emits `srgb-float`/`srgba-float`. Linear colors have no hex form and
always serialize as float regardless. Decode divides each hex byte by 255; encode
clamps to `[0, 1]`, multiplies by 255, and rounds, so a byte-valued component
round-trips hex to float to hex exactly (the `round` absorbs the `x/255*255`
float error). Reading an `srgba-hex` document and writing it back at the default
therefore intentionally changes the on-wire kind to `srgba-float`, the accepted
Q5 churn.

### A zero-material palette round-trips; wire rule 10.4 is voxj-codec's to enforce

A voxelless object may reference a palette with bindings but no materials, which
round-trips as `materials: [[]]`, one empty column per binding. voxcore's
`validate` accepts it (no material means no value-index to range-check), and
`from_voxj_file` runs only voxcore's validate, so the seam preserves the old
empty-palette behavior. The wire's stricter rule 10.4 (`M >= 1`) is a
voxj-codec `validate` rule, applied by `vxl validate`, not by the load path.

### Adversarial review: ragged-column bug fixed; two hostile-input gaps deferred

A multi-agent adversarial review of the seam surfaced three distinct confirmed
issues. One was a real bug in the new code and is fixed here; two are
hostile-input-only gaps in the pre-existing load-path convention and are
deferred.

Fixed: `vox_palette_from_voxj_palette` derived the material count from the first
`materials` column and only rejected columns SHORTER than it, so a LONGER later
column was silently truncated, dropping material data on any ragged palette
(not just hostile input) and contradicting the function's own doc. It now
rejects any column whose length differs from the first, in either direction, and
a test covers the longer-non-first orientation the old check missed.

Deferred, hostile-input only, and uniform with the existing load path:
1. `vox_object_from_voxj_decoded_object` computes the runtime-to-edit offset as
   `object.origin[i] - origin[i]`, an `i32` subtraction that can overflow when a
   crafted document sets the object and edit origins about four billion apart.
   This line is copied verbatim from the pre-port code; `check_edit_state`
   already widens to `i64`, so the load path could adopt the same widening.
2. Every wire index reaching an id (`poolRef`, material value-index, layer
   palette ref, root nodes, hierarchy child nodes) is narrowed with `as u32`, so
   a value at or above `2^32` truncates into a possibly-in-range id that
   voxcore's `validate` then accepts. `as u32` is the codebase-wide convention
   for a wire `usize` becoming a `U32Id`, and the root and child-node sites
   predate this change; closing the gap means switching every such site to a
   checked `u32::try_from`, a uniform hardening pass better done on its own than
   folded into the palette port. Neither gap affects a well-formed document,
   since a real producer never emits a billion-span origin or a `2^32` index.

## Phase 5: `voxsmith` color helpers and glTF pipeline

### Chunk boundary: the glTF material-atlas read/bake path, verified scoped

Phase 5's `gltf` feature holds two code-independent subsystems. The read/bake
path (`used_materials`, `bake_atlas`, `material_document`, `material_atlas`, and
the `object_to_material_*` writers) turns a voxcore palette into a glTF material
atlas. The build/voxelize path (`MeshMaterial`, `sample_material`, the mesh maps,
`voxelize_mesh`, and the `from_gltf_bytes` importer) turns a glTF mesh into a
voxcore palette. They couple only through the `gltf` feature flag, not through
code: the read path never names `MeshMaterial`. This chunk ports the read/bake
path. It was verified under `--no-default-features --features gltf,voxj` with the
build path (`internal/mesh`, `convert/voxelize`, `from_gltf_bytes`) and the
still-unported `reduce_palette` temporarily cfg-gated out and then reverted, so
the gating is absent from the staged diff; 51 tests pass. The color helpers
(`object_color_ref`, `cell_color`, `parse_color_hex`, checklist item 2) are
deferred to Phase 6, since their only callers are the Phase 6 converters.

### The shared glTF attribute vocab is a crate-root public module

`gltf_attributes.rs` holds `pub const BASE_COLOR_FACTOR` and the rest of the
glTF metallic-roughness names. It is a top-level, always-compiled, public module
like `color_space`, not behind the `gltf` feature, because the non-`gltf` format
converters (Goxel, MagicaVoxel, Qubicle, Voxel Max) and `vxl` also bind palette
attributes by these names. Being public API, a constant unused in a feature
subset does not warn as dead code.

### `used_materials` reads one layer by id, returning `Option<UsedMaterials>`

The cross-layer merge is gone. `resolve_used_materials(object, layer:
U32Id<BVoxLayer>) -> Option<UsedMaterials>` returns `None` when `layer` is not one
of the object's layers and so names no palette. `UsedMaterials` then carries a
non-optional `palette: U32Id<BVoxPalette>` and a plain `Vec<U32Id<BVoxMaterial>>`,
since a live voxel in a valid layer always samples a real material, plus
`index_of: HashMap<U32Id<BVoxVoxel>, u32>` mapping each voxel to its atlas texel.
The layer is a passed-in id, not a hardcoded index; resolving a `--layer` ordinal
to a layer id lives at the CLI boundary in Phase 7. The `Option` is hoisted out of
the bake machinery: each entry point (`object_to_material_atlas`,
`build_material_document`) consumes it once with `ok_or_else` into an `Err`, so
`bake_atlas_pixels` and its helpers take a plain `&UsedMaterials`. An
unresolvable layer is therefore an error, not the old one-default-texel fallback,
which was a merge-model vestige; real objects always carry a layer and the CLI
resolves `--layer` first.

### `material_index` stays a `u32` texel position

The one non-id integer in this surface is `UsedMaterials::material_index -> u32`,
the dense `0..len` atlas-texel position that `texel_center` indexes and
`mesh_slices` keys on. It is a positional index into the atlas grid, not a pooled
entity, so it is not branded.

### The bake decodes colors and scalars by pool kind

`bake_atlas` reads an attribute as `(pool, value_index)` through
`VoxMain::material_value`, then decodes by the pool's kind: sRGB float components
in `[0, 1]` scale to bytes, linear components re-encode to sRGB through `ty_math`,
`float`/`int` pools give scalars, and a missing attribute falls back to
`default_scalar` (or opaque white for a color). The `default_scalar` table is
retargeted to the glTF names with `metallicFactor` defaulting to `1`.

### The EmissiveColor bake splits emissive into factor and strength

`MaterialBake::EmissiveColor` now reads `emissiveFactor`, an sRGB color defaulting
to black, times `emissiveStrength`, a float defaulting to `0`, multiplied in
linear light, replacing the old base-color-times-scalar form. The
`sample_material` and `mesh_emissive_map` side of the split lives on the build
path and is deferred with it.

### Chunk boundary: the glTF build/voxelize path is one chunk

Phase 5's second chunk ports the build/voxelize path, the counterpart to the
read/bake chunk: `MeshMaterial`, `sample_material`, `mesh_emissive_map`,
`voxelize_mesh`, and the `from_gltf_bytes` importer. These five files couple
tightly: `voxelize_mesh` builds a palette from `MeshMaterial`, `from_gltf_bytes`
constructs `MeshMaterial` and its tests voxelize and read the palette back, so
renaming `MeshMaterial`'s fields breaks all of them at once and they move
together. The chunk was verified under `--no-default-features --features
gltf,voxj`; the still-unported `reduce_palette` straggler (no caller in this
feature set) was temporarily cfg-gated out and reverted, so the gating is absent
from the staged diff; 81 tests pass, clippy clean. A four-dimension adversarial
review (pool/material bookkeeping, color and emissive round-trip, validate and
API contracts, test fidelity) surfaced no confirmed findings. The color helpers
(`object_color_ref`, `cell_color`, `parse_color_hex`, checklist item 2) stay
deferred to Phase 6 with their only callers.

### `MeshMaterial` moves to the glTF vocab; both colors are 8-bit sRGB

`MeshMaterial` drops the old `rgba`/`metallic`/`roughness`/`emissive`/`occlusion`
row for the glTF names: `base_color`, `metallic`, `roughness`, `emissive_factor`,
`emissive_strength`, `occlusion`. The old single `emissive` scalar (a strength
scaling the base color) splits into `emissiveFactor`, a color, and
`emissiveStrength`, a number, matching the format's split. Both colors are stored
as `TySrgbaColor`, the codebase-wide 8-bit sRGB type; `emissive_factor` carries
no alpha in glTF, so its alpha is held opaque and ignored (a 3-component
quantity in a 4-component type, keeping `flat()`, the material key, and the
sampler's `to_srgba()` output uniform with `base_color`). `MeshMaterial` also
loses `cell_values`, `hex`, and `MATERIAL_ATTRIBUTES`: the palette build, now a
value-pool build, moved into `voxelize_mesh`, so `MeshMaterial` is plain data.

### `emissive_factor` is stored sRGB; the strength stays a per-material scalar

`emissive_factor` is stored in the same sRGB form as `base_color`, matching the
read/bake chunk's documented convention (`MaterialBake::EmissiveColor` reads an
sRGB `emissiveFactor`). `from_gltf_bytes` sRGB-encodes glTF's linear
`emissiveFactor` to the stored color, so the bake decodes it back to linear and
multiplies by strength there, round-tripping through the 8-bit atlas texel the
same way base color does. `emissive_strength` defaults to `1` on import: reading
`KHR_materials_emissive_strength` is deferred, so a plain `emissiveFactor` imports
as the emissive color at unit strength rather than being lost. In `sample_material`
the emissive TEXTURE overrides only the color (`CellAccum.emissive` became a
three-component linear sum, meaned then `to_srgba`); the strength is a
per-material scalar the texture does not carry, so it stays the material's flat
value. `mesh_emissive_map`'s `emissive()` returns the linear color instead of
collapsing to the strongest channel.

### `voxelize_mesh` builds six per-attribute pools, deduped, clamped to validate

`build_palette` takes `&mut VoxMain`, merges near-identical materials into a
distinct list (the `MaterialKey` now keys on both 8-bit colors and the scalar bit
patterns, alpha excluded from the emissive key since it is always opaque), then
builds one deduplicated value pool per attribute and binds the six attributes in
a fixed order: `baseColorFactor`, `metallicFactor`, `roughnessFactor`,
`emissiveFactor`, `emissiveStrength`, `occlusionStrength`. Each distinct material
adds one value-index per binding in that order. Color pools dedup by 8-bit bytes
and store float components in `[0, 1]` (`Srgba` for base color, `Srgb` for the
alpha-less emissive); scalar pools dedup by bit pattern. `metallicFactor`,
`roughnessFactor`, and `occlusionStrength` are `Float` `0..1` and `emissiveStrength`
is `Float` `0..none`, per the Q5 bounds table; colors carry no bounds. The bounded
scalars are `clamp`ed to their pool range at extraction, so a malformed glTF (for
example an `occlusionTexture.strength` above 1 driving `1 + strength * (red - 1)`
negative) still assembles a state that `VoxMain::validate` accepts, where the old
inline model stored the raw value unvalidated. For a well-formed glTF the clamp is
a no-op, so the change only guards hostile input. All bindings precede any
material, so no material carries `add_binding`'s back-fill placeholder index. The
object gets one layer (`add_layer` replacing `add_palette_ref`) whose samples are
`BVoxMaterial` ids.

### The tests read voxel values through the layer/material/binding/pool API

The `from_gltf_bytes` tests read a voxel's attribute by resolving its first layer
and material, its palette's binding by attribute name, and
`VoxMain::material_value`, then decoding by pool kind: `voxel_hex` reads the
`Srgba` base color to a hex string, `voxel_number` reads a `Float` pool, and the
emissive test reads the `Srgb` `emissiveFactor` color directly and asserts its
green component (about `0.737`, the sRGB round trip of a linear `0.503`) plus a
unit `emissiveStrength`. The attribute-list assertion pins the six glTF binding
names in order. `palette.cell_count`/`iter_cells`/`iter_attributes`/`cell_value`
and `object.voxel_cell`/`iter_palette_refs` gave way to `material_count`/
`iter_materials`/`iter_bindings` and `voxel_material`/`iter_layers`.

## Phase 6: `voxsmith` goxl, mvox, qbcl, vmax converters

### First chunk: the unconditional `reduce_palette` straggler, ported before the converters

`reduce_palette` is compiled unconditionally (a crate-root `mod`, not behind any
codec feature), so it blocks every scoped build the converter chunks need; phases
4 and 5 worked around it by temporarily cfg-gating it out and reverting. Porting
it first is the smallest coherent chunk that compiles and passes on its own: it
is self-contained, carries its own color reading rather than the `_color`-gated
helpers (checklist item 2, still deferred), and verifies under
`--no-default-features --features voxj`, where it was the sole remaining error and
the voxj seam is already green. Doing it first retires the temporary-gate hack for
every later converter chunk. It is not a listed Phase 6 item, but it is a real
prerequisite the checklist omits.

### `reduce_palette` clusters materials by `baseColorFactor`, merges through `remove_material`

The port maps the cell-grid palette reduction onto the pool/material model one
concept at a time, preserving behavior (both known-pattern dither tests pass
byte-for-byte):

1. It reduces the material count, not a cell count; `max_cells` becomes
   `max_materials` and the cell-to-material rename runs through the file
   (`Point.material`, `material_populations`, `dither_layer`), matching Q7 and
   voxcore's `BVoxMaterial`. The public signature change is safe because vxl, its
   only caller, does not compile until Phase 7 and adopts the new name there.
2. The clustering attribute moves from an inline `rgba` hex string to
   `baseColorFactor` resolved through `binding_by_attribute` and
   `VoxMain::material_value`. A new `material_color` decodes the bound pool to
   sRGB bytes exactly as the atlas bake does, sRGB components to bytes and linear
   re-encoded through `ty_math`, returning `None` for a non-color or absent value
   so a colorless material is left untouched, mirroring the old colorless-cell
   skip. Clustering in sRGB bytes keeps the reduction math and the `Rgb`-space
   dither tests identical to the old inline-hex path.
3. Merging a cluster onto its representative is
   `VoxMain::remove_material(palette, dropped, representative)` then one `gc`,
   replacing the old `remove_cell`; the model's repaint-then-drop is what the
   cell removal did.
4. Dithering rewrites one layer's sample per voxel via `retain_voxel`, reading
   `voxel_material`/`iter_layers` in place of `voxel_cell`/`iter_palette_refs`;
   `BVoxPaletteRef` becomes `BVoxLayer`.

The in-file tests rebuild their fixtures to pools, bindings, materials, and one
layer: `baseColorFactor` is an `Srgba` pool decoded from the test hex strings
(byte / 255), the `tag` scalar is an unbounded `float` pool with one distinct
value per material, and each material draws one value-index per binding. A
material's color reads back to hex through the same byte round trip, so the golden
hex assertions are unchanged.

### Second chunk: the shared `_color` helpers plus goxl

The goxl chunk lands checklist Phase 6 item 1 and the deferred Phase 5 item 2
together, because goxl is the first `_color` consumer and the helpers compile and
are exercised under `--no-default-features --features goxl` (mvox, qbcl, and vmax
modules are off in that feature set). It was verified there; 15 tests pass,
including the three byte-exact round-trips, clippy clean. The still-broken mvox,
qbcl, and vmax converters keep the default build red until their own chunks land.

The three shared color helpers move to the pool/material model with new
signatures that every converter will adopt in its chunk:

1. `object_color_ref(state, object)` returns `(layer, palette, binding)` ids for
   the first layer whose palette binds `baseColorFactor`, replacing the old
   `(reference, &palette, attribute)` that preferred `rgba` over `rgb`. Returning
   ids, not a borrowed `&VoxPalette`, drops the lifetime entanglement, since the
   color now resolves through `state`, not the palette alone.
2. `cell_color` gains a `state` parameter and resolves a voxel's color through
   `voxel_material` then `VoxMain::material_value` then the pool decode, defaulting
   to transparent black. It no longer reads an inline hex `VoxValue`.
3. `parse_color_hex` is renamed to `pool_color` (its file too, per the
   one-item-per-file rule) and generalized from hex-string parsing to decoding a
   resolved `(pool, index)` by the pool's kind: sRGB components straight to bytes,
   linear re-encoded to sRGB through `ty_math`, `None` for a non-color or
   out-of-range value. This is the same decode `bake_atlas` and reduce_palette use,
   duplicated across the three feature scopes rather than hoisted to an
   always-compiled home, matching the faithful-port tradeoff already taken for
   reduce_palette.

goxl's own paths follow: `from_goxl_file` builds one shared `srgba` pool of the
distinct block colors (float components, byte / 255) bound to `baseColorFactor`,
one material per color, and each 16-cube object references it on one layer with
solid voxels sampling their color material. `to_goxl_file` reads colors back
through `object_color_ref` plus `cell_color` in both the ext-driven
`block_from_object` (which now takes `state` and drops its bespoke `voxel_color`,
`attribute_id`, and `parse_rgba` for the shared helpers) and the synthesized
`emit_object`. The byte-exact `.gox` round-trip holds because the hex-to-float
(byte / 255) then float-to-byte (round of component times 255) trip is exact for a
byte-valued component.

### Follow-up: the sRGB-float quantization moves to `ty_math::TyRgbaColor::to_srgba`

`pool_color` stays in voxsmith, since it matches on voxcore's `VoxValuePool` and
voxcore depends on ty_math, not the reverse, so a `VoxValuePool` match cannot live
in ty_math. But the primitive it, `reduce_palette`, and `bake_atlas` each
hand-rolled, sRGB float `[0, 1]` to an 8-bit byte (clamp, times 255, round), is
ty_math's. ty_math already models an sRGB float color as `TyRgbaColorF64`, which is
what `TySrgbaColor::to_rgba` returns, and already encodes linear floats to bytes
via `TyLinearRgbaColorF64::to_srgba`, but the gamma-encoded float-to-byte inverse
of `to_rgba` was missing. Added `TyRgbaColor<f64>::to_srgba() -> TySrgbaColor` to
fill that gap and routed all three decoders through it, deleting each private
`srgb_bytes`; no new type was needed. Landed as its own commit after the goxl
port. The voxj seam's float-to-hex encoder and the glTF importer's byte encoders
were left alone; they are different operations, not this same decode.

### Third chunk: the Qubicle family maps three channels to a shared `Srgb` pool

The `qb`, `qbt`, and `qbcl` converters are three near-duplicate file pairs that
all built the old inline `rgb` cell palette (one `#RRGGBB` hex `VoxValue::Text`
per cell) the same way. The port applies one uniform shape to all six files,
mirroring the goxl port but with a three-channel color kind, since a Qubicle
voxel stores no alpha:

1. Each `from_*_file`'s `build_palette` gains a `&mut VoxMain`, collects the
   distinct `[u8; 3]` block colors in first-seen order exactly as before, then
   builds one shared `VoxValuePool::Srgb` of those colors as float components in
   `[0, 1]` (byte / 255) added to the runtime state, binds it to
   `baseColorFactor`, and adds one material per color. It returns the palette and
   a `[u8; 3] -> U32Id<BVoxMaterial>` map, replacing the old
   `[u8; 3] -> U32Id<BVoxPaletteCell>` map. This settles the checklist's alpha
   question: a three-channel source is canonical `srgb`, not `srgba` with a
   synthesized alpha, so the pool is the narrower kind and no alpha is invented on
   read.
2. Each `build_object` / `build_node` calls `add_layer` + `retain_voxel` with the
   resolved material id instead of `add_palette_ref` + the old cell id; the grid
   sizing, storage-order loops, and oversized-grid error are untouched.
3. Each `to_*_file` reads a voxel's color through the shared `object_color_ref` +
   `cell_color` helpers (the same ones goxl adopted) and drops the fourth
   component with `let [r, g, b, _] = ...`, since a Qubicle voxel is alpha-less.
   This deletes each file's bespoke `voxel_color`, `attribute_id`, and `parse_rgb`
   plus the `hex` writer. Because `cell_color` resolves color through `state`, the
   writers no longer thread a borrowed `&VoxPalette`: `to_qb` drops the `palette`
   local, and `to_qbt` / `to_qbcl` drop the `palette` parameter from
   `rebuild_node` / `rebuild_children` / `matrix_from_object` and pass `state`
   instead. `to_qbcl`'s synthesis path (`synthesize_matrix`) moves to the new
   `cell_color(state, object, voxel, layer, palette, binding)` signature the same
   way.

The byte-exact `.qb` / `.qbt` / `.qbcl` round-trips hold because the color makes a
byte -> float (byte / 255) -> byte (`round(component * 255)`) trip that is exact
for a byte-valued component, the same argument the goxl port relies on;
`pool_color` returns opaque alpha for an `Srgb` pool, which the writers discard.
Only `from_qbcl_file` carries an in-file voxcore-level fixture (`source_state`);
it was rebuilt to the pool / binding / material / layer shape with a local `srgb`
hex-to-float helper, and the format-level `qb` / `qbt` fixtures needed no change.
Verified under `--no-default-features --features qbcl`; 26 tests pass. On default
features the crate still fails to build only in the unported `mvox` and `vmax`
modules, the remaining Phase 6 items; nothing in `qbcl` errors.

### Fourth chunk: MagicaVoxel folds its 256-slot palette into one material per color index

The MagicaVoxel converter is its own chunk, verified under `--no-default-features
--features mvox` where it is the sole scoped codec; 18 tests pass, clippy clean.
On default features the crate now fails to build only in the unported `vmax`
module and its `vmax`-gated `internal/grid.rs` helper, the last Phase 6 item.

The load-bearing shape is that a MagicaVoxel voxel references a palette color
index `0..=255`, and a `MATL` material's `id` IS a color index. So the port
builds exactly 256 materials, one per color index, material index == color index.
A voxel then samples that material directly (its sample is the color index it
always was), and one material index resolves to both the color and, when present,
the MagicaVoxel material properties, the same one-material-per-combination fold Q2
prescribes. `build_object` adds one layer over the shared palette and
`retain_voxel`s `BVoxMaterial` = the voxel's color index; `model_from_object`
reads it back through `voxel_material` on the first layer and narrows to a `u8`.

`baseColorFactor` binds a deduplicated `Srgba` pool of the palette colors (byte /
255), each of the 256 materials drawing the color at its own index; deduping
matters because a typical file leaves most of the 256 slots on the reserved empty
color. When the file declares materials, `type` binds a `String` pool and
`weight`, `rough`, `spec`, `ior`, `att`, `flux` bind `Float` pools, in that fixed
binding order. The scalars are custom MagicaVoxel attributes the glTF bounds table
does not cover, so their pools are unbounded (`min`/`max` = `none`); in
particular `ior` is unbounded here even though it shares glTF's `ior` name, since
MagicaVoxel stores a value like `0.3` that glTF's `1..none` would reject, and
validation checks a pool against its own declared bounds, not a name table. All
bindings are added before any material, so no material carries `add_binding`'s
back-fill placeholder. A small `intern` helper deduplicates each per-slot value
list into a pool and returns the per-material value-index, keyed by color bytes,
float bit pattern, or the token string.

### The exact optional material fields move into the ext; pools carry a default

A value pool cannot hold null, but the old inline palette stored `Null` for an
absent material field and write-back read it straight back, so an absent `_spec`
round-tripped as absent. Under the pool model an absent field must take a real
default (`0.0` for a scalar, `_diffuse` for `type`, MagicaVoxel's default shading
model), which alone would turn `spec: None` into `spec: Some(0.0)` and break the
byte-exact round-trip the faithful port requires. Comparing a pool value against
the default to recover `None` is wrong, since a material may legitimately hold the
default value, and for `type` the presence of a `_material` key differs on the
wire from its absence. So the exact optionals moved into the ext:
`MagicaVoxelMaterial` grew `material_type: Option<String>` (the token) and
`weight`/`rough`/`spec`/`ior`/`att`/`flux: Option<f32>`, each
`skip_serializing_if` so an absent field stays absent in the ext too.
`build_materials` now reads every material field from the ext, dropping the
palette entirely, so an absent field round-trips as absent; the palette pools
carry only the default-substituted neutral copy for a cross-format or `vxl`
consumer. This mirrors the converter's existing design, where the hierarchy nodes
are the neutral view and the ext holds the exact per-node provenance; materials
now follow the same split. `colors_from_palette` is the one write-back path still
reading the palette, resolving material index `i`'s `baseColorFactor` through
`material_value` + `pool_color` back to slot `i`.

### Adversarial review: a non-finite material scalar is defaulted in the pool

A four-lens adversarial review of the diff confirmed one real defect (found by
every lens): the mvox codec's `parse_f32` accepts a non-finite scalar such as
`_ior inf`, `_flux nan`, or an overflowing `_weight 1e40`, so a decodable file
can carry a non-finite material scalar. Folded verbatim into a `Float` pool that
`VoxMain::validate` requires to be finite, it made `from_mvox_file` reject a file
the codec loaded, a spurious rejection the old inline path did not have. The fix
is the Phase 5 voxelizer's pattern: `build_palette` `.filter(|v| v.is_finite())`
before the scalar enters the pool, defaulting a non-finite value to `0.0` in the
neutral pool. The ext keeps the exact `Option<f32>`, and `to_vox_value` builds a
`VoxValue::Number(f64)` that holds the non-finite value without a JSON round trip
(the mvox byte path never serializes the ext to JSON), so `build_materials` reads
it back exactly and the material still round-trips. `round_trips_a_non_finite_
material_scalar` covers it. A NaN scalar loads the same way but cannot be tested
by value equality, since `NaN != NaN`, so the test uses `f32::INFINITY`.

The review's other confirmed items were the same finding re-reported per lens.
Two findings were refuted and left as-is, consistent with the Phase 4 seam's
deferred hostile-input gaps: `model_from_object` narrows a sampled material id
with `as u32 as u8`, which only truncates for a hand-built foreign state that
carries a `magica-voxel` ext and samples a material id at or above 256, a state
the converter itself never produces (`build_palette` makes exactly 256 materials
and `build_object` only ever samples those); and `material_type_from_token`
canonicalizes a reserved token, unreachable from a parsed file.

### Fifth chunk: Voxel Max folds its two palettes with a full color pool and one-per-material scalar pools

Voxel Max is the load-bearing fold case: a voxel carries an independent
`color_idx` (1-based, into a 255-color palette) and `material_idx` (into a
separate material list), which the old model held as two palettes on one object.
Q2 folds them into one palette with one material per distinct
color-and-material combination the voxels use. The two indices are independent,
so the fold is a two-dimensional cross product, unlike MagicaVoxel where the
material index *is* the color index.

The fold is designed so both original indices round-trip as pool value-indices,
avoiding any per-folded-material provenance:

1. The color pool is the object's full color table (255 colors) as `Srgba`,
   undeduped and in order, so a material's `baseColorFactor` value-index is
   exactly `color_idx - 1`, and the whole table (unused colors included)
   reconstructs by iterating the pool. `color_cells` returns the png pixels
   (terminator dropped), else the sidecar `colors`, else 255 placeholders.
2. The material scalar pools hold one value per vmax material, undeduped and in
   order, so every material binding's value-index of a folded material is that
   material's `material_idx`. Bindings, in fixed order: `metallicFactor` (mc),
   `roughnessFactor` (rc), `emissiveStrength` (sic, the scalar half of the
   emissive split), `shadows` (sh, a `Bool` pool), and when any material carries
   an `md` block `ior`, `transmissionFactor`, `absorption`. `shadows` and
   `absorption` have no glTF name and stay custom, in `voxel_max_attributes.rs`.
3. The scalar pools are UNBOUNDED, mirroring the MagicaVoxel converter: a Voxel
   Max coefficient need not sit in the glTF range its name implies, and validation
   checks a pool against its own bounds, not a name table. A non-finite
   coefficient (a crafted bplist real) is defaulted to `0.0` in the pool so
   validation accepts it; the exact value rides in the ext.

The exact material list moves into the ext, the same split the MagicaVoxel
converter uses for its optionals: `VoxelMaxPalette` grows `materials:
Vec<VoxelMaxMaterial>` (mc/rc/sic/sh and an optional `VoxelMaxMaterialDispersion`),
aligned by `material_idx`. This is needed because a pool cannot hold the
absent-dispersion case, and because two materials with identical coefficients
must stay distinct on write-back. The `mi` token is re-derived as `(slot + 1)`
and `tc` is dropped, both matching the old writer. On write-back the material
list comes from the ext (byte-exact); the pools are the finite-defaulted neutral
copy for a cross-format or `vxl` consumer.

The write path (`write_vmax`) resolves materials two ways, branching on whether
the ext carries an exact list for the palette (`material_plan`):

- Voxel-Max-origin (ext has materials): `material_idx` is the value-index into
  the first material binding, which equals the original `material_idx` because
  the pools are one-per-material; the material list is the ext's, so unused
  materials and absent dispersion survive.
- Synthesized, no ext (a state loaded from another format, e.g. the
  material-palette synthesis test or a glTF-to-vmax conversion):
  `derive_materials` builds one `VMaxMaterial` per distinct SIGNATURE of the
  material bindings' value-indices, reading each coefficient from its pool, and
  `material_idx` is the signature's first-seen index. This is correct for
  deduped per-attribute pools where no single value-index is a coherent material
  index, and reduces to the identity for the one-per-material case.

`color_index` recovers `color_idx = baseColorFactor value-index + 1`, rejecting a
value-index at or past 255 with the same over-budget error the old path raised.
`color_palette_colors` reads the `baseColorFactor` pool through `pool_color`, so
an `Srgb` pool (a Qubicle-to-vmax state) decodes with opaque alpha just as an
`Srgba` one does.

Palettes are no longer deduped across objects by source filename, because the
folded materials depend on which combinations each object's voxels use. Each
distinct object builds its own folded palette (instances still share via their
shared object), so `ext.palettes` stays aligned by palette id: `build_object`
adds exactly one palette and pushes exactly one provenance entry, and an empty
object adds neither. A scene of several distinct objects that shared one
`palette.png` therefore round-trips to per-object palette files rather than the
one shared file; this is an accepted byte change of the fold (the byte-exact
tests use single objects), not a fidelity loss, since each object reconstructs
its own colors and materials.

`vmax` now enables the `_color` feature (its write path samples colors in
production, not just tests), so the `all(feature = "vmax", test)` special case on
the shared color helpers in `internal/mod.rs` collapses to plain `feature =
"_color"`. `internal/grid.rs` (`tighten`, vmax-only) is ported to layers and
`BVoxMaterial` samples. Verified under `--no-default-features --features vmax`
and, since every default-feature converter is now ported, under default features:
`cargo test -p voxsmith` passes and clippy is clean with and without `gltf`.

### Adversarial review: the ext gains `tc`, the derive path gains a material cap

A four-lens adversarial review of the port (round-trip fidelity, the two material
paths, color invariants, edge/validation) confirmed two distinct defects, both
low severity and both faithful to the old code, and both fixed here:

1. `VMaxMaterial.tc` (a transmission color the codec documents on some real
   documents, such as MagicaVoxel exports) had no home in the ext copy, so a
   material carrying `tc` lost it on write-back. Since the ext is meant to be the
   exact material, `VoxelMaxMaterial` grew `transmission_color: Option<f64>`,
   populated on read and read back on write. The old writer also hardcoded `tc:
   None`, so this closes a pre-existing byte-exactness gap rather than a
   regression. `round_trips_rich_materials` covers it alongside dispersion
   optionality and a non-finite `mc`.
2. `derive_materials` assigned `material_idx = signatures.len() as u8`, so a
   synthesized cross-format state (no ext) with 257 or more distinct material
   signatures, reachable from a voxelized textured mesh, wrapped the 257th index
   to 0 and silently mis-assigned voxels. A Voxel Max voxel indexes its material
   with a single byte, so this now errors at `MAX_MATERIALS = 256`, symmetric
   with the over-budget error `color_index` raises past 255; `material_plan`
   became fallible and the one call site propagates it.
   `errors_when_derived_materials_exceed_the_byte_budget` covers it. The
   Voxel-Max-origin path needs no cap: its value-indices come from a real
   document's material list, always within a byte.

The review's other reported items were the same wrap re-reported per lens, or
were refuted as unreachable from a well-formed document or a state the converter
itself produces.

## Phase 7: `vxl`

### First chunk: the read/inspection commands, because vxl cannot compile in halves

Unlike the additive early phases, `vxl` is a single lib+bin crate with no
per-command feature gates, so a partial port leaves compile errors that block
the whole crate and every test. The hard compile errors (the removed voxcore
methods `cell_count`, `iter_cells`, `cell_value`, `iter_attributes`,
`palette_ref_count`, `iter_palette_refs`, and voxsmith's new
`MaterialMeshRequest.layer` field) fall in exactly six files:
`implementation/{attribute_names,info,hierarchy_show,palette_list,palette_show,
mesh}.rs`. That set is the read/inspection command cluster plus a one-field mesh
fix, so it is both the smallest compiling unit and a coherent reviewable chunk.
The mesh vocab and `AttributeType` rework (items 3-5), the `--color-format` flag
(item 6), and the `max_palette_cells`/doc updates (item 7) live in files that
still compile as-is and are deferred to later chunks. Verified: the whole
workspace compiles clean for the first time since the port began, clippy
`-D warnings` passes workspace-wide, and 138 `vxl` tests pass.

### `palette show` classifies by pool kind and precomputes text, JSON, and swatch

The rework replaces the old inline-`VoxValue` type sniffing (a string is a
color, a number a scalar) with classification by the bound pool's kind. Each
attribute resolves its binding once (`binding_by_attribute` ->
`state.value_pool(binding.pool)`), reads the pool's variant into a small `Kind`
(color with space and component count, number, or other), then samples each
material's value-index. A `Sample` precomputes three things so every layout
renders uniformly: the display `text`, the native `json` value, and a `Swatch`
(a true-color block for a whole color, a grayscale block for a scalar or color
component, or none). This collapsed the old sixteen-arm format-by-sample-type
`render_cell` to a swatch-directed match and made `abuts` key off "every sample
carries a swatch" rather than "no sample is a raw fallback," preserving the
bool-spacing behavior.

Colors decode the same way voxsmith's `pool_color` does, replicated in vxl
because that helper is `pub(crate)` in voxsmith and unreachable here (the same
faithful-port duplication the converters already accept): sRGB components map
straight to bytes via `TyRgbaColorF64::to_srgba`, linear components re-encode
through `TyLinearRgbaColorF64::to_srgba`, and a three-component color takes
opaque alpha. Rendering then splits on space and encoding per the deferred-color
policy: an sRGB color prints `#RRGGBB` or `#RRGGBBAA` hex and byte components, a
linear color prints its natural-range float components (which no hex can hold),
`.a` on a three-component color errors, and `float`/`int`/`bool`/`string`/`json`
pools each render their native value rather than collapsing to null. Because
every existing fixture is sRGB, all the old hex/byte goldens are preserved
byte-for-byte; new tests cover the three-component, linear, int, and json-array
cases.

### Terminology: material and layer where the model changed, `palettes` header kept

`info` renames the palette table's `Cells` column and JSON `cells` key to
`Materials`/`materials` (now `material_count`), and the object table's
`Palettes` column and `palettes` key to `Layers`/`layers` (now `layer_count`,
since two layers may share a palette so it is a layer count, not a distinct
palette count). `palette list` renames the `--show-cells` flag to
`--show-materials`, the `cells` column and JSON key to `materials`, and the
`cellCount` leaf to `materialCount`. `hierarchy show`'s per-object subtree and
its flag are renamed from `palettes`/`--show-palettes` to `layers`/
`--show-layers`; it now enumerates the object's layers (`iter_layers`, one child
per layer as the old `iter_palette_refs` gave one per ref), each labeled by its
palette index and `{materials: <count>}`. `layers` is the honest name for the
new model: the subtree is per-layer, and two layers may share a palette, so
`palettes` would wrongly imply a distinct-palette set. (`palette list`'s own
`palettes` header is unchanged: it genuinely lists the document's palettes, not
an object's layers.)

### Fixtures keep the old attribute names; the glTF-vocab flip is item 4's

The rebuilt `info`/`palette_list`/`palette_show`/`hierarchy_show` fixtures move
onto value pools plus bindings plus materials plus layers but keep the old
attribute names (`rgba`, `metallic`, `shadows`) and the same color/scalar
values, so the only golden churn is the terminology rename, not an attribute
revocabulary. `palette show` classifies by pool kind, not by attribute name, so
the name is a pure label and `rgba` is valid test data; flipping fixtures to the
glTF names (`baseColorFactor`, `metallicFactor`) is deferred to item 4 with the
production mesh/texture vocab. Colors are stored as byte/255 float components (a
shared `srgba_pool` test helper), so a color still round-trips its old hex
exactly through the `pool_color` quantization.

### Minimal `mesh` fix: default the required layer to the object's first

`MaterialMeshRequest` gained a required `layer: U32Id<BVoxLayer>` on the
voxsmith side. `mesh_object` resolves it inside the function
(`object.iter_layers().next()`, per Q2a the first layer) rather than adding a
`--layer` CLI parameter, so `commands/mesh.rs` and the `Dependencies` trait are
untouched and the CLI layer selector (item 5) stays deferred. The resolution
runs only on the material path, after the pure-geometry early return, and errors
with a clear message when the object carries no layers; that matches voxsmith's
new contract, which already dropped the old merge-model one-default-texel
fallback in favor of erroring on an unresolvable layer.
