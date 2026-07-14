# treegrid design notes

*Part of the [treegrid plan](../README.md).* Rationale for the
non-obvious choices, and the strain analysis: where one consumer's needs
pull against another's, and what was cut or deferred because of it.

## Strain analysis

The question this crate has to survive: does supporting every capability
(six layouts, three label modes, four hierarchy commands, swatches,
markers, JSON) warp the design for any one consumer? Each strain found,
and its resolution:

### 1. Two table orientations exist, and only one ships in v1

`palette show --layout markdown` is **series-shaped**: one column per
collection, one row per value index, a `#` column. `info` and
`palette list --layout markdown` are **record-shaped**: one row per
entity, one column per field -- the transpose. Both are legitimate
readings of the same tree (series: columns are data nodes; records: rows
are branch nodes, columns their single-valued children's labels).

Forcing one shape onto the other's consumers would produce absurd output
(a one-row-per-palette listing as a 40-column-wide table). Auto-detecting
the shape from value counts is worse: a palette with a single material
would silently flip a series table into a record table. Resolution: v1
ships series only (the `palette show` shape); the record shape is an
explicit `TreeGridTableShape::Records` render option, landing with the
`info` / `validate` / `palette list` adoption in phase 6. That phase is
committed scope, not optional -- the owner accepted series-first on the
condition that the record shape lands before the plan closes
(2026-07-12). Until then those commands keep their bespoke
`markdown_table` rendering, which is duplication we accept knowingly.

### 1b. The hierarchy dry-run reshaped the label modes (2026-07-13)

Dry-running `tables` against `vxl hierarchy show --show-transforms` on
the energy-reactor asset (now a worked example in the
[rendering spec](rendering-spec.md)) broke the first design twice: the
single cross-parent `concat` table degenerated to 21 columns and one
row, and flat `##` group headings carrying long concatenated parents
read worse than real document structure. Owner revisions: `tables`
always groups by parent path in every label mode; both modes nest
headings per segment (level `header_level + depth`, clamped at 6),
reversing the earlier "depth lives in the concatenated path, never in
nested heading levels" rule -- `concat` and `header` differ only in
heading text, the branch's full path versus its leaf segment (flat
same-level concat headings were considered and dropped: a group
following an unrelated section would visually nest under it); and
`header_level` defaults to `1` so standalone output reads as its own
document. Two costs, accepted eyes-open: the old
interleaved `markdown` table -- palette collections compared
side-by-side across parents in one table -- has no equivalent (if
wanted later it is a new explicit option, not a label mode), and
`--layout markdown -> --layout tables` in the S7 migration is a
redesign, not a rename.

### 2. Generic JSON versus today's bespoke records

`palette show` emits `[{"palette": 0, "attribute": "baseColorFactor",
"values": [...]}]` -- domain keys a generic crate cannot know.
`palette list` emits `[{"index", "attributes", "materials", "used_by"}]`.
A shared renderer must pick a shape, and label-keyed nesting
(`{"0": {"baseColorFactor": [...]}}`) is disqualified on correctness:
sibling labels repeat (two objects both named `door`), and labels are
arbitrary user strings that would collide with any structural key.

Resolution: the record envelope (`{"label", "annotation"?, "values"?,
"children"?}`). This *is* the "one shared JSON envelope across the read
commands" the vxl-commands plan deferred
([palette/show.md](../../vxl-commands/reference/palette/show.md),
Deferred item 3) -- settled here, in the library, once. Cost: `palette
show`'s JSON output breaks (a conscious `feat(vxl)!`), and `palette
list` / `info` / `validate` JSON stays bespoke until the deferred phase.

### 3. Composite leaf labels

`0."baseColorFactor".a` looks like a quoted middle segment with suffixes,
which breaks "one label per node" if the leaf owns `baseColorFactor.a`
as one string. Resolution: the component is its own `Bare("a")` child
node. Concat labels, header grouping, the hierarchy tree, and JSON
nesting then all fall out of the same structure with no special casing.
The general rule: **if a label has internal structure, model the
structure as nodes.**

### 4. Two suffix styles: tag values and annotations

The tree walks decorate names two ways, checked against real output in
`submodules/tyt-assets`. `vxl hierarchy show` prints
`"energy-tank-1": {node: 0}` and `0: {materials: 10}` -- label, colon,
brace tag -- which is *exactly* the data-node form, so the tag is simply
the node's value (text `{node: 0}`, JSON `{"node": 0}`) and needs no
feature. vmax prints `energy-tank (Group)` -- a space-joined suffix with
no colon -- which does not fit the value form; that is what the
`annotation` field is for, rendered verbatim (caller supplies the
parens), `hierarchy` layout only, optional in JSON. Baking either style
into the label would poison concat paths (`0."door (Object)".transform`)
and table headers. Markers (`ancestors`, `descendants`, `layers: []`)
need no feature at all: they are ordinary `Bare` nodes the command
inserts.

### 4b. Quoting parity in the not-yet-quoted commands

Commit 82e803a quoted user-entered names in `hierarchy show` and
`palette show`, but `palette list --layout hierarchy` still prints
attribute and object names bare (`├ baseColorFactor`), and vmax / fbx
never quote. `Quoted` labels always quote, so a parity adoption must use
`Bare` for those names, or accept a one-line quoting normalization in
the same commit. Recommendation: normalize to `Quoted` (it finishes what
82e803a started), but decide per adoption at the keyboard and log it in
implementation decisions -- it is the one place "byte-identical parity"
and "consistent quoting" cannot both hold.

### 5. Two top-level tree styles

vmax and the collapsed-ancestors listings give roots connectors
(`├ door`); `hierarchy show`'s sections (`root`, `unplaced`) and
`palette list`'s `palettes` line are bare header lines with connectored
children. Resolution: `TreeGridOptions::bare_roots`. With it, `palette
list`'s current output is exactly a bare `palettes` root, and `hierarchy
show`'s sections are two bare roots -- no caller-side preamble hacks.

### 6. Policy formatting stays upstream; canonical rendering is constructors

Anything with a knob stays in the commands: precision flags
(`--show-transforms local rad 4`, `fmt3`), space/unit conversion, pool
classification. Values shaped by those arrive as finished text through
the `new(text)` escape hatch. What the typed constructors own is
knob-free canonical rendering: `Display` numbers with the integral
collapse (`format_number` / `number_json` move in), `#RRGGBB(AA)` hex
for 8-bit sRGB, and the `rgb(...)` / `rgba(...)` / `lrgb(...)` /
`lrgba(...)` functional notation for float-component colors. The
notation is a cross-command convention (owner's rule: any float-valued
color renders functionally, 2026-07-12), and conventions belong in the
shared crate. The swatch is the one rich display concept the crate owns
end to end, because alignment must measure past its escape codes and
every text layout renders it.

The split is also why a value stores both `text` and an optional `json`
instead of deriving one from the other: display text and JSON canon
legitimately diverge today (Rust `Display` prints
`0.0000000018626451422920631` where serde's ryu prints
`1.8626451422920631e-9`; `fmt3` text is precision-rounded while JSON
should keep full-fidelity numbers; the `{node: 0}` tags are not JSON at
all). Deriving text from JSON would force a canonical rendering of
arbitrary JSON into the library. A value built with only `text` falls
back to `String(text)` in the JSON layouts, so tree-only adopters skip
the field entirely.

### 6b. Typed colors ride a `ty-math` feature, not DI

The float-color constructors need real color math: quantization for
sRGB floats, and the linear-to-sRGB transfer for a `lin_*` swatch.
Trait-style DI (the repo's `Dependencies` pattern) exists to seam side
effects for testing; a pure function gains nothing from injection and
would noise every call site. Reimplementing the transfer in treegrid
is vetoed outright: ty-math's `to_srgba` does CSS Color 4 odd-extension
for out-of-gamut components (settled in the ty-color-model plan), and a
second copy would drift. So the constructors sit behind an optional
`ty-math` feature -- free for vxl, which already depends on ty-math --
and take the component-generic color family (`TySrgba<T>` /
`TyLinSrgba<T>`, T = f32 / f64), so f32 support falls out of the
generics (`TySrgba<f32>` already has a real producer: the fbx vertex
colors). The featureless escape hatch stays available: inject the
*result* (`new(text).with_swatch(...)`) instead of the function.

Linear is worth carrying, not speculative: voxj value pools have
`LinearRgb` / `LinearRgba` kinds, `palette show` renders them today
(the pinned HDR `emissiveFactor` test at `2.0`, a component no hex can
hold), and glTF emissive factors in the mesh pipeline are linear.

### 7. tyt-fbx renders inside Blender

The strongest strain. `tyt fbx hierarchy` builds its tree in
`FBX_HIERARCHY_PY` (Blender Python) and streams it to stdout; Rust only
packs flags. Adoption means the process boundary carries **data, not
text**: extend the existing `FBX_HIERARCHY_JSON_PY` (already used for
`--select` matching) to emit the hierarchy with per-object payloads --
type, and the transform / bounds / extents values computed *in Blender*
for the requested space/unit/precision, since that math needs Blender's
scene evaluation -- then build and render the `TreeGrid` in Rust. Wins:
one renderer, output testable in Rust without Blender, and the JSON
layouts come free. Cost: the largest adoption diff, and output buffers
through JSON instead of streaming. It is phase 5 and fully severable; the
crate's design takes nothing from deferring it.

### 8. Commands support different layout subsets

`hierarchy show` as `rows` or `tables` is semantically defined
(pre-order data nodes: every `position`, `rotation`, ... as a row) but of
dubious value, and the vxl-commands plan currently promises "`hierarchy
show` prints only its tree." Resolution: the library defines all layouts
for all trees; each command exposes only the subset it wants via its own
clap `ValueEnum` (the repo's `FillMode` / `MaterialMode` mapping
pattern). `hierarchy show` starts with `hierarchy` + `json-pretty` +
`json-compact`. No shared clap feature: per-command enums are exactly how
subsets stay honest, and it matches the existing
`PaletteListLayout`-versus-`ReportLayout` split.

### 9. Width wrapping stays a rows-only concern

Today only the `row` layouts wrap, and terminal detection is a libc
ioctl in vxl. Pulling detection into the crate would drag in libc and a
platform surface for one adopter. Resolution: the crate takes
`width: Option<usize>`; vxl keeps `Width` parsing and
`terminal_columns()`. If vmax/fbx ever gain `rows`, move the helper then.

### 10. Label mode interactions must be total

Every (layout, label mode) pair needs defined behavior or a defined
error, or command authors will guess: `none` + `tables` errors;
`hierarchy` and the JSON layouts ignore label mode; `header` with a
root-level data node emits a headerless leading group; `header_level` set
on a render that emits no headers is an error, not a silent no-op. All
pinned in the
[rendering spec](rendering-spec.md).

## What this crate deliberately is not

- Not a selection engine: `--select` / `--select-index` / pathspec stay
  upstream, per the owner's direction that selection exists outside the
  system.
- Not a schema/reflection system: no derive macros, no serde-driven tree
  building. Commands write explicit builders; the four adopters each
  have one already in the shape of their current render walk.
- Not a TUI or styling framework: the only ANSI it knows is the swatch
  pair and how to measure past CSI sequences.
- Not a DAG renderer: single-parent forest by construction; instancing
  expands upstream.

## Naming

`treegrid` (owner's choice, 2026-07-12) over the draft's `treegrid`:
the owner wanted a generic standalone name, and "tree grid" is the term
of art for exactly this widget concept -- a hierarchy whose nodes carry
data columns (the ARIA role is `treegrid`; JavaFX/SWT call it a tree
table, Blender an outliner). It matches the voxcore / voxsmith /
pathspec single-word style. Checked unclaimed on crates.io 2026-07-12:
`treegrid`, `treetable`, `outliner`, `datatree`, `showtree`,
`treeport`, `treegrid`, `ty-view`. Earlier draft names rejected:
`ty-view` ("view" already means a per-node data lens in the files being
migrated: `HierarchyViews`, `TransformView`), then `treegrid` (too
CLI-specific for the owner's taste). Type names carry the `TreeGrid`
prefix, voxcore-style -- which as a bonus no longer neighbors vxl's
existing `ReportLayout` clap enum during migration; that enum keeps its
name until S16 retires it.

Label variants are `Bare` / `Quoted` (owner's choice, 2026-07-12),
naming the rendering effect rather than the provenance; an earlier
draft's `Word` / `Name` made readers learn the quoting rule indirectly.
"Bare" over "Unquoted" for brevity and because it is the word the
existing doc comments already use ("prints bare"). Caller-side
pre-quoting was considered and rejected: the JSON layouts need the raw
string from the same populated grid, and `{:?}` escaping (embedded
quotes, control characters, the empty string) should live in one place.

## Prior art consulted in-repo

- `palette_show.rs`: the cell/format matrix, abutting, row wrapping,
  visible-width alignment -- ported nearly verbatim into the spec.
- `hierarchy_show.rs` / vmax `hierarchy.rs` / `palette_list.rs`: the
  three tree walks whose union defines the `hierarchy` layout
  (annotations, markers, sections, `label: value` leaves).
- `markdown_table.rs`, `tree_glyphs.rs`, `text_width.rs`,
  `quote_name.rs`, `to_json_string.rs`: move into the crate (as private
  modules) rather than being re-invented; vxl keeps local copies only for
  the not-yet-adopted `info` / `validate` renderers until the deferred
  phase.
