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
- The crate lives at `projects/utilities/treegrid` with dependency
  `branded-id`, an optional `json` feature gating `serde_json`
  (`preserve_order`) with the value JSON forms and the JSON layouts,
  an optional `ty-math` feature gating the typed-color
  constructors, and one default-on feature per layout, named for its
  render module (`render_hierarchy`, `render_rows`, `render_columns`,
  `render_tables`), gating that module -- no
  clap, libc, tyt-common, or tyt-injection. No IO
  `Dependencies` trait, no `impl` feature (pure math and pure
  serialization ride feature gates; the `TreeGridCells` cell policy
  is the one injection point).
  Publishable metadata like the sibling crates (license, repository,
  description); add to workspace `members` and `[patch.crates-io]`.
- House style: one public item per file named for it, render methods
  on extension traits, private `mod` + `pub use` re-exports, doc
  comments on public items, leaf-item imports.
- Parity phases (3, 4) end with byte-identical default command output.
  Phase 2 changes flag values but keeps the default (`rows` + `concat`)
  output byte-identical; its JSON change is deliberate and called out.
- Behavior questions are settled by the
  [rendering spec](reference/rendering-spec.md); if implementation
  disagrees with it, fix one of them and say which in
  implementation decisions.

## Phase 1: the crate

- [x] **S1. Scaffold and model.** Crate skeleton; `BTreeGridNode`,
      `TreeGrid` (append-only arena over `U32Id<BTreeGridNode>`, ordered
      roots), `TreeGridNode` (`label`, `annotation`, `format`, `values`,
      children), `TreeGridLabel::{Bare, Quoted}` with `bare` / `quoted`
      constructors, `TreeGridValue` (`text` + optional `swatch`) with
      the `json`-gated `TreeGridJsonValue` pairing (the native JSON
      form beside a value; core typed constructors `int` / `float` /
      `unorm` / `unorm8` / `bool` / `json` / `srgb8` / `srgba8` per
      the [rendering spec](reference/rendering-spec.md) table, and
      the `new` + `with_swatch` / `with_json` escape hatch),
      `TreeGridSwatch`
      (with the canonical `render`), `TreeGridCellFormat`, the
      `TreeGridCells` cell policy with the default `TreeGridValueCells`
      and the `json`-gated `TreeGridJsonCells` plus
      `TreeGridJsonValueCells`, `TreeGridVisual`,
      `TreeGridLabelMode`, `TreeGridOptions` (with
      `Default`), `TreeGridError`. Builder API: `add_root`, `add_child`,
      `node_mut`, `push_value`. Unit tests for the builder and label
      quoting.
- [x] **S2. Text machinery and cells.** Private modules ported from vxl:
      `visible_width` / `pad_right`, quote formatting, tree glyphs,
      markdown table core with `md_cell` escaping. Cell rendering: the
      resolved-format x visual matrix and the `Visual` strip rule from
      `palette_show::render_cell` / `abuts` (the swatch escape builder
      landed at S1 as `TreeGridSwatch::render`). The `ty-math` feature
      and its typed-color constructors
      (`srgb` / `srgba` / `lin_rgb` / `lin_rgba` over the
      component-generic family, f32 / f64): functional-notation text,
      number-array JSON, quantized (for `lin_*`, transfer-encoded)
      swatch bytes, all color math through ty-math. Tests: the matrix,
      the strip with a visual-less value mixed in, visible width past
      CSI sequences, an HDR `lin_rgba` component above 1.
- [x] **S3. `hierarchy` layout.** `render_hierarchy`:
      connectors/extensions, `bare_roots`
      both ways, annotations, `label: cells` data lines, values-on-branch
      nodes, and the `value_children` mode (one connector line per
      value).
      Golden tests shaped on `palette list --layout hierarchy`,
      vmax trees (connectored roots, `(Group)` annotations, marker
      nodes), a section-style forest (`root` / `unplaced` bare
      roots), and a value-children palette tree.
- [x] **S4. `rows` and `columns` layouts.** `render_rows` /
      `render_columns`: label modes `none` / `concat`
      / `header`, per-group label padding, blank-line separation,
      right-trimming, `width` wrapping with continuation indent
      (rows only). `header` mode is nested headings via the shared
      depth-first grouping walk, level `header_level + depth`;
      the level carried by `header` labels (default 1; a heading past
      level 6 renders as a bold `**label**` line; the flag on a
      headerless render -> `HeaderLevelWithoutHeaders` from
      `resolve`). Port the `row` / `row-no-header` /
      `column` / `column-no-header` / wrapping golden tests from
      `implementation/palette_show.rs`; add `header`-mode tests
      including headerless root-level data, a nested two-level parent
      path (`#` then `##`), a non-default `header_level`, and the
      past-6 bold fallback.
- [x] **S4b. Render extension traits and per-layout features.** Owner
      restructure (2026-07-20). Each layout becomes its own module
      holding an extension trait on `TreeGrid`
      (`TreeGridRenderHierarchy`, `TreeGridRenderRows`,
      `TreeGridRenderColumns`; `TreeGridRenderTables` joins at S5 and
      `TreeGridRenderJson` at S6), its options payload, and its
      `resolve_*` impl, gated by a default-on cargo feature
      (`hierarchy`, `rows`, `columns`, `tables`; `json` and `ty-math`
      stay non-default), with `render/` reduced to shared
      crate-private machinery and no type changing shape with a
      feature. One pub item per file, named for it, across the crate:
      split `text_width.rs` (`visible_width` / `pad_right`),
      `markdown_table.rs` (`md_cell` out), `group.rs` (`data_paths` /
      `groups` out); fold the tree glyphs into the hierarchy render
      as private consts; rename `json/tree_grid_options.rs` to
      `resolve_json.rs`. Impl-only constructor-family files
      (`color/`) keep the extended type's name as the conforming
      reading. Amend CLAUDE.md's style rule, the plan README, and
      the rendering spec, and log the decision.
- [x] **S5. `tables` layout.** The shape types landed with the options
      restructure (`TreeGridTableShape`: `Nested` default with its
      label and level payload, `Flat`; `Records` joins at S15); this
      step is `render_tables`. Nested: one table per
      parent-path group (`#` index column, one column per data node,
      leaf-label headers, blank cells past shorter series), under
      nested headings carrying the full path (`concat`) or the leaf
      segment (`header`). Flat: one table over every data node with
      concat-path headers, the comparison view (the invalid label and
      shape combinations are rejected by `resolve`, tested with the
      restructure). Goldens: grouped tables in
      both label modes including the hierarchy-shaped worked example
      from the [rendering spec](reference/rendering-spec.md), and the
      flat comparison table.
- [x] **S6. JSON layouts.** Behind the `json` feature,
      `render_json_pretty` / `render_json_compact`: the record
      envelope (`label`, optional `annotation` / `values` /
      `children`), pretty and compact, trailing newline. Tests:
      envelope shape, native value types, duplicate
      sibling labels surviving, pretty/compact value equality.

## Phase 2: `vxl palette show` adopts (breaking flags + JSON)

- [x] **S7. Adoption.** Sampling keeps producing `Sample`-shaped data,
      now as `TreeGridValue`s; build the forest (palette `Bare` ->
      property `Quoted` -> optional component `Bare`; per-collection
      format on the data node). A scalar property's data node sets
      `annotation: "(scalar)"`: the text layouts suffix it (chunk 1,
      2026-07-21) and the envelope's `annotation` field replaces the
      bespoke `scalar: true` flag. Map pools to the typed constructors
      (`srgba8` for sRGB colors, `unorm8` for components, `float` for
      numbers); linear colors keep their space-joined text through the
      escape hatch until the S17 notation flip. Decide the
      number-pool gray-swatch rule (today every number gray-swatches as
      if `0..1`; keep that for parity or key `unorm` off the pool's
      declared bounds -- `auto` output is identical either way) and log
      it. Replace `PaletteShowLayout` values with
      `hierarchy | rows | columns | tables | json-pretty | json-compact`
      (default `rows`), add `--label none | concat | header` (default
      `concat`), `--header-level` (heading-emitting renders only,
      headings start at `#` when unset), and `--table-shape`
      (`nested` default | `flat`; `records` joins at S15), keep
      `--width` and its terminal resolution in vxl. Note in the commit
      message that `tables` defaults to a grouped redesign of
      `markdown`, and that the old interleaved table is now
      `--table-shape flat`.
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
      in Rust (matching logic already lives there; the closure comes
      from `TreeSelection`, S18, pulled forward if this phase lands
      first), build the `TreeGrid`, render, and delete the
      tree-printing half of `FBX_HIERARCHY_PY`. Builder tests against
      fixture JSON so the output is testable without Blender.

## Phase 6: record tables and consistency (committed)

Owner condition on the series-first tables call (2026-07-12): this phase
is in scope, not optional. The plan does not close without it.

- [ ] **S15. Record-shaped tables and the last adopters.** Add
      `Records` to `TreeGridTableShape`: rows are one branch's
      children, columns the union of relative flattened descendant
      paths plus the row's own value; settle column naming,
      multi-valued cells, and heterogeneous-children sparsity here,
      against the `info` / `validate` / `palette list` markdown + JSON
      adoption; add `records` to the adopters' `--table-shape`. Retires
      vxl's `markdown_table` and `to_json_string`; `info`'s
      `##`-sectioned markdown becomes `tables` + `header` with
      `header_level` 2 beneath its command-printed `# {input}` title.
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

## Phase 7: tree-selection closure in pathspec (committed, closes the plan)

The one shared win from the query-layer investigation (2026-07-14, see
README decision 11): the selected / visible / match-roots closure that
`tyt vmax hierarchy` and `vxl hierarchy show` each hand-roll and S14
would otherwise write a third time. It lives in `pathspec`, not
treegrid, so the crate keeps its no-selection boundary. Independent of
every other phase; land it any time, and pull it forward if phase 5
runs first.

- [ ] **S18. `TreeSelection`.** A new pathspec public struct:
      `selected` / `visible` per-node flag vectors plus `match_roots`
      indices, built by `from_matches(matched, parents)`, with
      `matched` as `match_paths` / `match_subtrees` return it and
      `parents` a per-node parent index (`None` at a root). Semantics:
      mark each selected node's ancestor chain visible; a selected
      node whose parent is unselected is a match root (the rule both
      existing implementations use, which differs from "no selected
      ancestor" when a `!` pattern deselects a middle node). Unit
      tests: a root match, a nested match, several match roots, and
      the deselected-middle-node divergence case.
- [ ] **S19. Adopt at both call sites.** One commit per site,
      byte-identical output:
      1. `tyt vmax hierarchy`: scatter the `match_subtrees` flags over
         the full node index range, then replace the hand-built
         `selected` / `visible` / `match_roots` sets in `select_nodes`.
      2. `vxl hierarchy show`: record a parent index per placement in
         `enumerate_placements`, wrap a `TreeSelection` in `Filter`,
         and delete the path-prefix set assembly in `build_filter`.

## Deferred

- [ ] Publish `treegrid` 0.1.0 to crates.io before any dependent crate
      cuts a release (local builds ride `[patch.crates-io]` meanwhile).
- [ ] Decide whether `hierarchy show` / vmax / fbx expose `rows` /
      `columns` / `tables`, once someone wants them.
