# treegrid implementation decisions

Code-level decisions made while building the
[treegrid plan](../README.md), the Rust-level companion to the
[design notes](design-notes.md). Append an entry per decision as the
crate and adoptions land; if a decision contradicts the
[rendering spec](rendering-spec.md), amend the spec in the same commit
and note it here.

## S1 chunk 1: scaffold, model, and builder (2026-07-14)

S1 lands in two chunks. This chunk is the crate skeleton, the model
types, and the builder API with the `new` / `with_json` / `with_swatch`
escape hatch; chunk 2 is the typed value constructors (`int` / `float`
/ `unorm` / `unorm8` / `bool` / `json` / `srgb8` / `srgba8`) and their
tests, which completes S1.

- **The arena is a branded `IdVec`, ids are dense indices.** `TreeGrid`
  holds `nodes: IdVec<BTreeGridNode, TreeGridNode>` plus ordered root
  ids (`Vec<U32Id<BTreeGridNode>>`, the shape of voxcore's
  `root_hierarchy_nodes`), and the push index mints ids. No
  `IdStruct` / `IdField` (owner call, 2026-07-15): those exist for
  retain / release recycling, which this append-only arena never does
  (README decision 2), and the sparse `MaybeUninit` column would cost
  `unsafe` accessors, a manual `Drop`, and the derived `Clone` /
  `Debug` / `PartialEq`. Accessors panic on an id from another grid
  (out of range) and `add_root` / `add_child` panic past `u32::MAX`
  nodes.
- **`TreeGridNode.children` is crate-private.** Exposed read-only
  through `children()`; the other node fields stay public for
  `node_mut` editing. This keeps the no-cycle, single-parent guarantee
  by construction: children attach only through `add_child`, so render
  never needs a visited set.
- **Label API: `text()` and `render()`.** `TreeGridLabel::text` is the
  raw segment (what JSON emits); `TreeGridLabel::render` is the
  text-layout form, `format!("{:?}")` for `Quoted` -- vxl `quote_name`
  semantics.
- **Enum defaults: `Rows` and `Concat`.** `TreeGridLayout::Rows` and
  `TreeGridLabelMode::Concat` carry `#[default]` so `TreeGridOptions`
  derives `Default`, matching the plan's default `rows` + `concat`
  render.
- **The spec's error variants all land at S1** even though the two
  table-shape variants become reachable only at S5, so the enum
  matches the spec's initial set from the start.

## Bold labels past heading level 6 (2026-07-15)

Owner call: a heading that would land deeper than markdown's `######`
renders as a bold label line (`**label**`) instead of clamping to
level 6, and `HeaderLevelOutOfRange` is deleted --
`TreeGridOptions::header_level` is `Option<NonZeroU8>`, so the zero
level is unrepresentable and every remaining value is valid, deep
ones included. The rendering spec (Labels, Errors), checklist (S4,
S7), and README were amended to match in the same change.
`HeaderLevelWithoutHeaders` stays: the option on a headerless render
is still an error, not a silent no-op.

## Label modes are rejected, not ignored; the text cell format is Text (2026-07-15)

Owner cleanups to the scaffold. `TreeGridOptions::label` is
`Option<TreeGridLabelMode>` (`None` means `concat` on the layouts
that consume a mode), and a mode set with `hierarchy` or the JSON
layouts, which carry labels structurally, is the new
`TreeGridError::LabelModeWithoutLabels` -- the no-silent-no-op rule
that already governs `header_level` and `table_shape`, applied to
label modes. `TreeGridLabelMode` drops its `Default`; the `Option`
expresses the default now. `TreeGridCellFormat::Value` is renamed
`Text`, naming what the cell shows. Open question, deliberately not
decided here: `width` on a layout that does not consume it is still
documented as not consumed; the same rejection treatment would need
a `WidthWithoutRows` variant and a vxl flag-mapping decision at S7.

## TreeGridOptions gains with_* builder methods (2026-07-15)

Owner request. `TreeGridOptions::default()` stays the entry point and
the fields stay public, but consuming `with_layout` / `with_label` /
`with_width` / `with_header_level` / `with_bare_roots` setters chain
off it, `TreeGridValue`-style, and wrap the `Option` fields so
callers never write `Some(...)`. Questioned and kept: `header_level`
stays `Option<NonZeroU8>` rather than a `0`-default sentinel -- zero
is not a markdown level, and set-versus-unset is what lets
`HeaderLevelWithoutHeaders` reject an explicit level on a headerless
render without rejecting the default.

## serde_json behind the `json` feature (2026-07-15)

Owner call: serde_json is optional. A non-default feature
(`json = ["dep:serde_json"]`) gates the `TreeGridValue::json` field,
`with_json`, the JSON forms of the typed constructors and
`json(Value)` itself (S1 chunk 2), and the `JsonPretty` /
`JsonCompact` layout variants, so with the feature off a JSON render
is unrequestable rather than a runtime error. JSON-rendering adopters
enable the feature explicitly, like the planned `ty-math` gate.
Accepted caveat while the crate is unpublished: a cfg-gated public
field and enum variants are not strictly additive -- feature-off
struct literals or exhaustive matches can stop compiling when another
crate in the build graph enables `json`; constructor-built values and
mapped layouts (the house pattern) are unaffected. The plan README,
checklist ground rules and S6, continue prompt, and rendering spec
were amended in the same change.

## Hierarchy values can render as child lines (2026-07-15)

Owner request, from the crate README example: inline `label: cells`
data lines put a palette's whole series on one line. New
`TreeGridOptions::value_children` (default false): when set, a data
node prints its label alone and each value prints as its own
connector line beneath, one cell per line before the node's child
nodes, so swatches keep working and long series read vertically.
Inline stays the default because the phase 3 and 4 parity trees
(`"energy-tank-1": {node: 0}`, `position: [...]`) are inline.
Per-render, not per-node: no adopter mixes both forms in one tree.
S3 implements the render. With it, `bare_roots` gains the rejection
rule the other options already follow -- `BareRootsWithoutHierarchy`
and `ValueChildrenWithoutHierarchy` on a non-hierarchy layout --
leaving `width` as the one logged open question.

## Two-layer options: flag-shaped resolve into structural layouts (2026-07-15)

Owner design. The options split into two objects with builders on
both:

- `TreeGridOptions` stays flag-shaped -- one public field per command
  flag, `Kind` enums for the flag vocabularies (`TreeGridLayoutKind`,
  `TreeGridLabelKind`, `TreeGridTableShapeKind`), the `with_*` chain
  -- and gains `resolve() -> Result<TreeGridLayout, TreeGridError>`,
  the one place invalid combinations are rejected.
- `TreeGridLayout` becomes the structural render input: each variant
  carries only the options its layout consumes
  (`TreeGridHierarchyOptions`, `TreeGridRowsOptions`,
  `TreeGridColumnsOptions`, `TreeGridTableShape` with
  `TreeGridNestedTableOptions`), heading levels ride
  `TreeGridLabelMode::Header(TreeGridHeaderOptions)` and the nested
  table payload as plain `NonZeroU8`s, and every `TreeGridError`
  combination is unrepresentable -- render, when it lands at S3,
  takes `&TreeGridLayout` and returns `String` infallibly.

Consequences: the `width` open question is settled by the structure
(`WidthWithoutRows`, the eighth and last variant -- width lives only
on the rows payload, so vxl passes `--width` through only for `rows`
at S7); the table-shape types moved forward from S5 so `resolve` and
its tests are complete now, S5 keeping only the render; tables carry
a two-variant `TreeGridTableLabelMode` (no `None` to reject); and the
strict payloads drop every set-detection `Option` -- the heading
level is a plain level wherever it exists.

## One render and resolve method per layout (2026-07-15)

Owner call, refining the two-layer design: instead of one
`render(&TreeGridLayout)` dispatching on a layout enum, each layout
gets its own pair -- `render_hierarchy(&TreeGridHierarchyOptions)`,
`render_rows(&TreeGridRowsOptions)`,
`render_columns(&TreeGridColumnsOptions)`,
`render_tables(&TreeGridTableShape)`, and `render_json_pretty()` /
`render_json_compact()` behind the `json` feature -- each returning
`String` infallibly. Calling a method is choosing the layout, so
`TreeGridLayout` and `TreeGridLayoutKind` are deleted; a command
dispatches on the clap layout enum it already has. `TreeGridOptions`
loses its `layout` field and `resolve` splits into
`resolve_hierarchy` / `resolve_rows` / `resolve_columns` /
`resolve_tables` / `resolve_json`, each rejecting every option its
render does not consume (`resolve_json` consumes none and returns
`Ok(())` when nothing is set).

## Cell policy: TreeGrid is generic over TreeGridCells (2026-07-16)

Owner design, developed over three review rounds. The grid becomes
`TreeGrid<C: TreeGridCells = TreeGridValueCells>`, storing the policy
instance and `C::Value` values, so any value type renders -- including
foreign types an adopter does not own (the orphan rule never bites,
and one value type can carry different policies). The pattern is the
stdlib's policy parameter (`HashMap<K, V, S = RandomState>`), with
`TreeGrid::new()` pinned to the default policy the way `HashMap::new`
pins `RandomState`; custom policies enter through `with_cells`. It
deliberately does not reuse the house `Dependencies` name, which
means injected IO effects, not a pure rendering strategy.

- `TreeGridCells` supplies `text` (a `Cow`, so pre-rendered values
  lend their strings), an optional `visual`, and the format a node
  with no explicit format uses for a value; `TreeGridJsonCells` adds
  `json` behind the `json` feature, so a JSON render is uncallable --
  not merely rejected -- on a policy without JSON forms. Everything
  the feature adds collects in the one gated `json` module -- the
  trait, `TreeGridOptions::resolve_json` (the `no_*` helpers widen
  to `pub(crate)` for it), and the battery's json side (later
  entries) -- so the crate's whole cfg surface is that module's two
  lines in `lib.rs`.
- `TreeGridVisual` is flag-free opaque content,
  `{ rendered: String, width: usize }`: the renderer emits the bytes
  and lays out by the declared width, so any terminal medium works
  without the crate knowing it exists.
- The two semantic swatch rules moved to their owners. Abutting is
  the `Visual` format's definition -- every cell a bare visual makes
  a continuous strip, a structural check, not a flag -- and the
  `Auto` rule is the default policy's per-value format (`Color` maps
  to `VisualText`, everything else to `Text`).
  `TreeGridCellFormat::Auto` is deleted; `TreeGridNode::format` is
  `Option<TreeGridCellFormat>`, unset meaning the policy decides per
  value -- the same Option-expresses-the-default idiom as the label
  modes. The remaining variants rename to what the cell shows --
  `Visual` and `VisualText`, the same rule as the earlier `Value` ->
  `Text` rename -- since the core deals in visuals, not swatches;
  vxl's `swatch` / `swatch-value` selector vocabulary maps into them
  at S7, the `FillMode` pattern.
- `TreeGridValue` and `TreeGridSwatch` survive as the default
  policy's vocabulary: the battery keeps the typed swatch field
  (something must remember gray versus color for the unset-format
  rule) and `TreeGridSwatch::render` owns the canonical ANSI block
  bytes.
- `TreeGrid` trades its derives for manual `Clone` / `Debug` /
  `Default` / `PartialEq` impls, since a derive cannot bound
  `C::Value`.

README decisions 1 and 5 carry dated amendments, and the spec's
model, cells, and JSON-layout sections, the checklist ground rules
and S1 / S2, the continue prompt, and the crate README were amended
in the same change. S1 chunk 2 (the typed constructors) is unchanged
in scope: the constructors build the default policy's values.

## The battery specialization lives in its own module (2026-07-17)

Owner request: the crate root holds only the generic shape -- the
grid, the policy traits, `TreeGridVisual`, the cell formats, labels,
options, and errors -- and the default-policy specialization moves
to `src/value/`: `TreeGridValue`, `TreeGridValueCells`, and
`TreeGridSwatch`, all of it feature-independent; the battery's json
side lives under the crate's `json` module (`json/value/`, next
entry). Re-exports stay flat, so the public API is unchanged; the
layout states that `TreeGridValue` is one specialization among any
number of adopter policies, not the model. The core still names the
battery in exactly two places, deliberately: the
`TreeGrid<C = TreeGridValueCells>` default parameter and the pinned
`TreeGrid::new`, the `HashMap` / `RandomState` relationship.

## TreeGridJsonValue isolates the json feature (2026-07-17)

Owner request. `TreeGridValue` drops its cfg-gated `json` field and
`with_json`, becoming `{ text, swatch }` -- no type in the crate
changes shape with the feature any more, which retires the accepted
additivity caveat from the serde_json entry: enabling `json`
elsewhere in a build graph can no longer break a feature-off
consumer, and the crate's only cfg attributes are the `json`
module's two lines in `lib.rs`. The battery's json side is
`json/value/`, inside the one gated module:

- `TreeGridJsonValue` pairs a `TreeGridValue` with the optional
  native JSON form (the divergence rationale lives on its field),
  with `new` / `with_json` / `with_swatch` builders.
- `TreeGridJsonValueCells` renders it: text, visual, and format
  delegate to `TreeGridValueCells`, and the JSON side emits the
  paired form with the `String(text)` fallback.
- The plain battery stays JSON-renderable: `TreeGridValueCells`
  implements `TreeGridJsonCells` as `String(text)` always -- exactly
  what the tag-valued tree adopters (`hierarchy show`, vmax) emit.

At S7 the divergent-json adopter builds
`TreeGrid::with_cells(TreeGridJsonValueCells)`, and chunk 2's
constructors split accordingly: text + swatch on `TreeGridValue`,
mirrored json-adding constructors on `TreeGridJsonValue`. The spec's
model, constructor, and JSON-layout sections, the plan README model
paragraph and decision 5, the checklist S1, and the crate README
were amended in the same change.

## S1 chunk 2: the typed value constructors (2026-07-19)

The spec's core constructor table lands split per the previous entry:
`int` / `float` / `unorm` / `unorm8` / `bool` / `srgb8` / `srgba8`
fill text and swatch on `TreeGridValue`, and `TreeGridJsonValue`
mirrors each, delegating to the inner constructor and pairing the
native JSON form. `json(Value)` exists only on `TreeGridJsonValue`,
since its argument type is serde_json's. This completes S1.

- **`unorm` inlines the unorm8 quantization.** The gray level is
  `(value.clamp(0.0, 1.0) * 255.0).round() as u8` -- the rule
  ty-math's `TyFloatExt::to_unorm8` implements and vxl's
  `scalar_level` applies today -- inlined because the core
  constructors must build without the `ty-math` feature. The S2
  typed-color constructors do all color math through ty-math; this
  scalar gray level is the one duplicated rule, and the two must stay
  identical.
- **The integral collapse ports vxl's `number_json` verbatim.** JSON
  is an integer when the float is integral and its magnitude is below
  `i64::MAX`, else a float. Text needs no collapse rule of its own:
  it is plain `Display` (vxl's `format_number`), which already prints
  integral floats without a fractional part.

## S2 chunk 1: text machinery and the cell matrix (2026-07-19)

S2 lands in two chunks. This chunk is the ported text machinery
(`text_width`, `tree_glyphs`, the markdown table core) and the generic
cell rendering (the resolved-format x visual matrix and the `Visual`
strip rule); chunk 2 is the `ty-math` feature and its typed-color
constructors, which completes S2. Quote formatting needed no port: it
landed at S1 as `TreeGridLabel::render`.

- **Rendered cells carry their declared width.** The crate-private
  `Cell { rendered, width }` is the unit alignment lays out by:
  `Cell::render` resolves the format (the node's, else the policy's
  per value) and applies the matrix, measuring text past ANSI escapes
  and taking a visual's declared width verbatim, so an opaque visual
  is never re-measured -- the contract `TreeGridVisual` promises. The
  markdown table core therefore takes `Cell` headers and rows rather
  than vxl's plain strings and pads by the declared widths; vxl's
  `pad_right` still ports for the label padding the `rows` /
  `columns` layouts need at S4.
- **The strip rule is structural over rendered cells.** A `Cell`
  remembers whether it is a bare visual, and `Cell::separator` joins
  a node's cells with nothing when every one is (vxl's `abuts`), one
  space otherwise -- computed from the rendered cells, matching the
  spec's phrasing, so the policy is consulted once per value.
- **The machinery lives in the private `render` module.** Like the
  battery's `src/value/`, the crate root keeps only the model:
  `src/render/` holds `cell`, `markdown_table`, `text_width`, and
  `tree_glyphs`, and its `mod.rs` flattens them with `pub(crate)
  use`, so calls carry the parent prefix -- `render::visible_width`,
  `render::markdown_table`, `crate::render::Cell` -- vxl's
  `implementation::` pattern, and the module gives the S3-S5 layout
  render implementations a home. The table function keeps vxl's
  `markdown_table` name (no stutter under the `render::` prefix);
  vxl's `row` const-generic helper is caller convenience that does
  not port.
- **The module rides one `#![allow(dead_code)]`** in `render/mod.rs`
  until the S3-S5 renders reach the machinery from the public API,
  plus targeted `#[allow(unused_imports)]` on the two re-exports
  nothing in the crate consumes yet (`markdown_table`,
  `tree_glyphs`); each allow drops with its first consumer.

## S2 chunk 2: the ty-math feature and typed-color constructors (2026-07-19)

The `ty-math` feature lands with the four float-color constructors,
completing S2. ty-math itself grows the missing pieces (owner
authorized adding `TyLinSrgb`): the three-component linear type did
not exist, and the byte quantize was defined only on the
four-component sRGB type.

- **ty-math additions, no version bump.** `TyLinSrgb<T>` mirrors
  `TySrgb` (fields, `new`, array conversions) with `to_srgb` on the
  f64 instantiation, and `TySrgb<f64>::to_u8` mirrors
  `TySrgba<f64>::to_u8`. The two sRGB transfer functions move to a
  crate-private `srgb_transfer` module (the `array_conversions`
  pattern) so both linear types share one definition. Versions bump
  at release per repo convention (`chore(release)` commits), not per
  change. The spec's "exact 3-component type names confirmed at the
  keyboard" resolved to `TyLinSrgb`; the constructor table was
  filled in, and its number-array collapse was made explicit, in the
  same change.
- **The constructors are generic over
  `T: Copy + Display + TyFloatExt + Into<f64>`.** The `TyFloatExt`
  bound pins `T` to f32 / f64, the spec's domain, so the family's
  8-bit instantiations cannot reach the float quantize (a
  `TySrgb<u8>` would clamp every nonzero channel to white; the byte
  forms have `srgb8` / `srgba8`). Text renders each component's own
  `Display`, keeping an f32's shortest form; the color math widens
  components losslessly to f64, where ty-math defines the
  conversions. The swatch drops alpha, like `srgba8`.
- **The JSON arrays collapse per component through `number_json`.**
  This is vxl's linear-color rule today (`sample_color` builds a
  `Value::Array` of `number_json` components), kept so the S17
  notation flip changes text only; `number_json` widens to
  `pub(crate)` for it. An f32 component's JSON is its exact f64
  widening -- full fidelity, the json field's stated principle -- so
  a non-dyadic f32 serializes long while its text keeps the short
  form.
- **All ty-math-gated code collects in `src/color/`**, impl-only
  files named for the types they extend (the `json/tree_grid_options`
  precedent), with the `TreeGridJsonValue` mirrors riding one nested
  `#[cfg(feature = "json")]` line in `color/mod.rs`. This amends the
  earlier "the crate's whole cfg surface is the json module's two
  lines" statement: each feature's surface is its gated module's
  lines in `lib.rs`, plus this one intersection line, placed with the
  color constructors so they read as one set.

## S3: the hierarchy render (2026-07-19)

`render_hierarchy` lands as `src/render/hierarchy.rs`, an impl-only
file on `TreeGrid` inside the render module (the
`json/tree_grid_options` precedent, and where the S2 entry said the
layout renders would live); S4 and S5 follow the same shape. The
line grammar is vxl's: `{prefix}{connector} {content}`, the child
prefix extending by the last / non-last extension glyph.

- **Bare-root sections separate with one blank line.** The spec said
  nothing; `hierarchy show` prints a blank line between its `root`
  and `unplaced` sections (the `render_group` gap rule), and the S9
  parity bar needs it from one grid rendered once. Connectored roots
  stay contiguous (vmax trees, collapsed-ancestors lists). The spec's
  `bare_roots` bullet was amended, spec fixed rather than
  implementation.
- **Every line ends `\n`; an empty grid renders empty.** The rows
  layout stated this and the hierarchy section did not; made explicit
  there in the same commit. Content ends with a label, an
  annotation, or a cell, so no hierarchy line can end in spaces and
  there is no right-trim step.
- **The node-line rule applies to bare roots too.** A bare root with
  values prints `label: cells` on its unprefixed line, and under
  `value_children` its values take connector lines at depth zero. No
  parity adopter puts values on a section root, so this is the
  uniform reading of "each root prints its label alone", not a
  divergence.
- **The S2 allows narrowed to their surviving scope.** The
  render module's blanket `#![allow(dead_code)]` dropped with its
  first public consumer; what remains is one targeted allow on the
  markdown-table module (S5) and one on `pad_right` (S4), each
  annotated with the step that retires it, plus the S2
  `unused_imports` allow on the `markdown_table` re-export.

## S4 chunk 1: the grouping walk and the rows render (2026-07-19)

S4 lands in two chunks. This chunk is the shared label-mode machinery
(the grouping walk, the concat path enumeration, and the heading
renderer) plus `render_rows` over all three label modes; chunk 2 is
`render_columns`, which completes S4.

- **The label-mode walks are crate-private `TreeGrid` methods in
  `render/group.rs`.** `data_paths` enumerates data nodes in
  pre-order with their full concat paths (the `concat` rows; `none`
  reuses the enumeration and ignores the paths), and `groups` is the
  spec's depth-first grouping walk, returning
  `Group { branch, depth, members }` entries: the root-level group
  first when any root bears values, then one group per branch that
  leads to data, a branch's own group before its child branches.
  "Leads to data" is implemented as "is a proper ancestor of a data
  node", the reading under which the worked example's `# 1` heads an
  empty group while a childless data node heads nothing. The
  recursion runs over the public accessors (`roots` / `node` /
  `children`) rather than one bottom-up arena pass, since the dense
  storage is private to `tree_grid.rs` and read-command trees are
  small. `Group` is not re-exported from `render/mod.rs`: every
  consumer reaches the walk through methods, so a re-export would
  sit unused and trip the import lint; S5 adds it with the first
  named use if one appears.
- **The heading rule lives in `render/heading.rs`.** One function
  turns level plus depth into a `#` run through markdown's level 6
  and the bold fallback past it, shared with the S5 nested tables.
- **Wrapping stays in `rows.rs`.** The port of vxl's `wrap_cells`
  operates on rendered `Cell` widths, and the block assembly inlines
  vxl's `assemble_row` without its empty-row arm: a data node always
  has at least one value, so a block always has a first segment.
  `pad_right` gains its first consumer (the label columns) and loses
  its S4 allow.
- **The workspace clippy gate is failing before this chunk.**
  `voxsmith` does not compile at the parent commit (it still calls
  `VoxPalette::binding`, renamed by the voxcore properties change),
  so verification ran `cargo clippy -p treegrid --all-targets
  --all-features` instead of the workspace sweep; treegrid is clean.

## S4 chunk 2: the columns render (2026-07-19)

`render_columns` lands as `src/render/columns.rs`, completing S4. The
unit is the column block: an optional label line, then one line per
value index, each column padded to the wider of its widest cell and
its label. `none` and `concat` render one block over every data node;
`header` renders one block per group between headings, and blocks
join like row blocks (blank line between, one trailing newline), so a
heading over an empty group stands alone exactly as in rows.

- **`join_padded` ports from vxl with the right-trim folded in.**
  Every line right-trims, so the helper trims once instead of each
  call site (vxl trimmed at the callers). Padding is `pad_right`,
  measuring visible width, so decorated swatch columns align past
  their ANSI escapes.
- **The strip rule does not reach this layout.** Each cell occupies
  its own line slot in its column, so `Cell::separator` is never
  consulted; the format matrix still applies per cell.
- **The voxsmith clippy failure from chunk 1 persists**, so
  verification again ran `cargo clippy -p treegrid --all-targets
  --all-features`; treegrid is clean.

## S4b: render extension traits and per-layout features (2026-07-20)

Owner restructure. Each layout moved into its own top-level module
named for its render method (`src/render_hierarchy/`,
`src/render_rows/`, `src/render_columns/`, `src/render_tables/`;
owner call during review, so the render family clusters beside
`render/` in the source listing while the features keep the bare
layout names) holding its extension trait on `TreeGrid`
(`TreeGridRenderHierarchy`,
`TreeGridRenderRows`, `TreeGridRenderColumns`; `TreeGridRenderTables`
joins at S5 and `TreeGridRenderJson` at S6), its options payload, and
its `resolve_*` impl, each behind a default-on cargo feature named
for the layout; `json` and `ty-math` stay non-default. `render/`
keeps only crate-private machinery shared by two or more layouts.

- **Trait impls carry the render; private helpers stay inherent.**
  Each trait has one blanket impl for `TreeGrid<C: TreeGridCells>`,
  and the render's private helpers (`render_subtree`, `row_block`,
  `column_block`) sit in an inherent `impl` block in the same file,
  so nothing private rides the trait. Tests and adopters import the
  trait alongside the payload.
- **No type changes shape with a feature.** The loose
  `TreeGridOptions` and the `Kind` enums its fields reference
  (`TreeGridLabelKind`, `TreeGridTableShapeKind`) stay ungated in the
  crate root, as do `TreeGridLabelMode` and `TreeGridHeaderOptions`,
  which the rows and columns payloads share. Whole items gate on
  features; fields never do.
- **Shared machinery gates on the union of its consumers, one gate
  per module (owner review call).** `render/` groups by feature
  predicate so each `cfg(any(...))` sits once, on a module, and leaf
  files carry no cfg at all: `render/label/` (the label-mode
  machinery: `data_paths`, `group`, `groups`, `heading`,
  `pad_right`) rides `columns` / `rows`, and the two gated `Cell`
  capabilities are impl-only files (`cell_render.rs` on `columns` /
  `hierarchy` / `rows`, `cell_separator.rs` on `hierarchy` / `rows`),
  leaving `cell.rs` (the struct and `Cell::text`) and
  `visible_width.rs` ungated under `render`'s own gate.
  Single-consumer machinery lives in its consumer instead (owner
  call, the tree-glyphs reading): `markdown_table` / `markdown_cell`
  sit in `render_tables/` under its feature, carrying the S5
  dead-code allows, and the visual-padding test builds its `Cell`
  directly so no `render_tables` test reaches the `cell_render`
  gate.
  `render/mod.rs` still flattens, so call sites keep the `render::`
  prefix, and `Cell::bare_visual` widened to `pub(crate)` for the
  sibling impl files. Every feature combination builds and documents
  warning-free (checked over the empty set, each single feature, and
  json pairings). S5 widens the unions when the tables render
  arrives (`tables` joins `cell_render` and `label`, and the
  markdown allows drop). `cell_separator`'s union is `hierarchy` /
  `rows` permanently: the columns layout never consults the strip
  rule, and tables put one cell per table cell (S15 revisits it if
  records join multi-valued cells). The `TreeGridOptions` helpers
  (`no_*`, `text_label`, `level`) keep per-method cfg unions: they
  cross the `json` feature, which no module grouping aligns with.
- **One pub item per file now covers crate-private machinery.**
  `text_width.rs` split into `visible_width.rs` / `pad_right.rs`,
  vxl's `md_cell` moved out of `markdown_table.rs` and renamed
  `markdown_cell` (owner call: the crate spells `markdown` out
  consistently, where vxl mixed the two), and the walk methods
  moved out of `group.rs` into `data_paths.rs` / `groups.rs`;
  `group.rs` keeps the `Group` struct, whose re-export gains its
  first named consumer (`groups.rs` imports `render::Group`),
  retiring the S4 no-re-export note. A method-named impl-only file
  takes its type as a prefix when the bare method name would stutter
  or collide (`cell_render.rs`, not `render/render.rs`). The tree
  glyphs folded into the hierarchy render as private consts and
  `tree_glyphs.rs` is gone.
- **Impl-only file naming.** A file adding a single method to another
  file's type is named for the method (`resolve_rows.rs`;
  `json/resolve_json.rs`, renamed from `json/tree_grid_options.rs`);
  a constructor-family file keeps the extended type's name (`color/`
  unchanged). CLAUDE.md's one-pub-per-file rule was amended to say
  both halves, plus the extension-trait rule.
- **Cross-feature doc references are plain code spans.** A core
  type's rustdoc cannot intra-doc-link a feature-gated type without
  breaking `deny(rustdoc::broken_intra_doc_links)` in feature-off
  builds, so the `Kind` enums now name `TreeGridTableLabelMode` /
  `TreeGridTableShape` in backticks without links; `cargo doc` runs
  clean across the same feature matrix.

The plan README (crate section and type roster), checklist ground
rules, the continue prompt, the rendering spec (model bullet and
per-layout render lines), and the crate README were amended in the
same change. The voxsmith workspace breakage persists, so
verification stayed scoped to `-p treegrid`.

## S5: the tables render (2026-07-20)

`render_tables` lands as `TreeGridRenderTables` in
`render_tables/tree_grid_render_tables.rs`, completing the layout the
shape types were staged for at S4b. Both shapes ride one private
`table_block` (a `#` index column, one column per node headed by its
label, one row per value index); `Nested` walks the shared `groups()`
and `Flat` reuses `data_paths()` for its concat headers.

- **Nested concat paths reconstruct from the walk order.** The arena
  has no parent links and `Group` carries only the branch id; rather
  than add a `path` field to `Group` (dead weight and a dead-code
  warning in every build where `rows` / `columns` compile the label
  machinery without `tables`), the nested render keeps a
  depth-indexed stack of cumulative paths. Groups arrive depth-first
  with parents before children and exact depths -- the walk's
  documented contract -- so truncating the stack to the group's depth
  always leaves the parent path on top. S15's relative record columns
  revisit this if they need real ancestor queries.
- **The gate unions widened as S4b planned, except `pad_right`.**
  `tables` joined `cell_render` and `label`, and the markdown
  dead-code allows dropped with their first consumer. But a
  tables-only build never pads outside `markdown_table`, so
  `pad_right` keeps the narrower `columns` / `rows` union on its
  `mod` line inside `label/` -- the `render/mod.rs` per-module
  pattern; leaf files still carry no cfg.
- **Table cells escape after the format matrix.** `markdown_cell`
  applies to the rendered bytes of every non-bare-visual cell (and to
  the label headers), with the cell's declared width growing by one
  per escaped pipe; newline flattening is width-neutral. A bare
  visual passes verbatim: the spec's rule covers cell *text*, and
  opaque visual bytes are the adopter's to keep table-safe. ANSI CSI
  sequences contain no pipes, so a decorated swatch cell is
  unaffected.
- **Blocks join like rows and columns.** Headings and tables are
  blocks joined with one blank line and a single trailing newline;
  `markdown_table`'s own trailing newline is popped so the join rule
  stays uniform. An empty grid renders as the empty string in both
  shapes.

The rendering spec's stale `(S5)` marker and the plan README's type
roster were amended in the same change. The goldens include both
worked examples from the spec (the palette forest in both label modes
plus flat, and the hierarchy-data tree cut to one full scene node in
both modes), the bold past-level-6 fallback, root-level data with no
heading, pipe escaping, and a visual cell padding by declared width.
The voxsmith workspace breakage persists, so verification stayed
scoped to `-p treegrid`.

## Layout features take the render_ prefix (2026-07-20)

Owner call, right after S5 landed: the four layout features rename to
match their modules -- `render_hierarchy`, `render_rows`,
`render_columns`, `render_tables` -- so the feature, the module, and
the render method all carry one name. This supersedes the S4b
reading that the features keep the bare layout names; `json` and
`ty-math` are untouched (they gate value forms and constructors, not
renders), and the layout names themselves are unchanged -- prose and
the adopters' `--layout` values still say `hierarchy` / `rows` /
`columns` / `tables`. The plan README, checklist ground rules, the
continue prompt, the rendering spec, and the crate README were
amended in the same change.

## S6: the JSON renders (2026-07-20)

`render_json_pretty` / `render_json_compact` land as
`TreeGridRenderJson` in `json/tree_grid_render_json.rs`, completing
phase 1. The blanket impl bounds on `TreeGridJsonCells`, so the
renders exist only for grids whose policy carries JSON forms -- the
uncallable-not-rejected rule from the cell-policy entry. The private
envelope builders ride an inherent impl under the same bound in the
same file, the S4b helper pattern.

- **Records build as `serde_json::Map`s in envelope order.** `label`,
  `annotation`, `values`, and `children` insert in the spec's key
  order and `preserve_order` keeps it. No serde derive: the
  omit-when-empty rules and the per-value policy call are the whole
  shape.
- **The renders stay infallible.** Serializing a built `Value` cannot
  fail, so the trait methods `expect` on the serde result and keep
  returning plain `String`s.
- **An empty grid renders as the empty array.** `[]` plus the
  trailing newline, not the text layouts' empty string, so a JSON
  consumer always receives valid JSON. Today vxl's JSON reports emit
  `[]` for an empty selection. The spec's JSON section gained the
  sentence and dropped its stale `(S6)` marker, and the plan README's
  type roster was amended in the same change.

The voxsmith half of the voxcore properties breakage is fixed, but
vxl still fails the workspace clippy gate. Verification therefore
stayed scoped to `-p treegrid`, across the empty set, single
features, json pairings, and `--all-features`.
