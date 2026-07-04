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
