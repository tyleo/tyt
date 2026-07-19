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
