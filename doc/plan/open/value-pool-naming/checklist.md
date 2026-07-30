# Value-Pool Naming Checklist

Tracks the sweep from the [README](README.md): spell "value pool" out across
`voxcore`, `voxsmith`, and `vxl` wherever a bare "pool" means a value pool.
Run the phases in order; they touch the same files.

Log any mention whose entity (value pool versus branded-id pool) took real
judgment in [reference/ambiguous-mentions.md](reference/ambiguous-mentions.md),
one line each: file, line, ruling, why.

## Ground rules

- **The disambiguation rule.** A "pool" is a value pool only when it holds
  property values (`VoxValuePool`, `pool_id: U32Id<BVoxValuePool>`, a binding
  fetched from `value_pools`). The branded-id pools behind the entity listings
  (palette, object, node, layer, voxel, property, material, and the value ids
  inside a value pool) are a different entity and keep their names; see the
  README's file list and Q1.
- Keep spec kinds bare inside kind-qualified prose: "an `int` value pool".
- Renamed placeholders in `format!` strings keep the surrounding literal text
  byte-identical; message-text changes belong only in the Phase D message
  commit.
- Run `cargo fmt --all` and
  `cargo clippy --workspace --all-targets -- -D warnings` before every commit;
  `cargo test -p voxcore -p voxsmith -p vxl` after each phase. The workspace
  stays green throughout.
- Follow `CLAUDE.md` and the repo style; one concern per commit, mirroring
  the `refactor(voxel)!` series that renamed the `VoxPool*` types.
- The gate grep, after each phase:
  `grep -rni '\bpool' projects/utilities/{voxcore,voxsmith,vxl}/src --include='*.rs' | grep -vi 'value_pool\|value pool\|value-pool\|ValuePool'`
  Each phase shrinks the output; after Phase D only id-pool mentions remain.

## Phase A: the public `value_pool_id` surface (breaking)

One `refactor(voxel)!` commit; the workspace must compile at its end, so every
dependent call site lands with it.

- [ ] Rename `VoxProperty::pool_id` to `value_pool_id`.
- [ ] Rename the `pool_id` fields on every `Error` variant to `value_pool_id`,
      keeping each display message's text unchanged.
- [ ] Rename the `pool_id` parameters on `VoxPalette::add_property` and the
      `VoxMain` value-pool methods (`add_property`, `remove_value_pool_value`,
      `reorder_value_pool_values`, and kin), and follow their doc comments.
- [ ] Update every voxsmith and vxl call site the field and parameter renames
      ripple into (`voxj_palette_from_vox_palette`, `order_palette_colors`,
      `reduce_palette`, `write_vmax`, `voxelize_mesh`, `from_gltf_bytes`,
      `mesh_object`, `palette_show`, and the tests).

Gate: workspace compiles, lints, and tests green; the gate grep no longer
shows `pool_id` outside id-pool contexts.

## Phase B: the voxsmith `pool_*` helpers

- [ ] Rename `pool_color` to `value_pool_color`, including the file
      `internal/pool_color.rs` and the `mod` and re-export lines.
- [ ] Rename `pool_scalar`, `pool_flag`, and `property_pool` in
      `internal/vmax/write_vmax.rs` to `value_pool_scalar`,
      `value_pool_flag`, and `property_value_pool`.
- [ ] Rename `PoolColumn`, `float_pool`, and the color-pool builders in
      `convert/voxelize/voxelize_mesh.rs` to their `ValuePoolColumn` /
      `float_value_pool` forms.
- [ ] Rename the `pool_len` test helper in `reduce_palette.rs` to
      `value_pool_len`.

Gate: workspace green; the gate grep shows no `pool_` helper names.

## Phase C: bindings, locals, and test helpers

- [ ] voxcore: `pool`, `pool_ref`, `pools`, `pool_remap`, `pool_id_space`,
      `pool_property_ids`, and `property_pool_ids` bindings become
      `value_pool*` forms; `VoxEffectiveProperty::pool` becomes `value_pool`.
- [ ] voxcore tests: `int_pool`, the `pool_id(..)` helper, `pool_a_id`,
      `pool_b_id`, and `wild_pool_id` become `value_pool*` forms.
- [ ] voxsmith: the `pool` bindings in `order_palette_colors`,
      `reduce_palette`, `bake_atlas`, the converters, and their tests.
- [ ] vxl: the `pool` / `pool_id` bindings and the `pool_kind` helper in
      `mesh_object.rs` and `palette_show.rs`, and the test locals.

Gate: workspace green; the gate grep shows only prose, messages, and id-pool
mentions.

## Phase D: prose and messages

- [ ] Sweep the doc prose in all three crates: "an `int` pool", "a color
      pool", "a float pool", "the pool's values", "a non-color pool", "the
      bound pool", and kin become value-pool forms. Apply the disambiguation
      rule line by line; log judgment calls.
- [ ] In a separate commit, reword the display messages that say bare "pool"
      (`error.rs`: "not one of the pool's", "each of the pool's value ids",
      "of its pool's"; voxsmith: "draws from a non-color pool"; vxl: "bound
      to a string or json pool"). Message text changes are observable output;
      the commit body says so.
- [ ] Run the gate grep and confirm every remaining line is an id-pool
      mention; paste the residue into
      [reference/ambiguous-mentions.md](reference/ambiguous-mentions.md) as
      the closing record.

Gate: workspace green; the residue list is id-pool mentions only.

## Phase E: id-pool prose (blocked on Q1)

Only if the owner resolves Q1 to A:

- [ ] Expand bare id-pool prose to "id pool" forms ("the palette id pool",
      "the voxel id pool", "the layer id pool") across the files the README
      lists, one docs commit.

Gate: the gate grep returns nothing.
