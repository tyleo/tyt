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

## voxj round-trip tests gate on the serde feature

2026-07-19, phase 3. The crate's serde support is optional, so the new
round-trip tests sit in `voxj_file.rs` under
`#[cfg(all(test, feature = "serde"))]` with `serde_json` as a plain
dev-dependency, not a self-referential dev-dependency forcing the feature
on. `cargo test -p voxj --features serde` runs them; a workspace-wide
`cargo test` runs them too because voxj-codec enables the feature through
unification.
