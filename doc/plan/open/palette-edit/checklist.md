# Checklist

The design is in the [README](README.md). Check steps off as they land.

## Ground rules

- Each step is one commit, staged for review before the next starts.
- The voxsmith operations are generic over `VoxExt`. vxl stays a thin clap
  layer shaped like `palette quantize`.
- Each voxsmith operation gets unit tests. Each vxl command gets a parse test.

## Steps

- [x] **S1. f64 language.** In `vox-value-language`, rename `Scalar::F32`,
      `Components::F32`, and `NumberSuffix::F32` (`lexer/number_suffix.rs`)
      to `F64`. Rename the `f32` conversion in `function.rs` to `f64`. Drop
      the conversion's exactness error for unsigned input. Recheck
      `CHROMA_FLOOR` (`evaluator/eval_node.rs:1327`) and the `test/f32s.rs`
      and `test/assert_close.rs` helpers. Follow every `Components::F32` and
      `Scalar::F32` match through voxsmith's `operations/object/object_mesh`
      and vxl's `object_mesh/internal/record/program_builder.rs`.
      `program/mesh_environment.rs` reads pools without narrowing.
      `write/write_materials.rs` and `write/write_attributes.rs` narrow under
      the README's rule. `write/write_files.rs` and `write/write_extras.rs`
      write `f64`. Update the crate README, `doc/ref/mesh/value-language.md`,
      and `doc/ref/mesh/mesh.md`.
- [x] **S2. Swatch count and mentioned names.** Add the swatch count to
      `Groupings` (`environment/groupings.rs`). `Lengths::from_groupings`
      (`evaluator/lengths.rs`) checks voxel swatch ids against it.
      `groupings_of` (`mesh_environment.rs:299`) passes `swatches.count()`.
      The `test/groupings.rs` helper takes a count. Add a syntax-level scan of
      the names a parsed program or expression mentions (`parser/program.rs`,
      `parser/syntax_node.rs`). `MeshEnvironment::bind` binds only the
      properties the scan finds, so `program/program_run.rs` parses before
      binding.
- [x] **S3. voxcore setters.** Add `VoxValuePoolValue` and an append on
      `VoxValuePool` beside `release_value_stable` (`vox_value_pool.rs`). The
      append domain-checks like `checked`. Add a cell setter on `VoxPalette`
      beside `value_id` (`vox_palette.rs`). `VoxMain` wraps them as
      `retain_value_pool_value` and `set_material_value`.
- [ ] **S4. Palette selection.** Add a voxsmith `PaletteSelector` holding `*`
      or an `IndexRange` (`utilities/index_range.rs`). Add a
      `resolve_palette_selectors` returning palette ids in palette order. vxl
      parses the selector with a `parse_palette_selector` beside
      `parse_index_range` (`internal/parse_index_range.rs`). `quantize_palette`
      (`operations/palette/quantize_palette.rs`) takes the resolved ids. It
      errors on a palette no live voxel samples before
      `release_unsampled_materials` runs. `PaletteQuantize::index`
      (`commands/palette/palette_quantize/palette_quantize.rs`) becomes the
      repeatable selector. Behavior change: a bare quantize now reaches every
      palette.
- [ ] **S5. voxsmith `edit_palettes`.** Move `property_value` and its pool
      readers out of `mesh_environment.rs` into a crate-internal file. That
      file's `mod` line is gated on `any(feature = "object", feature =
      "palette")`. The `palette` feature gains `dep:vox-value-language`. The
      operation builds one environment per selected palette, evaluates all of
      them, then writes. The held-value release at the end of
      `release_unsampled_materials` moves into a crate-internal helper both
      operations call. Range checks go through `check_material_range`
      (`utilities/check_material_range.rs`).
- [ ] **S6. vxl `palette edit` and profiles.** Add
      `commands/palette/palette_edit/` beside `palette_quantize/`. The command
      writes through `edit_document` (`internal/edit_document.rs`). Program
      assembly, fragment origins, and `--values-from` follow
      `object_mesh/internal/flags/program_flags.rs` and
      `object_mesh/internal/record/program_builder.rs`. Stacking and
      `valuesFrom` follow palette show's `stack_palette_show_profiles`
      (`commands/palette/palette_show/palette_show.rs:206`) and
      `palette_show/internal/property_selector_builder.rs`. `PaletteConfig`
      (`commands/palette/internal/palette_config.rs`) gains `edit`. A
      `load_palette_edit_profile_set` loads it the way
      `load_palette_quantize_profile_set` loads `quantize`.
      `profile palette edit list` mirrors
      `profile/profile_palette/profile_palette_quantize/`.
- [ ] **S7. Docs.** Extend the vxl README's Palettes section. Its quantize
      example now reaches every palette. The
      [vxl-commands README](../vxl-commands/README.md#commands) command list
      links `palette edit` here. Item 4 of
      [conventions.md](../vxl-commands/reference/conventions.md) takes the
      [palette selection](README.md#palette-selection) rule.
      [palette/README.md](../vxl-commands/reference/palette/README.md),
      item 2 of [quantize.md](../vxl-commands/reference/palette/quantize.md),
      and items 2 and 4 of
      [remap.md](../vxl-commands/reference/palette/remap.md) point to it.
      The intro of `doc/ref/mesh/value-language.md` mentions `palette edit`
      beside `object mesh`.
