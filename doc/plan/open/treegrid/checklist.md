# treegrid implementation checklist

Tracks building the crate and migrating the adopters. Start from the
[README](README.md) for the design, the
[rendering spec](reference/rendering-spec.md) for exact layout behavior,
and the [design notes](reference/design-notes.md) for rationale. Log
code-level decisions in
[implementation decisions](reference/implementation-decisions.md). Check
items off as they land.

## Ground rules

- One reviewable chunk per step. `cargo fmt --all`, `cargo clippy
  --workspace --all-targets -- -D warnings`, and `cargo test` before each
  commit; the pre-commit hook enforces fmt + clippy.
- The crate lives at `projects/utilities/treegrid` with dependencies
  `branded-id` and `serde_json` (`preserve_order`), plus an optional
  `ty-math` feature gating the typed-color constructors -- no clap,
  libc, tyt-common, or tyt-injection. No `Dependencies` trait, no
  `impl` feature (pure math rides the feature gate, not DI).
  Publishable metadata like the sibling crates (license, repository,
  description); add to workspace `members` and `[patch.crates-io]`.
- House style: one public type per file, private `mod` + `pub use`
  re-exports, doc comments on public items, leaf-item imports.
- Parity phases (3, 4) end with byte-identical default command output.
  Phase 2 changes flag values but keeps the default (`rows` + `concat`)
  output byte-identical; its JSON change is deliberate and called out.
- Behavior questions are settled by the
  [rendering spec](reference/rendering-spec.md); if implementation
  disagrees with it, fix one of them and say which in
  implementation decisions.

## Phase 1: the crate

- [ ] **S1. Scaffold and model.** Crate skeleton; `BTreeGridNode`,
      `TreeGrid` (append-only arena over `U32Id<BTreeGridNode>`, ordered
      roots), `TreeGridNode` (`label`, `annotation`, `format`, `values`,
      children), `TreeGridLabel::{Bare, Quoted}` with `bare` / `quoted`
      constructors, `TreeGridValue` (`text` + optional `json` + optional
      `swatch`; core typed constructors `int` / `float` / `unorm` /
      `unorm8` / `bool` / `json` / `srgb8` / `srgba8` per the
      [rendering spec](reference/rendering-spec.md) table, and the
      `new` + `with_json` / `with_swatch` escape hatch), `TreeGridSwatch`,
      `TreeGridCellFormat`,
      `TreeGridLayout`, `TreeGridLabelMode`, `TreeGridOptions` (with
      `Default`), `TreeGridError`. Builder API: `add_root`, `add_child`,
      `node_mut`, `push_value`. Unit tests for the builder and label
      quoting.
- [ ] **S2. Text machinery and cells.** Private modules ported from vxl:
      `visible_width` / `pad_right`, quote formatting, tree glyphs,
      markdown table core with `md_cell` escaping. Cell rendering: the
      format x swatch matrix and the abutting rule from
      `palette_show::render_cell` / `abuts`, with the swatch escape
      builders. The `ty-math` feature and its typed-color constructors
      (`srgb` / `srgba` / `lin_rgb` / `lin_rgba` over the
      component-generic family, f32 / f64): functional-notation text,
      number-array JSON, quantized (for `lin_*`, transfer-encoded)
      swatch bytes, all color math through ty-math. Tests: the matrix,
      abutting with a swatchless value mixed in, visible width past CSI
      sequences, an HDR `lin_rgba` component above 1.
- [ ] **S3. `hierarchy` layout.** Connectors/extensions, `bare_roots`
      both ways, annotations, `label: cells` data lines, values-on-branch
      nodes. Golden tests shaped on `palette list --layout hierarchy`,
      vmax trees (connectored roots, `(Group)` annotations, marker
      nodes), and a section-style forest (`root` / `unplaced` bare
      roots).
- [ ] **S4. `rows` and `columns` layouts.** Label modes `none` / `concat`
      / `header`, per-group label padding, blank-line separation,
      right-trimming, `width` wrapping with continuation indent
      (rows only), `header_level` (default 2, `1..=6` else
      `HeaderLevelOutOfRange`; set on a headerless render ->
      `HeaderLevelWithoutHeaders`). Port the `row` / `row-no-header` /
      `column` / `column-no-header` / wrapping golden tests from
      `implementation/palette_show.rs`; add `header`-mode tests including
      a headerless root-level group, a two-level parent path, and a
      non-default `header_level`.
- [ ] **S5. `tables` layout (series shape).** `#` index column, one
      column per data node, per-group tables under `header`, blank cells
      past shorter series, `none` -> `TreeGridError::LabelNoneWithTables`.
      Port the markdown golden test; add a `header`-grouped two-table
      test.
- [ ] **S6. JSON layouts.** The record envelope (`label`, optional
      `annotation` / `values` / `children`), pretty and compact, trailing
      newline. Tests: envelope shape, native value types, duplicate
      sibling labels surviving, pretty/compact value equality.

## Phase 2: `vxl palette show` adopts (breaking flags + JSON)

- [ ] **S7. Adoption.** Sampling keeps producing `Sample`-shaped data,
      now as `TreeGridValue`s; build the forest (palette `Bare` ->
      attribute `Quoted` -> optional component `Bare`; per-collection
      format on the data node). Map pools to the typed constructors
      (`srgba8` for sRGB colors, `unorm8` for components, `float` for
      numbers); linear colors keep their space-joined text through the
      escape hatch until the S17 notation flip. Decide the
      number-pool gray-swatch rule (today every number gray-swatches as
      if `0..1`; keep that for parity or key `unorm` off the pool's
      declared bounds -- `auto` output is identical either way) and log
      it. Replace `PaletteShowLayout` values with
      `hierarchy | rows | columns | tables | json-pretty | json-compact`
      (default `rows`), add `--label none | concat | header` (default
      `concat`) and `--header-level` (`1..=6`, valid only with
      `--label header`, headers default to `##` when unset), keep
      `--width` and its terminal resolution in vxl.
      Delete `render*`, `wrap_cells`, `assemble_row`, `join_padded`,
      `render_cell`, `color_swatch` / `gray_swatch`, `abuts`. Keep
      selector/sampling tests in vxl plus a few end-to-end renders; the
      layout goldens now live in treegrid. Commit as `feat(vxl)!` with
      the old->new flag mapping in the message.
- [ ] **S8. Docs.** Update
      [vxl-commands palette/show.md](../vxl-commands/reference/palette/show.md)
      (layouts, `--label`, the JSON envelope; check off its deferred
      envelope item) and
      [conventions.md](../vxl-commands/reference/conventions.md) item 5.

## Phase 3: vxl tree renderers adopt (parity)

- [ ] **S9. `hierarchy show`.** Keep `Scene` / placements / `Filter` /
      view math; replace `Walk`'s string assembly with a `TreeGrid` builder
      (sections as bare roots; the `{node: 0}` / `{object: 0, instance:
      1}` / `{materials: 10}` tags as node *values*, not annotations;
      `ancestors` / `descendants` markers and the view subtrees as `Bare`
      nodes with pre-formatted values like `position: [12.5, 0.5,
      10.0]`). Byte-identical output; existing render tests keep passing
      unchanged.
- [ ] **S10. `hierarchy show --layout`.** Expose `hierarchy` (default) |
      `json-pretty` | `json-compact` now that the tree is data. Update
      the vxl-commands hierarchy reference pages and conventions item 5
      ("prints only its tree" is no longer true).
- [ ] **S11. `palette list --layout hierarchy`.** Build the `palettes`
      tree (bare root, index `Bare` children, field leaves) and render
      through the crate; markdown and JSON stay bespoke until phase 6
      (S15). Attribute and object names print bare today
      (82e803a skipped this command): decide `Bare` for parity versus
      `Quoted` to normalize quoting, and log it (see design notes 4b).
      Delete vxl's `tree_glyphs` once nothing imports it.

## Phase 4: `tyt vmax hierarchy` adopts (parity)

- [ ] **S12. Adoption.** Keep scene load, `select_nodes`, transform
      resolution, and all flag parsing; replace `Renderer` with a
      `TreeGrid` builder (connectored roots, `(Group)` / `(Object)`
      annotations, `transform` / `bounds` `Bare` subtrees, `ancestors`
      markers). vmax never quotes names: decide `Bare` for parity versus
      `Quoted` to normalize quoting (design notes 4b). Byte-identical
      output apart from that call. Optionally expose `--layout
      hierarchy | json-pretty | json-compact` in a follow-up commit.

## Phase 5: `tyt fbx hierarchy` adopts (severable)

- [ ] **S13. Data over the Blender boundary.** Extend
      `FBX_HIERARCHY_JSON_PY` to emit the object hierarchy with
      per-object payloads: type, and the transform / bounds / extents
      values computed in Blender for the requested
      space/unit/precision/scale (the math stays where the evaluated
      scene is).
- [ ] **S14. Render in Rust.** Parse the payload, apply select/collapse
      in Rust (matching logic already lives there), build the `TreeGrid`,
      render, and delete the tree-printing half of `FBX_HIERARCHY_PY`.
      Builder tests against fixture JSON so the output is testable
      without Blender.

## Phase 6: record tables and consistency (committed -- closes the plan)

Owner condition on the series-first tables call (2026-07-12): this phase
is in scope, not optional. The plan does not close without it.

- [ ] **S15. Record-shaped tables and the last adopters.**
      `TreeGridTableShape::Records` (explicit render option) and the
      `info` / `validate` / `palette list` markdown + JSON adoption,
      retiring vxl's `markdown_table` and `to_json_string`; `info`'s
      `##`-sectioned markdown becomes `tables` + `header`.
- [ ] **S16. Layout-name consistency.** `markdown -> tables`,
      `pretty-json -> json-pretty`, `compact-json -> json-compact`
      across `list` / `info` / `validate`, one breaking commit, retiring
      vxl's `ReportLayout` clap enum; update conventions.md.
- [ ] **S17. Linear-color notation flip.** Flip `vxl palette show`'s
      linear colors from space-joined text (today `2 1 0.5 1`,
      preserved through the escape hatch in S7) to the `lin_rgb` /
      `lin_rgba` constructors, adopting the `lrgb(...)` / `lrgba(...)`
      functional notation the color feature renders (S2);
      `rgb(...)` / `rgba(...)` likewise should a float-form sRGB
      rendering ever appear (8-bit sRGB keeps its `#RRGGBB(AA)` hex).
      Text only -- the JSON form stays the native number array. Updates
      `a_linear_color_renders_float_components` and the
      [palette show reference](../vxl-commands/reference/palette/show.md)
      wording.

## Deferred

- [ ] Publish `treegrid` 0.1.0 to crates.io before any dependent crate
      cuts a release (local builds ride `[patch.crates-io]` meanwhile).
- [ ] Decide whether `hierarchy show` / vmax / fbx expose `rows` /
      `columns` / `tables`, once someone wants them.
