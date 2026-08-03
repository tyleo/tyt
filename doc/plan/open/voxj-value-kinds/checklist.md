# voxj value kinds implementation checklist

Tracks the change from the [README](README.md): the target spec text is
agreed with the owner first, lands as one spec commit, and the code
follows in two moves, the property rename first and then the shape-only
kind vocabulary one crate per iteration in dependency order.
[reference/format-design.md](reference/format-design.md) is the target
spec text once iteration 1 drafts it. The line-level inventory behind the
code items is [reference/survey.md](reference/survey.md); its line numbers
are from the 2026-08-01 survey and drift as work lands, so re-grep before
editing. Log code-level choices in
[reference/implementation-decisions.md](reference/implementation-decisions.md).
The per-session resume prompt is
[continue-voxj-value-kinds.md](continue-voxj-value-kinds.md).

## Ground rules

- Iteration order is strict: the format design is approved before the
  spec commit, the spec lands before any code moves, and the rename lands
  before the kind iterations. Once the spec lands it is authoritative for
  any detail the plan leaves implicit.
- One concern per commit, one iteration per session where it fits, owner
  review at each gate. `docs(voxj)` for spec commits, `docs(plan)` for
  plan-page commits, `feat`/`refactor` with `!` where a public surface
  breaks.
- `cargo fmt --all` before every commit. Iterations 1 through 3 keep the
  whole workspace green (1 touches plan pages, 2 the spec). Iteration 4
  opens a red window that iteration 8 closes: downstream crates do not
  compile between crate iterations, so scope clippy and tests to the
  crates in play (`cargo clippy -p <crate> --all-features --all-targets
  -- -D warnings`, `cargo test -p <crate> --all-features`) and say so
  plainly. The pre-commit hook's workspace clippy cannot pass inside the
  window; its header documents the bypass.
- A per-crate run needs the crate's features spelled out or it compiles
  none of the work. voxj declares no default features and gates every
  test on `serde`, so a bare `cargo test -p voxj` runs zero tests.
  voxsmith's default set omits `gltf`, the only enabler of the private
  `_mesh` marker, so a bare `cargo test -p voxsmith` runs 132 of 218
  tests and compiles neither the glTF trees nor the mesh ones. A bare
  `cargo clippy -p voxsmith --all-targets` fails on the pristine tree for
  the same reason: `scalar_range`'s only caller sits behind `_mesh`.
- The README stays as approved. New findings land in the survey and the
  decisions log, and the survey's
  [corrections](reference/survey.md#corrections-to-the-readmes-blast-radius)
  section already carries five deltas the items below encode: the int
  parse rules are new (not moved from voxj-codec), voxcore's `error.rs`
  and effective-palette files are in the blast radius, roughly twenty more
  color-constructor call sites break at compile, the u8 identity test
  passes on today's transfer and so drives no rewrite of it, and the
  `float_roundtrip` additions only prove out in per-crate test runs.
- Fixtures are inline in tests and regenerate in the same commit that
  breaks them. The one on-disk asset regenerates once, at closeout;
  nothing in the workspace reads it.
- Iteration 3 alone would fail silently at a crossing (an old file's
  `Factor` names would no-op), which the README accepts only because the
  kind break follows in this same plan. Do not close or pause the plan
  after iteration 3 lands; drive on to the break.
- The glTF wire keeps its `Factor` field names everywhere: the
  serde_json output keys, the `gltf` crate accessor calls, the test GLB
  builders, and the spec table's per-row glTF citations. Only voxj
  property names rename.
- voxsmith and vxl production code spells property names through the
  `gltf_attributes` constants; voxj, voxj-codec, and voxcore sit below
  voxsmith and keep plain literals. Four voxsmith test modules spell them
  literally too, so they rename alongside the constants rather than
  through them.
- The vmax `--color-format` (`png`/`plist`/`all`) is an unrelated flag and
  never changes; only the voxj one deletes.
- Closed plan pages never change. The open vxl-commands pages sweep at
  closeout.
- The Factor gate grep, run at iteration 3's gate and again at closeout:
  `grep -rn 'baseColorFactor\|metallicFactor\|roughnessFactor\|emissiveFactor\|transmissionFactor' projects doc/plan/open`
  Read the hits against the wire rule. Under `projects` only glTF wire
  contexts and treegrid's display sample data may remain. Under
  `doc/plan/open` this plan's own pages keep the old names by design, and
  the vxl-commands pages keep them until the closeout sweep. The grep
  searches neither `doc/plan/closed` nor `submodules`, so the closed
  pages and the asset never reach its output.
- The kind gate grep, run at closeout:
  `grep -rn 'srgb-hex\|srgba-hex\|srgb-float\|srgba-float\|linear-rgb-float\|linear-rgba-float\|VoxjBound\|VoxBound' projects doc/plan/open`
  returns nothing under `projects`. It keeps matching this plan's own
  pages, which name the old kinds throughout, until the plan moves to
  `doc/plan/closed`.

## Iteration 1: format design

Iterate [reference/format-design.md](reference/format-design.md) with the
owner until approved. Nothing else starts first; the format gets perfect
here, not in the spec commit.

- [x] Draft `reference/format-design.md`: the target text for every spec
      section the change touches, both the rename and the kinds in one
      document. The sections: the kind table (the eleven shape kinds,
      typical-use column on the renamed names) and its notes, the value
      domains (the `"inf"`/`"-inf"` sentinels, NaN rejected everywhere,
      int values integer-spelled and capped at `2^53 - 1` in magnitude),
      validation rules 9 and 10 reduced to shape checks, the TypeScript
      schema, the examples (new kinds, renamed properties, no
      `min`/`max`), the glTF conventions table (the property column
      renames and each row keeps its glTF citation in the description
      column; the Kind column moves to the new kinds, and the Range
      column restates each range from
      [reference/gltf-ranges.md](reference/gltf-ranges.md), `ior`
      included, so the spec and iteration 7's check agree on it), the
      emissive-composition paragraph (which names the
      properties in both the voxj and the glTF sense; only the voxj sense
      renames), the layer-override worked example, the format-wide
      sentence fixing linear light with sRGB primaries and the D65 white
      point, the sentence scoping ranges to the property vocabulary, and
      the texture scope boundary.
- [x] Owner reads the draft; fold in wording and structure edits until
      the text stands on its own. A design ruling the review surfaces is
      the owner's to make; record it in the decisions log (the README
      stays as approved).

Gate: the owner approves format-design.md.

## Iteration 2: the spec, one change

- [x] Rewrite `projects/voxel-codecs/voxj/docs/voxel-json-file-format.md`
      to the approved design in one commit. No code changes. Every
      section from the format-design draft, the rename and the kinds
      together.
- [x] Sweep the whole spec for stale color-kind, transfer, bound, or
      `Factor` wording the section list missed.
- [x] Sweep the whole spec's comment blocks for hanging indents: a lead
      line's sub-items sit flush after a blank `//` line, never indented
      under the lead.

Gate: the spec matches format-design.md; the reference file stops being
authoritative and the spec is, for any detail the plan leaves implicit.

## Iteration 3: the property rename in code

The `Factor` suffix drops from the five voxj property names
(`baseColor`, `metallic`, `roughness`, `emissiveColor`, `transmission`;
`occlusionStrength`, `emissiveStrength`, and `ior` stay). Workspace green
throughout; one workspace-wide commit, split only if review needs it.

- [x] The vocabulary source: `voxsmith/src/gltf_attributes.rs` constant
      values and identifiers (`BASE_COLOR_FACTOR` becomes `BASE_COLOR`
      valued `"baseColor"`, and kin), `default_scalar`, `scalar_range`,
      and `default_color` in lockstep, and `gltf_attribute_kind.rs`. Doc
      comments keep their glTF field citations.
- [x] The constant users in voxsmith and vxl (the survey's per-file
      list: the converters, the atlas bake, the palette reduction, the
      vmax writer, the mesh commands), with the
      `material_document.rs` tuple that pairs the voxj constant with the
      wire literal `"transmissionFactor"` kept as two strings.
- [x] The bare literals in the crates below voxsmith: voxcore
      (`vox_palette.rs`, `vox_main.rs`, `vox_property.rs` doc examples),
      voxj-codec (`validate_voxj_file.rs`, `check_voxj_file.rs`,
      `voxj_palette_material_counts.rs`), and voxj (`voxj_file.rs` test
      fixture).
- [x] The bare literals in voxsmith's own test modules, which the
      constants cannot reach: `convert/vmax/to_vmax_file.rs` (12),
      `convert/voxj/from_voxj_file.rs` (11),
      `internal/voxj/vox_palette_from_voxj_palette.rs` (4), and
      `internal/gltf/used_materials.rs` (1). `to_vmax_file.rs` is the one
      that fails rather than merely going stale: its literal-keyed
      lookups read a palette that `from_vmax_file` rebuilds from the
      constants, so they miss once the constant carries `"metallic"`. The
      `property_id_by_name("metallicFactor")` at L1309 panics on its
      `expect`, and the `scalar("metallicFactor")` and
      `scalar("roughnessFactor")` at L1339-1340 panic on the `unwrap`
      inside the closure at L1316.
- [x] The vxl test fixtures and expectations (`palette_show.rs`,
      `palette_list.rs`, `info.rs`, `hierarchy_show.rs`, the mesh command
      tests), re-baselining the `--width 30` expectation whose comment
      counts the `baseColorFactor` prefix width.
- [x] The prose carrying the vocabulary: the clap help text in the vxl
      mesh, voxelize, and palette commands, the voxsmith
      `out_of_range_factor.rs` docs, the internal doc comments on the
      survey's list, and the `resolve_cell_color.rs` failure text.
- [x] Decide and log: whether `internal/mesh/mesh_material.rs`'s internal
      `emissive_factor` field keeps its name (the mesh side genuinely
      samples factor times texture).

Gate: workspace fmt, clippy, and `cargo test --workspace` green; the
Factor gate grep returns only glTF wire contexts, treegrid sample data,
and the open plan pages (this plan's own, which keep the old names by
design, and the vxl-commands pages that sweep at closeout).

## Iteration 4: voxj

Opens the red window: voxj-codec, voxsmith, vxl, tyt-vmax, and tyt stay
red until iteration 8 closes it.

- [x] Rewrite `voxj_value_pool.rs` to the adjacently tagged enum on `Vec`
      (`tag = "kind"`, `content = "values"`, `rename_all = "kebab-case"`,
      `deny_unknown_fields`): the six color variants and the `min`/`max`
      fields go, the six vector variants arrive, `values_len` covers the
      new arms, and the doc comments and their examples follow. The six
      vector variants each carry an explicit
      `#[serde(rename = "vec-3-float")]` and kin, because kebab-case
      inserts a separator only before an uppercase character and never
      before a digit, so `Vec3Float` would otherwise spell itself
      `vec3-float`.
- [x] Add the sentinel serde module: a float value reads a finite
      number, `"inf"`, or `"-inf"`; infinities write the sentinel
      strings; NaN errors on write; an integral number writes as a JSON
      integer so `1` does not round-trip as `1.0`; array forms cover the
      `[f64; N]` payloads.
- [x] Add the int visitor: a value beyond `2^53 - 1` in magnitude
      rejects, a fractional or exponent spelling rejects in the parse,
      and array forms cover the `[i64; N]` payloads.
- [x] Delete `voxj_bound.rs`; update `lib.rs`; move `voxj_file.rs`'s
      `document()` and `wire_document()` fixtures to the new kinds.
- [x] `Cargo.toml`: the dev serde_json gains `float_roundtrip`. The
      feature bears on text parsing only, and the crate's tests go
      through `to_value` and `from_value`, so pair it with a `to_string`
      and `from_str` case or the manifest change proves nothing here.
- [x] In-crate tests: sentinel round-trips, NaN write error, integral
      float written as an integer, the int cap and spelling rejections, a
      stray `min` rejecting through `deny_unknown_fields`.

Gate: `cargo test -p voxj --features serde` green. The bare
`cargo test -p voxj` runs zero tests, because the crate has no default
features and every test is gated on `serde`.

## Iteration 5: voxj-codec

- [x] Shrink `internal/voxj_validation/check_value_pools.rs` to shape
      checks: non-empty `values` stays a validation rule; the bound rules
      (`check_numeric`, `describe`), the hex rules (`check_hex`,
      `is_hex_color`), and the color range rules (`check_colors`) delete;
      the match covers the eleven kinds.
- [x] Rewrite the `check_voxj_file.rs` check-list doc (the `value-pools`
      item becomes a shape statement) and the same vocabulary where it
      appears a second time, on `Check::ValuePools` in
      `internal/voxj_validation/check.rs`, which restates the bound, hex,
      and color-range rules in prose no gate grep can see; and the
      `validate_voxj_file.rs` tests: the bound, hex, and color range
      tests go; the shared fixture moves to the new kinds; the structural
      majority survives on edited fixtures.
- [x] `Cargo.toml`: serde_json gains `float_roundtrip` beside
      `preserve_order`.
- [x] The byte-identity test: a `float` value pool holding
      17-significant-digit values saves and loads byte-identical, proven
      by `cargo test -p voxj-codec` (a workspace build already unifies
      the feature on through vmax-codec, so only the per-crate run proves
      the manifest).

Gate: `cargo test -p voxj-codec` green.

## Iteration 6: voxcore

- [x] Rework `vox_value_pool_kind.rs` to the eleven kinds (the four color
      variants and the `Float`/`Int` bounds go; six vector variants
      arrive on `IdField` payloads) and `vox_value_pool_value_ref.rs` to
      match.
- [x] `vox_value_pool.rs`: the four color constructors delete, `float`
      and `int` lose their bound parameters, six vector constructors
      arrive; the six kind matches (`first_flaw`, `clone_value_pool`,
      `release_value_stable`, `gc_values`, `value_ref`, `Drop`) cover the
      new arms; `first_color_flaw` and the six bound free functions
      delete, the exact-integer comparison with them (its rule lives in
      voxj's parse now). Decide and log what value flaw checking remains
      now that the wire owns the domain rules (for example whether a NaN
      component still flaws).
- [x] Delete `vox_bound.rs`; update `lib.rs`; drop
      `VoxValuePoolFlaw::Bound`, both of `error.rs`'s bound variants, and
      the `vox_main.rs` `validate` mapping arm. The two variants are
      `MalformedValuePoolBound` on the construction path and
      `ValuePoolBound` on the validate path, whose only constructor is
      that mapping arm. `Error` is public and not `non_exhaustive`, so an
      orphaned variant draws no warning and no gate grep matches it. The
      surviving `ValuePoolValue` doc still says "or outside its bounds".
- [x] Test fixtures across `vox_main.rs`, `vox_value_pool.rs`, and
      `vox_effective_palette.rs`; the bound, color range, and exactness
      tests delete with the machinery they cover.

Gate: `cargo test -p voxcore` green.

## Iteration 7: voxsmith

The largest iteration; three commits, refined further at execution if a
chunk outgrows review.

- [x] The voxj seam: `vox_value_pool_from_voxj_value_pool.rs` and
      `voxj_value_pool_from_vox_value_pool.rs` become one-to-one kind
      maps (the hex decode and the `ColorFormat` dispatch go);
      `convert/voxj/color_format.rs` deletes with
      `VoxjFileBuilder::color_format`, the `to_voxj_file.rs` reference,
      and the `internal/voxj/write_voxj.rs` plumbing;
      `from_voxj_file.rs` fixtures move to the new kinds; the independent
      transfer reference implementations in the write-side tests move to
      the boundary tests instead of deleting.
- [x] The linear pivot: `value_pool_color` accepts `Vec3Float` and
      `Vec4Float` and takes color-ness from its callers (seven call
      sites, five files), a 3-component value taking opaque alpha; every
      color value pool construction site moves to the vector constructors
      with the transfer applied at the boundary (the glTF, vmax, goxl,
      mvox, qbcl, and voxelize importers, `order_palette_colors.rs`,
      `resolve_cell_color.rs`, the atlas bake, the palette reduction, the
      material sampling, and the vmax writer's encode);
      `from_gltf_bytes.rs` stops sRGB-encoding glTF's linear factors and
      passes them through, which turns `MeshMaterial`'s `base_color` and
      `emissive_factor` from `TySrgbaU8` into linear and pulls
      `voxelize_mesh.rs`'s `[u8; 4]` dedup map and `material_key` with
      them. The transfer itself does not change: today's
      `into_format::<f64, f64>().into_linear()` pair already survives u8
      to linear to u8 as identity on all 256 codes, so no boundary moves
      to palette's u8 lookup tables. Alpha is never transfer-encoded
      anywhere.
- [x] The vocabulary range check, one new file (decide its home and log
      it): a single function walks every bound property and errors on any
      value outside that property's range, each range spelled exactly
      from [reference/gltf-ranges.md](reference/gltf-ranges.md), `ior`'s
      `{0} union [1, inf)` included; the glTF export runs it before
      writing and the glTF import runs it on what it read. Reconcile it
      with `gltf_attributes.rs`'s live `scalar_range`, which already is a
      per-property range table, already disagrees on `ior` (`1..`), and
      already feeds `voxelize_mesh.rs`'s `factor_value_pool` under the
      `--out-of-range-factor` clamp policy the new error-only check does
      not have. Decide whether the new function absorbs it or sits beside
      it, and log that.
- [x] The exactness test: all 256 values of `k/255` survive u8 to linear
      to u8 as identity, through the same conversions the importers and
      exporters call. It passes on today's transfer, so it lands as a
      regression guard rather than as a reason to change one.

Gate: `cargo test -p voxsmith --all-features` green. The bare
`cargo test -p voxsmith` omits `gltf` and the `_mesh` marker it alone
enables, so it runs 132 of 218 tests and compiles none of the glTF import
and export, the atlas bake, the material sampling, or the voxelizer this
iteration edits.

## Iteration 8: vxl

Closes the red window.

- [x] Delete `utilities/voxj_color_format.rs` and the voxj
      `--color-format` flag in `utilities/voxj_encoding_options.rs`, with
      the plumbing through `dependencies.rs`, `dependencies_impl.rs`,
      `to_voxj.rs`, `voxelize.rs`, and `write_voxj_document.rs`. The vmax
      `--color-format` stays.
- [x] Rename the voxelize `--out-of-range-factor` flag, its
      `out_of_range_factor.rs` file, and its help prose to the property
      vocabulary, following voxsmith's `OutOfRangeProperty` (owner
      ruling, 2026-08-03, in the decisions log).
- [x] `mesh_object.rs`: the channel kind comes from the property name
      (promote the existing `GltfAttributeKind::of` fallback over the
      kind switch); a `string` or `json` value pool still errors.
- [x] `palette_show.rs`: `classify(property_name, value_pool)` keys on
      the name per the glTF vocabulary; the sample, swatch, and alpha
      helpers follow; the swatch encodes linear to sRGB at display, alpha
      untouched; every idiomatic property renders exactly as it does
      today (color swatches with hex text and per-channel reads for
      `baseColor` and `emissiveColor`, grayscale swatches for the numeric
      properties); a custom key defaults to plain numbers, with `--type`
      asserting color for it (the flag returns; V2 had dropped it, per
      the decisions log).
- [ ] Fixtures and inline expectations across `palette_show.rs`,
      `palette_list.rs`, `info.rs`, and `hierarchy_show.rs`.

Gate: the whole workspace green: `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`.

## Iteration 9: one label vocabulary per command

Unrelated to the value-kinds work; it sits at the end so the fix never
rides a voxj commit, and lands in its own commit at the owner's request
(2026-08-02).

- [ ] One format decision across `hierarchy show`, `palette show`,
      `palette list`, and `info`: a field has one label, spelled the
      same in every layout, and no command relabels per layout.
      `palette show` (every label is data) and `hierarchy show` (one
      grid, hierarchy and JSON only) already comply. `palette list`
      does not: its tree/JSON grid says `materialCount` and `objects`
      where its tables grid says `materials` and `used by`. `info`
      does not: it splits a Title-Case display grid (`Properties`,
      `Has ext`) from its snake_case JSON grid (`properties`,
      `has_ext`). The recommendation is the machine labels everywhere,
      the convention the compliant two already follow; the owner picks
      the convention and the surviving label for each forked field at
      execution, fixtures re-baseline, and the ruling lands in the
      decisions log.

Gate: `cargo test -p vxl --all-features` green; owner review.

## Iteration 10: closeout

- [ ] Regenerate `submodules/tyt-assets/scratch/energy-turret.voxj` from
      its sibling `energy-turret.glb`: a tyt-assets commit plus the
      superproject gitlink bump. Verify the new file carries vector
      kinds, the renamed property names, and no `min`/`max` keys.
- [ ] Sweep the open vxl-commands plan pages: `reference/mesh.md` (the
      property names), `reference/to/voxj.md`, `reference/voxelize.md`,
      and `checklist.md` (the deleted `--color-format`),
      `reference/palette/remap.md` (the `srgba-hex` examples), and the
      eight further pages carrying the property vocabulary: `README.md`,
      `reference/conventions.md`, `reference/design-notes.md`,
      `reference/implementation-decisions.md`,
      `reference/palette/README.md`, `reference/palette/list.md`,
      `reference/palette/quantize.md`, and `reference/palette/show.md`.
- [ ] Run both gate greps from the ground rules; read any residue against
      the wire rule and clear or log it.
- [ ] Close the plan: status note at the top of the README and the move
      to `doc/plan/closed`.

Gate: greps clean, workspace green, owner sign-off. Close the plan.
