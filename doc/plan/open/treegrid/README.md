# treegrid: one hierarchical-data renderer for the read commands

Status: **open.** Nothing has landed; this document is the design.

Four commands orbit the same idea -- a hierarchical collection whose nodes
carry data, rendered as text -- and each one re-implements the rendering:

1. `vxl hierarchy show`
   (`projects/utilities/vxl/src/implementation/hierarchy_show.rs`): a
   box-glyph tree over the voxel-json scene graph, with per-node data
   subtrees (`transform`, bounds, origins, extents, layers, voxel counts),
   instancing and unplaced markers, and collapse stubs. Tree only, no
   `--layout`.
2. `vxl palette show`
   (`projects/utilities/vxl/src/implementation/palette_show.rs`): labeled
   value collections (an attribute's values down a palette) under
   `--layout row | row-no-header | column | column-no-header | markdown |
   pretty-json | compact-json`, with ANSI color swatches, visible-width
   alignment, and `--width` wrapping.
3. `tyt vmax hierarchy` (`projects/tyt/tyt-vmax/src/commands/hierarchy.rs`):
   its own tree renderer over the Voxel Max scene, with `transform` and
   `bounds` subtrees and `ancestors` / `descendants` stubs.
4. `tyt fbx hierarchy` (`projects/tyt/tyt-fbx/src/commands/hierarchy.rs`):
   the same shape again, except the tree is rendered inside a Blender
   Python script; the Rust side only packs flags.

Beside them, `vxl palette list`, `vxl info`, and `vxl validate` render
markdown tables and JSON reports from the same kind of data, sharing
`markdown_table`, `tree_glyphs`, `text_width`, `quote_name`, and
`to_json_string` inside vxl's `implementation/` where no other crate can
reach them.

This plan extracts one shared crate: a library you **populate** with a
hierarchy of labeled, data-bearing nodes, then **render** under a chosen
layout and label mode. Selection (`--select` / `--select-index`, glob
filters), value sampling, number formatting, and IO all stay in the
commands; the crate only arranges and serializes what it is handed.

## The model in one paragraph

A tree grid is a forest of nodes in a branded-id arena, voxcore-style
(`U32Id<BTreeGridNode>` into dense storage; compare
`projects/utilities/voxcore/src/vox_hierarchy_node.rs` /
`vox_runtime_state.rs`). Every node has a label (a single path segment,
either `Bare`, printed as-is for trusted identifiers like `transform` or
`0`, or `Quoted`, printed `{:?}`-quoted for user-entered strings like
`"baseColorFactor"`), an optional
annotation (a hierarchy-layout-only verbatim suffix, vmax's `(Group)`;
`hierarchy show`'s `: {node: 0}` tags are instead ordinary node values),
an ordered list of values, and children -- and any node may have both
values and children. A value carries its display text, its native JSON form, and
an optional swatch (truecolor or grayscale), exactly the shape of
`palette_show`'s `Sample`; a per-node cell format (`auto` / `swatch` /
`swatch-value` / `text`) picks how a value renders to a cell. One
`render(&TreeGridOptions) -> Result<String, TreeGridError>` call arranges
the same populated grid as a `hierarchy` tree, `rows`, `columns`, series
`tables`, or `json-pretty` / `json-compact`, and a label mode (`none` /
`concat` / `header`) decides how the text layouts spend the ancestor path.

## Design

### The crate

`projects/utilities/treegrid`, published like the other utilities crates.
"Tree grid" is the term of art for exactly this widget concept -- a
hierarchy whose nodes carry data columns (the ARIA role is `treegrid`) --
and a standalone word matches the voxcore / voxsmith / pathspec style.
Unclaimed on crates.io as of 2026-07-12, as were the checked alternates
(`treetable`, `outliner`, `datatree`, `showtree`, `treeport`, and the
draft names `ty-report` / `ty-view`); the owner chose `treegrid` for
genericity (2026-07-12).

Dependencies: `branded-id` and `serde_json` (with `preserve_order`), plus
an optional `ty-math` feature gating the typed-color value constructors
(vxl enables it at no cost; it already depends on ty-math). No clap, no
libc, no tyt-common. No `Dependencies` trait and no `impl` feature: the
one optional capability is pure math, so it rides a feature gate, not
DI. Public types
follow house style: one per file, `TreeGrid` prefix (`TreeGrid`,
`TreeGridNode`, `TreeGridLabel`, `TreeGridValue`, `TreeGridSwatch`,
`TreeGridCellFormat`, `TreeGridLayout`, `TreeGridLabelMode`,
`TreeGridTableShape`, `TreeGridOptions`, `TreeGridError`,
`BTreeGridNode`).

### Populate, then render

```rust
let mut grid = TreeGrid::default();

// 0."baseColorFactor".a  =>  Bare("0") / Quoted("baseColorFactor") / Bare("a")
let palette = grid.add_root(TreeGridLabel::bare("0"));
let attribute = grid.add_child(palette, TreeGridLabel::quoted("baseColorFactor"));
let component = grid.add_child(attribute, TreeGridLabel::bare("a"));

grid.node_mut(component).format = TreeGridCellFormat::Text;
grid.push_value(component, TreeGridValue::unorm8(255)); // text "255", json 255, gray swatch
grid.push_value(component, TreeGridValue::unorm8(128));

let output = grid.render(
    &TreeGridOptions::default()
        .with_layout(TreeGridLayout::Rows)
        .with_label(TreeGridLabelMode::Concat)
        .with_width(80),
)?;
// 0."baseColorFactor".a 255 128
```

The arena is append-only and single-parent by construction (`add_child`
attaches at creation; there is no re-parenting), so the forest cannot
cycle and render never needs a visited set. DAG sources like the voxel
scene graph expand instancing into placements *before* populating, exactly
as `hierarchy show` already does.

### Layouts

`TreeGridLayout`, with the semantics specified precisely in
[reference/rendering-spec.md](reference/rendering-spec.md):

1. `hierarchy`: the box-glyph tree. Annotations show, values print inline
   after `label: `, `bare_roots` chooses whether roots take connectors
   (vmax-style) or print as bare section headers (`root` / `unplaced` /
   `palettes`-style).
2. `rows`: each data-bearing node is one row, blank line between rows,
   only the label column padded, `width` wraps with continuation indent --
   today's `palette show --layout row`.
3. `columns`: each data-bearing node is one padded column under its label
   -- today's `column`.
4. `tables`: aligned markdown tables led by a `#` index column, shaped
   by `TreeGridOptions::table_shape`: `nested` (default) groups one
   table per parent path under `concat` or `header` headings; `flat`
   keeps today's `markdown` -- one table over everything with concat
   column headers, the cross-palette comparison view; `records`
   (phase 6, committed scope) transposes to one row per entity with
   relative-path columns, what `info` and `palette list` need -- see
   [design notes](reference/design-notes.md).
5. `json-pretty` / `json-compact`: the generic envelope, one record per
   node: `{"label", "annotation"?, "values"?, "children"?}`.

Data-bearing nodes enumerate in pre-order everywhere, so all layouts agree
on order.

### Label modes

`TreeGridLabelMode`, consumed by `rows`, `columns`, and `tables`; the
`hierarchy` and JSON layouts carry the labels structurally and reject
a set mode:

1. `none`: no labels. Errors under `tables`, which cannot head its columns
   with nothing.
2. `concat` (default): the full path joined with `.`, each `Quoted`
   segment quoted -- `0."baseColorFactor".a`. Inline on `rows` /
   `columns`, matching the current `row` / `column` headers (quoting
   landed in 82e803a); on `tables`, headings nest exactly like `header`
   -- same positions, same increasing levels -- but each carries its
   full path.
3. `header`: the ancestor chain becomes nested markdown headings --
   `# root`, `## "energy-tank-1"`, `### transform` -- one per branch
   segment that leads to data, depth-first, with each group's rows /
   columns / table under its heading labeled by leaf segment alone.
   Root-level data nodes print first with no heading.
   `TreeGridOptions::header_level` (default `1`) sets the
   shallowest heading's level so embedded output sits at the right
   depth under a host document's headings -- exposed as `--header-level`
   on adopting commands, and an error when set on a render that emits
   no headings rather than a silent no-op; a heading that nests past
   level `6` renders as a bold label (`**label**`) rather than a
   deeper `#` run. This fixes the motivating
   case: `palette show --layout markdown` on
   `tyt-assets/src/vmax/energy-reactor.vmax` today emits one 16-column
   table interleaving palettes 0 and 1; under `tables` it becomes two
   per-palette tables under `# 0` and `# 1`.

### Boundaries -- what stays in the commands

- **Selection.** `--select`, `--select-index`, glob filters, collapse
  logic: commands filter first and populate only what shows. `ancestors` /
  `descendants` / instance stubs are ordinary `Bare` nodes the command
  inserts. The one shared piece of that upstream logic, the
  selected / visible / match-roots closure, gets a home beside the
  matchers in `pathspec` at phase 7 (`TreeSelection`, decision 11);
  treegrid itself never selects.
- **Sampling and policy formatting.** Pool classification, precision
  (`fmt3`), space/unit conversion: anything with a knob stays upstream
  and arrives as finished text. The typed constructors own only
  knob-free canonical rendering: `Display` numbers, `#RRGGBB(AA)` hex,
  and the `rgba(...)` / `lrgba(...)` functional color notation.
- **IO.** Render returns a `String`; `write_stdout` stays put. Terminal
  width detection (libc ioctl) stays in vxl; the library takes
  `width: Option<usize>`.
- **clap.** The library exposes plain enums; each command keeps its own
  `ValueEnum` and maps, the `FillMode` / `MaterialMode` pattern. Commands
  expose only the layouts that make sense for them (e.g. `hierarchy show`
  starts with `hierarchy` + the JSON pair).

## CLI surface changes

`vxl palette show` is the breaking adopter (`feat(vxl)!`), consolidating
the no-header variants into `--label`:

| today                        | becomes                          |
| ---------------------------- | -------------------------------- |
| `--layout row` (default)     | `--layout rows` (default)        |
| `--layout row-no-header`     | `--layout rows --label none`     |
| `--layout column`            | `--layout columns`               |
| `--layout column-no-header`  | `--layout columns --label none`  |
| `--layout markdown`          | `--layout tables --table-shape flat` |
| `--layout pretty-json`       | `--layout json-pretty`           |
| `--layout compact-json`      | `--layout json-compact`          |
| (new)                        | `--label none \| concat \| header` |
| (new)                        | `--header-level`, heading-emitting renders only |
| (new)                        | `--table-shape nested \| flat` (`records` at S15) |
| (new)                        | `--layout hierarchy`             |

Default output (`rows` + `concat`) stays byte-identical. The JSON payload
changes from the bespoke `[{palette, attribute, values}]` records to the
generic envelope -- the "one shared JSON envelope across the read
commands" the [vxl-commands plan](../vxl-commands/reference/palette/show.md)
deferred; see the [design notes](reference/design-notes.md) for why the
envelope must be records rather than label-keyed objects.

The other vxl read commands (`list`, `info`, `validate`) keep their
current flags until the phase 6 consistency pass renames
`markdown -> tables` and `pretty-json -> json-pretty` /
`compact-json -> json-compact` in one breaking commit.

## Adoption and blast radius

| adopter | keeps | replaces | phase |
| --- | --- | --- | --- |
| `vxl palette show` | selectors, pool classification, sampling, `Width` | all of `render*`, `wrap_cells`, `assemble_row`, swatch fns (~350 lines) | 2 |
| `vxl hierarchy show` | `Scene`, placements, `Filter`, view math | `Walk`'s tree drawing | 3 |
| `vxl palette list` | selection, field gathering | `render_hierarchy` + `tree_glyphs` | 3 |
| `tyt vmax hierarchy` | scene load, `select_nodes`, transform resolve | `Renderer` | 4 |
| `tyt fbx hierarchy` | flag parsing, Blender data extraction | the tree-printing half of `FBX_HIERARCHY_PY` | 5 |
| `vxl info` / `validate` / `list` tables + JSON | -- | `markdown_table`, `to_json_string` | 6 |

Phases 2-4 each end with byte-identical default output (only flag values
change in phase 2). Phase 5 is the big one -- `tyt fbx hierarchy` renders
inside Blender today, so adoption moves data (not text) across the
process boundary; it is severable and can slip without blocking anything
else. Phase 6 (record tables and the consistency pass) is committed
scope. Phase 7 is a small coda outside the crate: `pathspec` gains the
`TreeSelection` closure (decision 11), `tyt vmax hierarchy` and
`vxl hierarchy show` drop their hand-rolled copies so lines go down,
and S14 builds on it rather than writing a third; it closes the plan.

## Decisions

1. **Owned arena, not a visitor trait.** "Populate with data, then choose
   to render" is the requested shape; it also keeps DAG/instancing
   expansion in the commands that understand it.
2. **Branded ids, voxcore-style storage.** `U32Id<BTreeGridNode>` into an
   append-only arena; no SoA pools needed since nodes are never removed.
3. **One label segment per node; suffixes are child nodes.** A composite
   leaf like `baseColorFactor.a` is `Quoted("baseColorFactor")` with a
   `Bare("a")` child, so concat, header grouping, and the hierarchy tree
   all fall out of one structure.
4. **`Bare` vs `Quoted` labels.** The variants name the rendering
   effect, not the provenance: `Bare` prints as-is (trusted
   identifiers), `Quoted` prints `{:?}`-escaped (`quote_name`
   semantics, for user-entered strings) in every text layout. JSON gets
   the raw string either way -- which is why quoting is a label flag
   rather than something callers bake into the string.
5. **Values are pre-rendered, JSON is optional, constructors are typed
   by domain.** `TreeGridValue { text, json: Option<Value>, swatch }` plus
   a per-node `TreeGridCellFormat`; the `auto`/`swatch`/`swatch-value`/
   `text` matrix and the swatch-strip abutting rule port from
   `palette_show::render_cell` / `abuts` unchanged. Text cannot derive
   from JSON (Rust `Display` versus serde's ryu float rendering,
   precision-rounded `fmt3` text versus full-fidelity numbers), so both
   are stored -- but the typed constructors fill every field from one
   domain-shaped argument: `int(i64)`, `float(f64)` (integral-collapse
   rule), `unorm(f64)` and `unorm8(u8)` (the two gray-swatch domains),
   `bool`, `json`, and `srgb8` / `srgba8` (bytes to hex + color
   swatch). A value built with just `new(text)` falls back to
   `String(text)` in the JSON layouts, so tree-only adopters never
   touch the JSON field; `with_json` / `with_swatch` are the escape
   hatch for genuinely divergent pairs.
6. **Float-component colors ride the `ty-math` feature.** `srgb` /
   `srgba` / `lin_rgb` / `lin_rgba` constructors take the
   component-generic ty-math color family (T = f32 / f64), rendering
   `rgb(...)` / `rgba(...)` / `lrgb(...)` / `lrgba(...)` functional
   text, native number-array JSON, and a quantized (for `lin_*`,
   transfer-encoded) swatch. Feature gate rather than DI because the
   transfer function is pure math, and rather than reimplementation
   because ty-math's CSS Color 4 out-of-gamut handling must not fork.
7. **JSON is a record envelope**, `{"label", "annotation"?, "values"?,
   "children"?}`, because sibling labels can repeat (two `door` objects)
   and arbitrary labels would collide with structural keys under
   object-keyed nesting.
8. **Table shapes: `Nested` and `Flat` first, `Records` committed.**
   `TreeGridTableShape` is `{Nested, Flat, Records}`: `Nested` is the
   grouped default, `Flat` keeps today's cross-palette comparison table
   as an explicit shape from S5, and `Records` -- one row per entity,
   columns the relative flattened descendant paths; what `info` and
   `palette list --layout markdown` render -- lands in phase 6. Always
   chosen explicitly, never auto-detected from value counts, which
   would flip on a one-material palette. Phase 6 is committed scope,
   not optional: the owner accepted records-later on the condition that
   the shape lands before the plan closes (2026-07-12).
9. **`bare_roots` render option** reconciles the two existing top-level
   styles: connectored roots (vmax, collapsed-ancestors lists) versus bare
   section headers (`root` / `unplaced` in `hierarchy show`, `palettes` in
   `palette list`).
10. **Layout value names**: `hierarchy`, `rows`, `columns`, `tables`,
   `json-pretty`, `json-compact`; label modes `none`, `concat`, `header`.
   `json-*` prefixes group the serializations together in `--help` and
   completions.
11. **The tree-selection closure lands in `pathspec`, not treegrid**
   (2026-07-14). A query/model-crate split (populate a typed tree
   once, then select and collapse against it) was investigated and
   rejected: the glob engine and index grammar are already shared,
   the residue is per-command policy, and every read command runs one
   selection known at parse time, so filter-then-populate stays. The
   one shared win is the selected / visible / match-roots closure,
   duplicated by `tyt vmax hierarchy` and `vxl hierarchy show` and
   needed a third time at S14. It is selection, so it lives beside
   the matchers in `pathspec` as `TreeSelection` (phase 7), and
   treegrid keeps its no-selection boundary.

## Documents

- [Implementation checklist](checklist.md): the phased task list. Start
  here when implementing.
- [Continue prompt](continue-treegrid.md): the per-session resume
  prompt; point a fresh session at it to advance the plan by one
  reviewable chunk.
- [Rendering spec](reference/rendering-spec.md): the exact contract for
  every layout, label mode, and the value/format matrix. Any behavior
  question is settled here.
- [Design notes](reference/design-notes.md): rationale and the strain
  analysis -- where one use case pulls against another, and what was cut
  or deferred because of it.
- [Implementation decisions](reference/implementation-decisions.md):
  code-level decisions recorded as the crate and adoptions are built.
