# Naming Sweep Checklist

Tracks the sweep from the [README](README.md): the `_id`/`_ids` and `_index`
suffixes and the spelled-out value pool, one iteration per crate, in order:
`voxcore`, `voxsmith`, `vxl`. The line-level inventory behind every item is
[reference/survey.md](reference/survey.md); its line numbers are from the
2026-07-29 survey and drift as work lands, so re-grep before editing.

## Ground rules

- One concern per commit, in the style of the reference series
  (`refactor(voxel)!` for a breaking voxcore rename, `refactor(voxsmith)`,
  `refactor(vxl)`, `docs(...)` for prose-only commits).
- A breaking rename lands with every forced call site in the same commit;
  the workspace compiles at every commit.
- Renamed `format!` placeholders keep rendered text byte-identical. Message
  text edits land only in the iteration's dedicated message commit.
- Only value-pool mentions are renamed to `value_pool`; the branded-id
  pools in voxcore keep their names (the README guard). Log any mention
  whose entity took real judgment in
  [reference/ambiguous-mentions.md](reference/ambiguous-mentions.md), one
  line each: file, line, ruling, why.
- Bare `id` survives only as the subject parameter of a single-entity
  function (Q2); every other bare-`id` binding takes its entity name. The
  bare id-pool prose expands to id-pool forms in iteration 1's final docs
  commit (Q1).
- Before every commit: `cargo fmt --all` and
  `cargo clippy --workspace --all-targets -- -D warnings`. After each
  iteration: `cargo test -p voxcore -p voxsmith -p vxl`.
- The value-pool gate grep, per crate:
  `grep -rni '\bpool' projects/utilities/<crate>/src --include='*.rs' | perl -ne 'my $s = $_; $s =~ s/^[^:]*:\d+://; $s =~ s/value[-_ ]pools?//gi; $s =~ s/valuepools?//gi; print if $s =~ /\bpool/i'`
  The perl filter tests a copy of each line with the file prefix and the
  spelled-out forms removed, so a `value_pool` filename or a spelled-out
  mention cannot mask a bare mention sharing its line, and `pooled` counts
  as bare. After iteration 1 it returns only id-pool mentions for voxcore
  (nothing, once the Q1 item lands); after iterations 2 and 3 it returns
  nothing for voxsmith and vxl.
- A heuristic id-suffix grep, worth running at each iteration's end:
  `grep -rnE '\b(let|for) &?\(?(palette|object|node|layer|voxel|material|property|root|child)\b' projects/utilities/<crate>/src --include='*.rs'`
  It over-matches (entity-value bindings are fine); read the hits against
  the rule rather than chasing zero.

## Iteration 1: voxcore

- [x] The breaking surface, one `refactor(voxel)!` commit: rename
      `VoxProperty::pool_id` to `value_pool_id`; rename the `pool_id` field
      on the six `Error` variants (`EmptyValuePool`, `UnknownValuePool`,
      `PropertyValuePoolRef`, `ValuePoolBound`, `ValuePoolValue`,
      `PropertyValuePool`), display texts unchanged; rename
      `VoxValuePool::clone_pool` to `clone_value_pool`. Update the forced
      voxsmith and vxl call sites in the same commit, taking only the new
      API names.
- [x] Public and `pub(crate)` parameter names and their rustdoc references:
      `pool_id` on `VoxMain::add_property`,
      `VoxMain::remove_value_pool_value`, `VoxMain::reorder_value_pool`,
      `VoxPalette::add_property`, and
      `VoxPalette::repoint_value_pool_value`; `pool` on
      `VoxMain::add_value_pool`; the `VoxEffectiveProperty::pool` field.
- [x] Internal bindings and locals: `pool_ref`, `pool`, `pools`,
      `pool_id_space`, `pool_remap`, `pool_ids`, `pool_property_ids`, and
      `property_pool_ids` in `vox_main.rs` and `vox_palette.rs`.
- [x] Test identifiers: the helpers (`pool_id` in `vox_main.rs` and
      `vox_palette.rs`, `int_pool` in `vox_main.rs` and
      `vox_effective_palette.rs`, the `pool_id` parameters on the palette
      helpers), the locals (`pool_a_id`, `pool_b_id`, `first_pool_id`,
      `second_pool_id`, `wild_pool_id`, and kin), the four test names
      saying bare pool, and the debug dump format string in `vox_main.rs`
      (`|pool {pool_id:?}`), whose snapshot text changes with it.
- [x] The index residue: `first_cycle_position`'s `start` and `node`
      bindings and its two callers' `index` and `position` locals become
      `_index` forms, matching the swept voxj-codec twin.
- [x] The bare-`id` bindings under the Q2 policy: the seven loop bindings
      (`vox_runtime_state.rs`, `vox_object.rs`, `vox_main.rs`,
      `vox_value_pool.rs`) and the closure arguments take entity names; the
      accessor subject parameters stay `id`.
- [x] Prose, one docs commit: the roughly 108 bare-pool doc lines meaning a
      value pool: `vox_value_pool_value_ref.rs` (all nine variant docs),
      `vox_value_pool.rs` (the nine constructor docs and the type and
      accessor docs), `error.rs`, the `vox_main.rs` rustdoc and internal
      comments, `vox_palette.rs`, `vox_effective_property.rs`,
      `vox_value_pool_flaw.rs`, `vox_value_pool_kind.rs`,
      `vox_gc_remap.rs`, and the test comments. Judge each line against the
      id-pool guard; log the close calls.
- [x] The message commit: the four `Display` strings in `error.rs` and the
      three `unreachable!` messages in `vox_value_pool.rs` spell value pool
      out.
- [x] Expand the bare id-pool prose to "id pool" forms (Q1), one docs
      commit.

Gate: workspace green; the voxcore gate grep returns only id-pool mentions
(nothing once the id-pool prose commit lands).

## Iteration 2: voxsmith

- [x] The breaking field, one commit: `MaterialMeshRequest::layer` to
      `layer_id`, with the forced vxl call sites. Moot: the layer-flatten
      rework landed after the survey and dropped the field, so there is
      nothing to rename.
- [x] Value-pool identifiers: rename `internal/pool_color.rs` to
      `value_pool_color.rs` with its function and the `mod` and re-export
      lines; `property_pool`, `pool_scalar`, and `pool_flag` in
      `internal/vmax/write_vmax.rs`; `PoolColumn`, `srgb_pool`,
      `srgba_pool`, and `float_pool` in
      `convert/voxelize/voxelize_mesh.rs`; `float_pool` in
      `convert/vmax/from_vmax_file.rs`; `pool_ref` in
      `order_palette_colors.rs`; `non_color_pool` in
      `internal/resolve_cell_color.rs`; the test helpers `pool_len` and
      `numbered_pool`; the six test names saying bare pool (the survey's
      three plus three that drifted in); and every `pool` and `pool_id`
      local, parameter, and closure argument on the survey's value-pool
      list, plus the `float_pool` and `pool_id` test helpers the layer
      flatten added to `internal/gltf/used_materials.rs`.
      `order_palette_colors.rs` drifted clean of its `pool_ref` before this
      iteration.
- [x] Id suffixes, the six hot files: `internal/vmax/write_vmax.rs`,
      `reduce_palette.rs`, `convert/vmax/to_vmax_file.rs`,
      `convert/voxelize/voxelize_mesh.rs`, `internal/gltf/bake_atlas.rs`,
      and `convert/vmax/from_vmax_file.rs`.
- [x] Id suffixes, the converter family and helpers: the qbcl, goxl, and
      mvox readers and writers, `order_palette_colors.rs`,
      `internal/grid.rs`, `internal/gltf/used_materials.rs`, and the small
      gltf, mesh, and voxj files on the survey's per-file list, including
      the bare-`id` locals, destructures, and closures under the Q2
      policy.
- [x] Index suffixes and the misnamed ids: the `_index` renames in
      `from_vmax_file.rs`, `from_mvox_file.rs`, `reduce_palette.rs`, and
      `to_vmax_file.rs`; the `_idx` cluster in `write_vmax.rs`, keeping the
      Voxel-Max wire-struct field names; and the seven `index` bindings
      holding a `U32Id<BVoxValuePoolValue>` (`from_gltf_bytes.rs` and
      `to_vmax_file.rs`, plus `PoolColumn::indices` to `value_ids`, and
      the three value-id `index` locals inside the `ValuePoolColumn`
      builders).
- [x] Prose, one docs commit: the roughly 90 comment and doc lines on the
      survey list, plus the lines the survey's gate grep could not see: the
      doc blocks in `value_pool_color.rs` and the two voxj value-pool
      converters (their filenames matched the grep's exclusion), a
      `pooled copy` in `from_mvox_file.rs`, and the mixed
      value-pool-and-bare lines in `reduce_palette.rs` and
      `vox_palette_from_voxj_palette.rs`.
- [x] The message commit: the `order_palette_colors.rs` and
      `resolve_cell_color.rs` failure texts spell value pool out, plus the
      failure texts that drifted past the survey: the seven `unreachable!`
      kind-mismatch texts in `voxj_value_pool_from_vox_value_pool.rs` and
      the seven test `panic!` texts in `voxelize_mesh.rs`,
      `from_gltf_bytes.rs`, and `to_vmax_file.rs`.
- [x] Gate residue the heuristic grep surfaced: `MeshTriangle::material`
      and `MeshGeometry::materials` hold indices into the mesh material
      table and become `material_index` and `material_indices` with the
      locals that feed and read them. The greedy mesher's material-key
      bindings keep their names: the key is a merge key, not a positional
      index, and the pure-geometry path keys every face with a constant.

Gate: workspace green; the voxsmith gate grep returns nothing.

## Iteration 3: vxl

- [x] Id suffixes: `implementation/hierarchy_show.rs` (the walkers, the
      treegrid node ids, and the test fixtures), `resolve_objects.rs`,
      `info.rs`, `palette_list.rs` (including the `count` binding that
      holds a node id), `palette_show.rs`, `mesh_object.rs`, `validate.rs`,
      `voxelize.rs`, the entity-named test helpers that return ids
      (`fn value`, `fn object`, and `resolve_objects.rs`'s sibling
      `fn node`), and the bare-`id` destructures and closures under the Q2
      policy (the placement-helper subject parameters stay `id`).
      `mesh_object.rs`'s main-code palette locals drifted away with the
      layer flatten, leaving only its test fixtures.
- [x] Index suffixes: the `object` parameter on `Dependencies` and
      `DependenciesImpl` and the `mesh.rs` locals feeding it;
      `mesh_object.rs`; `palette_show.rs` (`Collection::palette`, the
      `index` parameters, `channel`); `resolve_objects.rs`;
      `hierarchy_show.rs` (`parent`, `this`, `parents`, `instance`).
      Moot: `Mesh::layer`, its `--layer` flag, and the `layer` parameters
      went away with the layer-flatten rework, so there is no
      `--layer-index` to move.
- [x] Value-pool identifiers: `pool_kind` in `mesh_object.rs`; the
      `pool: &VoxValuePool` parameters in `palette_show.rs` (its `pool` and
      `pool_id` locals landed with voxcore's parameter rename); the
      `srgba_pool` test helper; the fixture locals (`let pool` and kin) in
      `palette_show.rs`, `info.rs`, and `hierarchy_show.rs`; and the four
      test names saying bare pool (the survey's three plus
      `mesh_object.rs`'s).
- [x] Prose, one docs commit: the comment and doc lines in
      `palette_show.rs`, `mesh_object.rs`, `channel_source.rs`,
      `dependencies.rs`, `write_voxj_document.rs`,
      `voxj_encoding_options.rs`, `voxj_color_format.rs`, and
      `voxelize.rs`. The survey's `info.rs` and `hierarchy_show.rs` prose
      lines were fixture identifiers, not comments, so they landed with the
      identifier commit.
- [ ] The message commit: the `mesh_object.rs` "bound to a string or json
      pool" text spells value pool out.

Gate: workspace green; the vxl gate grep returns nothing; the
report-command output assertions are unchanged outside the message commit.

## Iteration 4: voxj-codec

- [ ] One commit renaming the outer subject indices in the seven `check_*`
      functions to `<entity>_index` (the survey's 14 sites), message texts
      unchanged.

Gate: workspace green; `cargo test -p voxj-codec` green.
