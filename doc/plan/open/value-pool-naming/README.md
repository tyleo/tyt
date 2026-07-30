# Value-Pool Naming Plan

Status: **open.** No phase has started.

This plan spells "value pool" out across `voxcore`, `voxsmith`, and `vxl`
wherever a bare "pool" means a value pool, in identifiers, doc prose, and
failure messages. The owner's rule: prefer `value_pool` unless a name is fixed
by the voxj spec. The `voxj` and `voxj-codec` crates are already swept; this
plan finishes the workspace. The executable steps live in
[checklist.md](checklist.md).

The naming rule is recorded in the assistant memory `full-entity-names`:
compound entity names are spelled in full everywhere, and the only exception is
a name a wire spec fixes. Nothing in these three crates is wire-mapped, so no
exception applies here; the spec doc
[voxel-json-file-format.md](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md)
itself does not change.

## The two pool entities

The workspace has two unrelated "pool" concepts, and only one is in scope:

- **Value pools** (`VoxValuePool`): the shared, typed value lists palettes
  draw property values from. Every bare "pool" meaning one of these becomes
  `value_pool` / "value pool".
- **Branded-id pools**: the `branded-id` id allocators behind every entity
  listing (palette ids, object ids, node ids, layer ids, voxel ids, property
  ids, material ids, and the value ids *inside* a value pool). These are not
  value pools and must not be renamed to `value_pool`.

Files whose "pool" mentions are mostly or entirely id pools: `vox_gc_remap.rs`
(the palette, object, and node pool relabelings), `vox_runtime_state.rs` and
`vox_palette.rs` (the `* id pool` field docs), `vox_object.rs` (the layer and
voxel pools), `b_vox_voxel.rs`, and the gc section of `vox_main.rs` ("Compact
the palette pool", "every id pool", "the pool assigned the predicted ids").
`vox_value_pool.rs` mixes both: the type is a value pool, but its value ids
come from an internal id pool. Every mention needs a per-sentence judgment;
see Q1 for what happens to the id-pool mentions themselves.

## Crates in the blast radius

A survey (`grep -rn '\bpool'` filtered to lines not already spelling value
pool out) finds about 490 lines. The clusters:

- **voxcore, public surface (breaking).** `VoxProperty::pool_id`, the
  `pool_id` fields on the `Error` variants, and the `pool_id` parameters on
  `VoxPalette::add_property` and the `VoxMain` value-pool methods become
  `value_pool_id`. Ripples into every voxsmith and vxl call site.
- **voxcore, internals and tests.** Locals and bindings (`pool`, `pool_ref`,
  `pool_remap`, `pool_id_space`, `pool_property_ids`, `property_pool_ids`),
  the `VoxEffectiveProperty::pool` field, and test helpers (`int_pool`,
  `pool_id(..)`, `pool_a_id`, `wild_pool_id`).
- **voxsmith helpers.** `pool_color` (its own file), `pool_scalar`,
  `pool_flag`, `property_pool`, `float_pool`, and the `PoolColumn` type in
  `voxelize_mesh.rs`, plus the `pool_len` test helper in `reduce_palette.rs`.
- **vxl.** `pool` / `pool_id` bindings and the `pool_kind` helper in
  `mesh_object.rs` and `palette_show.rs`, plus test locals.
- **Prose and messages, all three crates.** Doc comments ("an `int` pool",
  "a color pool", "the pool's values", "a non-color pool") and the `Error`
  display messages ("value {} is not one of the pool's", "each of the pool's
  value ids"). Message text changes are observable output and land in their
  own commit.

## What is settled

- `voxj` and `voxj-codec` are already swept; their wire keys (`valuePool`,
  `valuePools`) and the serde fields mapped to them are the only sanctioned
  short forms and stay as the spec writes them.
- Spec kind names stay bare inside kind-qualified prose: "an `int` value
  pool", never "an `int-value` pool".
- The spec doc does not change.
- Contextual bare "pool" is not grandfathered; the earlier tolerance for it
  is withdrawn.

## Decisions

### Q1. Bare id-pool mentions (open)

Prose like "Compact the palette pool" or "the voxel pool" means a branded-id
pool, and after this sweep those would be the only bare "pool" mentions left.

- **A. Expand to "id pool"** ("the palette id pool", "the voxel id pool"), so
  a final grep for bare "pool" returns nothing and the two entities can never
  be confused again.
- **B. Leave them.** The rule the owner stated covers value pools only.

Recommendation: **A**, as a small docs-only final phase. It is the same
disambiguation motive at zero risk. Until the owner rules, phases A through D
leave id-pool mentions untouched.

## Execution shape

1. Phase A: the breaking `value_pool_id` rename across the public surface and
   every dependent call site, one commit.
2. Phase B: the voxsmith `pool_*` helpers and `PoolColumn`.
3. Phase C: entity bindings, locals, and test helpers in all three crates.
4. Phase D: doc prose, then the display messages in their own commit, then
   the final grep gate.
5. Phase E (only if Q1 resolves to A): expand bare id-pool prose to "id
   pool".

Phases B through D touch disjoint names but the same files; run them in order
rather than in parallel so each gate grep stays meaningful.
