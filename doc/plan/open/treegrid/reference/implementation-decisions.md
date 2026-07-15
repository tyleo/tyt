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
