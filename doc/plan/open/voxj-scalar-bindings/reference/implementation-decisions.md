# voxj scalar bindings implementation decisions

Code-level decisions made while executing the
[checklist](../checklist.md), recorded as they land. The plan-level decisions
and their rationale live in the [README](../README.md#decisions); this log is
for the finer implementation choices a reviewer of the Rust would want
explained, for example the final Rust binding type names, how voxcore stores
scalar bindings, and whether the glTF converter emits them.

## voxcore `VoxPalette` target surface

Owner, 2026-07-15, ahead of phase 5. The binding namespace split means
`VoxPalette` (`projects/utilities/voxcore/src/vox_palette.rs`) grows from
today's `binding_ids` / `bindings` / `material_ids` / `materials` /
`by_attribute` to:

1. `array_binding_ids`
2. `scalar_binding_ids`
3. `material_ids`
4. `materials`
5. `array_binding_by_property`
6. `scalar_binding_by_property`
7. `binding_by_property`, a `HashMap` from property `String` to an
   array-or-scalar-tagged binding id

Materials stay column-major over the array-binding ids only. The map names
follow the format-wide attribute-to-property rename (README decision 11,
2026-07-16); the owner originally specified them as `*_by_attribute`.

The 2026-07-17 wire rename (`arrayProperties` / `scalarProperties`, `name` /
`valuePool` / `valueIndex`; README decision 12) postdates this surface;
whether the Rust names follow (`array_property_ids`, `property_by_name`, and
so on) settles in phase 5.

With `materials` row-major on the wire (README decision 14, 2026-07-17),
voxcore's per-material storage and the wire share one orientation, so the
phase 6 seam maps material rows one-to-one.

## voxj Rust property type names

2026-07-19, phase 3. `VoxjPaletteBinding` is replaced by `VoxjArrayProperty`
(`name` / `value_pool`) and the new `VoxjScalarProperty` (`name` /
`value_pool` / `value_index`). The wire's plain-named fields (README
decision 12) make an entry the property itself, so the Rust types drop the
binding noun too, in files `voxj_array_property.rs` /
`voxj_scalar_property.rs`.

## voxj keeps raw indices, not branded ids

Owner, 2026-07-19, phase 3. Every cross-reference in `voxj` stays a plain
`usize` array index: `layers`, `child_nodes`, `child_objects`, `root_nodes`,
`value_pool`, `value_index`, and the `materials` cells. The crate is the
unvalidated wire mirror, and a `branded_id` id would imply a checked
reference a freshly parsed file does not have; the branded layer is voxcore,
where `U32Id<BVoxPalette>` and the `IdField` layer storage already model
layer-to-palette references.

## voxj-codec derives channel arity from material counts alone

2026-07-19, phase 4. `voxj_palette_material_counts` keeps its
one-count-per-layer signature (now trivially `materials.len()` per palette),
and `encode_voxj_object` / `decode_voxj_object` derive the sampled layers
from it by filtering counts above zero into a `channel_counts` list: the
channel arity and the packed bit-width source. The codec never rereads
palettes, and `check_geometry` uses the same per-layer counts to map a
channel back to its layer and palette for error messages.
`VoxjDecodedObject.samples` rows hold one entry per sampled layer, in
`layers` order with unsampled layers skipped, mirroring the wire's channel
order.

## voxj-codec lands in two commits

2026-07-19, phase 4 refinement. Commit 1 adapts the crate to the new wire
model: renames, row-major `M`, sampled-layer channels, and `check_palettes`
rewritten to the array-property and row rules, with scalar properties parsed
and carried but not content-checked. Commit 2 adds the scalar-property
checks (union name uniqueness, `valuePool` and `valueIndex` ranges) with
their failure fixtures. Each commit compiles and tests green on its own; the
split keeps the mechanical migration and the new validation semantics
separately reviewable.

## Scalar-property faults report under the existing palettes check

2026-07-19, phase 4. The scalar checks (empty name, cross-list duplicate,
`valuePool` and `valueIndex` range) report under the existing `palettes`
check rather than a new named check: they are palette-shape rules like their
array-side counterparts. The public thirteen-check surface is unchanged, so
`vxl validate` gains no new check name from voxj-codec.

## The sample-channel layout is one internal type

2026-07-19, phase 4 cleanup. The sampled-iff-`M > 0` rule appeared three
times (encoder, decoder, geometry check), each deriving channel arity, bit
widths, or the channel-to-layer mapping ad hoc from the raw material counts.
`SampleChannels` (`internal/sample_channels.rs`) states the concept once:
built from the per-layer counts, it carries each channel's layer and
material count, and all three sites consume it. `MAX_HILBERT_BITS` likewise
moved to one shared internal constant. Both stay crate-internal; the public
API still speaks plain material counts, and phase 6 can promote the type if
the voxsmith seam wants it.

## voxcore lands in three commits

2026-07-19, phase 5 refinement. Commit 1 renames the palette surface to the
spec's property vocabulary with no behavior change. Commit 2 adds
scalar-property storage, the state maintenance that keeps it consistent
(gc, prune, reorder, relabel), and the new validate rules. Commit 3 derives
layer sampledness from palette material counts and scopes the live-voxel
sample checks to sampled layers. Each compiles and tests green in voxcore
alone; voxsmith and vxl stay red until phase 6, as the plan expects.

## voxcore property naming follows the wire

2026-07-19, phase 5 commit 1. The naming question the owner's 2026-07-15
surface left open settles on the property vocabulary: `VoxArrayProperty`
(brand `BVoxArrayProperty`) replaces `VoxPaletteBinding`, its key field is
`name`, and the palette storage and maps are `array_property_ids` /
`array_properties` / `array_property_by_name`, with the scalar side to
mirror them (`scalar_property_*`, plus the combined `property_by_name` map)
in commit 2. The binding noun drops for the reason it dropped in voxj
(README decision 12): an entry is the property itself. The pool field stays
`pool`, not the wire's `valuePool`: voxcore references carry the target in
their type (`U32Id<BVoxValuePool>`), and the crate already says plain
`pool` throughout. Materials keep voxcore's per-material storage, now
described as rows to match the row-major wire (README decision 14).

## Pool values are referenced by branded value ids

2026-07-19, phase 5 commit 1, owner review. A bare `u32` cell in the
materials rows did not say what it indexes. Pools now store their values
as `IdVec<BVoxPoolValue, T>` and every reference to one is a
`U32Id<BVoxPoolValue>` called a value id: the materials cells,
`add_material`, `value_id`, `material_value`, the new `contains_value`
range check, `retain_values`, `reorder_value_pool`, and the prune and
reorder remap tables (`IdVec<BVoxPoolValue, U32Id<BVoxPoolValue>>`, old
value id to new). With the branded type the index noun went too (owner):
voxcore says value id (`value_id`, `remap_pool_value_ids`) while the
wire keeps `valueIndex` and its value-index prose; the voxj seam
translates in phase 6. The brand documents the reference without a
liveness claim: `IdVec` is dense positional storage, unlike the
`IdStruct`-minted entity ids, so pruning and reordering still rewrite
value ids in place. The brand is `BVoxPoolValue` because `BVoxValue`
would read as pairing with the `VoxValue` ext type. Errors keep raw
`u32` listing indices. Commit 2's scalar property carries the same
`value_id` type and name.

## The combined name map is keyed by an arity-tagged property id

2026-07-19, phase 5 commit 2. The owner's 2026-07-15 surface asked for three
name maps; the split maps return plain branded ids, and the combined
`property_by_name` returns `VoxPropertyId`, a public
`Array(U32Id<BVoxArrayProperty>) | Scalar(U32Id<BVoxScalarProperty>)` enum
in `vox_property_id.rs`. All three maps share the maintenance story the
array map already had: inserted on add, removed on remove only where the
entry still points at the removed id, rebuilt by palette gc.

## Scalar validate faults report as scalar-property errors

2026-07-19, phase 5 commit 2. Three new `Error` variants mirror the array
side: `ScalarPropertyPool`, `DuplicateScalarPropertyName`, and
`ScalarPropertyValue` (the pinned `value_id` out of pool range, the analog
of `MaterialValue`). `validate` checks a palette's array properties before
its scalar properties, so a cross-list duplicate name reports as the scalar
property's fault; a duplicate purely among array properties keeps reporting
`DuplicateArrayPropertyName`. The resolution helper beside `material_value`
is `scalar_property_value(palette, scalar_property)`, returning the same
`(&VoxValuePool, U32Id<BVoxPoolValue>)` pair.

## Sampledness lives on VoxMain, sample cells stay filler

2026-07-19, phase 5 commit 3. Sampledness needs the palette store, so the
view lives on `VoxMain`: `layer_is_sampled(object, layer)` and
`iter_sampled_layers(object)`, the latter yielding `(layer id, palette)` in
layer order with unsampled layers skipped, the wire's channel order. A new
`VoxObject::layer_palette(layer)` accessor backs them. `VoxObject` keeps a
dense sample column for every layer, sampled or not: an unsampled layer's
cells are filler, ignored like non-live voxels'. `validate` checks sample
materials only in sampled layers, and object gc skips translating a layer's
cells when its palette's material remap is empty, the `M = 0` signal, so
filler stays in place. `iter_sampled_layers` treats a dangling-palette
layer as unsampled; `validate` rejects such a state.

## voxsmith lands in three commits

2026-07-19, phase 6 refinement. Commit 1 adapts the whole crate to the
renamed voxcore and voxj APIs and reworks the voxj seam for the new shapes,
with every other converter at column parity. Commit 2 extends
`reduce_palette` and material sampling over scalar contributions and the
canonical layer-override order. Commit 3 settles and implements the glTF
`emissiveStrength` question. Each compiles and tests green in voxsmith
alone; vxl stays red until phase 7, as the plan expects.

## The object seam takes sampledness from opposite sides

2026-07-19, phase 6 commit 1. The two object conversions learn the
sampled-layer channel rule differently. Reading,
`vox_object_from_voxj_decoded_object` gains a `material_counts` parameter,
the same per-layer counts `from_voxj_file` already computes for
`decode_voxj_object`; it expands each sampled-only sample row to a
full-arity `retain_voxel` row, filler material 0 in unsampled layers'
slots. Writing, `voxj_decoded_object_from_vox_object` takes
`(&VoxMain, object id)` instead of `&VoxObject` and reads the channel
order off `VoxMain::iter_sampled_layers`, the accessor phase 5 added for
this seam. Row-major materials drop both palette transposes, and the old
property-less special case unifies: with no array properties every row is
legally empty.

## The seam checks names across both lists; converters stay at parity

2026-07-19, phase 6 commit 1. `vox_palette_from_voxj_palette` extends its
duplicate check over `arrayProperties` union `scalarProperties` (rule 10.2)
and carries scalar properties into voxcore storage both ways; ranges stay
with `VoxMain::validate`. Every other converter compiles at column parity:
value-index cells become branded value ids at the palette boundary
(`voxelize_mesh`'s `PoolColumn.indices` and `write_vmax`'s material
signatures are `U32Id<BVoxPoolValue>` end to end), and no converter emits
scalar properties yet. The internal `(pool, u32)` value plumbing
(`pool_color`, `bake_atlas`) now passes the branded id through. In
fixtures, the raw-json `object()` test helper derives its channel count
from the sample-row arity, not the layer count, so unsampled layers carry
no channel.

## Property resolution is one shared helper; the plan misnamed the sampler

2026-07-20, phase 6 commit 2. The spec's Resolution rule lives once in
`ObjectPropertyRef`: it names a property's winning supplier (`Array`
with layer, palette, and property ids, or `Scalar` with palette and
property), and `resolve_object_property_ref` scans the layers back to
front, taking a scalar property from any layer and an array property
only from a sampled one. Owner review split every pub item into its own
file, so the enum, each resolver, and the `CellColor` alias each get
one. Its `baseColorFactor`
specialization is `base_color_factor_ref`, renamed from
`object_color_ref` in owner review: the property should be obvious from
the name alone. A second owner review moved everything object-invariant
out of the exporters' inner loops, the arity dispatch included:
`resolve_cell_color` (`internal/cell_color.rs`) picks the read once per
object and returns it as a `CellColor` boxed function, the shape
`mesh_slices` already uses for per-voxel keys. An array winner samples
the winning layer through a table with each material's color decoded up
front; a scalar winner returns its pinned color, decoded once. The
loop body in every color exporter (goxl, mvox, qb, qbt, qbcl, vmax) is
`cell_color(voxel)`, with `resolve_cell_color_or_transparent` folding
the no-supplier case into a constant above the loop; only mvox palette
synthesis keeps the `Option`, skipping unsupplied objects. The checklist's "material sampling"
files (`internal/mesh/sample_material`, `mesh_material_maps`) turned out
to sample glTF textures during voxelization; the palette-reading sampler
is the atlas bake, whose `material_attribute`
now resolves through `VoxPalette::property_by_name`, feeding scalar
values (for example a pinned `emissiveStrength`) to every packing
channel, `material_scalar`, and `max_emissive_strength`.

## Color resolution fails loud

2026-07-20, phase 6 commit 2, owner review. The old `cell_color` read
transparent black for every miss, conflating three cases that
`resolve_cell_color` now separates. An unsupplied object stays a call
site policy: `resolve_cell_color_or_transparent` reads transparent
black, and mvox palette synthesis skips the object. A `baseColorFactor`
drawing from a non-color pool is a structurally valid file the export
now rejects with an error naming the object. The invariants `validate`
guarantees (value ids in range, sampled materials in the palette) are
expects. The error path added `Result` plumbing through the goxl and
qbcl synthesis builders and mvox palette synthesis, and the goxl and
qbcl writer docs now list the new error condition.

## reduce_palette treats a scalar color as colorless

2026-07-20, phase 6 commit 2. The reduction clusters on the
`baseColorFactor` array property only: a palette pinning it as a scalar
property has one palette-wide color, so its materials count as colorless
and the reduction no-ops. Voxcore already keeps scalar-referenced pool
values alive through the closing prune and accepts the dither's
full-arity `retain_voxel` rows around an unsampled layer's filler cells,
so both are covered by new tests. Populations keep weighing raw samples
even when a later layer overrides the color: the merge collapses whole
materials, so raw usage stays the honest weight.

## glTF import pins a shared emissiveStrength; export was already scalar-aware

2026-07-20, phase 6 commit 3. The README's open scoping question closes as:
the voxelizer produces scalar properties, the glTF exporter already consumed
them. `build_palette` (`convert/voxelize/voxelize_mesh.rs`) pins
`emissiveStrength` as a scalar property over its one-value pool whenever
every distinct material shares one strength, the one-value-pool contortion
the README's motivation names; mixed strengths keep the per-material column,
since glTF's `KHR_materials_emissive_strength` is per-material. Only
`emissiveStrength` scalarizes: on export it is already palette-scoped (the
one flat KHR factor and the emissive-atlas normalizer), while colors and the
other factors stay columns. The export path needed no change because
`material_attribute` resolves through `property_by_name`, so the existing
KHR round-trip test now covers a scalar strength end to end.

## The vmax material fold falls back to scalar properties

2026-07-20, phase 6 commit 3. `derived_material`
(`internal/vmax/write_vmax.rs`) read every coefficient positionally from the
array-property signature, so a palette pinning `emissiveStrength` would have
silently lost it on a glTF-to-vmax conversion. Its `scalar` and `flag` reads
now fall back to the palette's scalar property of the same name when no
column carries it, and the dispersion gate counts scalar pins of `ior`,
`transmissionFactor`, and `absorption`. The fallback is name-general rather
than emissiveStrength-special: the rule is one sentence (read the column at
the signature, else the palette's pin) and covers hand-authored voxj
palettes. Color reads (`emissiveFactor`, `baseColorFactor`) stay
array-only in this fold; no producer pins them and the color sidecar path
already resolves through `resolve_cell_color`.

## voxj round-trip tests gate on the serde feature

2026-07-19, phase 3. The crate's serde support is optional, so the new
round-trip tests sit in `voxj_file.rs` under
`#[cfg(all(test, feature = "serde"))]` with `serde_json` as a plain
dev-dependency, not a self-referential dev-dependency forcing the feature
on. `cargo test -p voxj --features serde` runs them; a workspace-wide
`cargo test` runs them too because voxj-codec enables the feature through
unification.
