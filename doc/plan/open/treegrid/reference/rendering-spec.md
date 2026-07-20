# treegrid rendering spec

*Part of the [treegrid plan](../README.md).* The exact contract for the
renderers. Where this spec and an adopting command's current output
disagree during a parity phase, the current output wins and this spec is
amended; after adoption, this spec is the single source of truth.

## The model

- A `TreeGrid<C: TreeGridCells = TreeGridValueCells>` is an ordered
  forest rendered through its cell policy `C`. Each `TreeGridNode` has:
  - `label: TreeGridLabel` -- one path segment. The variants name the
    rendering effect in the text layouts: `Bare(String)` prints as-is,
    for trusted identifiers (`transform`, `0`, `a`, `palettes`);
    `Quoted(String)` prints `{:?}`-escaped, for user-entered strings
    (`"baseColorFactor"`, `""`, `"has \"quotes\""`). JSON always emits
    the raw string, which is why quoting is a flag rather than baked
    into the label text by the caller.
  - `annotation: Option<String>` -- a verbatim suffix shown only by the
    `hierarchy` layout, joined to the label with one space; the caller
    supplies its own brackets (`vmax`'s `energy-tank (Group)` sets
    `annotation: "(Group)"`). Note that `vxl hierarchy show`'s tags are
    *not* annotations: its lines read `"energy-tank-1": {node: 0}` --
    label, colon, tag -- which is exactly the data-node form, so the tag
    is modeled as the node's single value (text `{node: 0}`).
  - `format: Option<TreeGridCellFormat>` -- how this node's values
    render to cells (`Visual`, `VisualText`, `Text`); unset defers to
    the policy per value.
  - `values: Vec<C::Value>` -- the node's data series, possibly empty.
  - children, in insertion order.
- The policy (`TreeGridCells`) supplies each value's cell pieces:
  `text`, an optional opaque `visual`
  (`TreeGridVisual { rendered, width }` -- bytes plus the display
  columns they occupy), and the format a node with no explicit format
  uses for that value. `TreeGridJsonCells` adds the value's native
  JSON form behind the `json` feature; the JSON renders exist only for
  grids whose policy implements it.
- A `TreeGridValue`, the default policy's value type, is
  `{ text: String, swatch: Option<TreeGridSwatch> }` where
  `TreeGridSwatch` is `Color([u8; 3])` or `Gray(u8)`; its JSON form
  is always `String(text)`. Behind the crate's non-default `json`
  feature, `TreeGridJsonValue` pairs a `TreeGridValue` with an
  optional native JSON form, rendered by `TreeGridJsonValueCells`
  (text layouts delegate to the default policy; a `None` json
  renders as `String(text)`). Values are built through the typed
  constructors below, or through the escape hatch `new(text)` +
  `with_swatch` / `with_json` for pairs that genuinely diverge
  (precision-rounded text over full-fidelity numbers, the
  `{node: 0}` tags).
- A **data node** is a node with at least one value. Data nodes enumerate
  in pre-order in every layout, so all layouts agree on order. A node may
  have both values and children.

## Cells

Ported from `palette_show::render_cell` / `abuts` / the swatch
functions. The renderer resolves a format per value -- the node's,
when set, else the policy's -- and builds the cell from the value's
`text` and optional `visual`:

| resolved format | value with a visual | without |
| --- | --- | --- |
| `Visual` | visual | text |
| `VisualText` | visual + space + text | text |
| `Text` | text | text |

- `Visual` is the strip format: a node whose every cell is a bare
  visual joins them with no separator (a continuous strip); otherwise
  cells join with one space.
- The default policy renders a value's swatch through
  `TreeGridSwatch::render` -- `\x1b[48;2;{r};{g};{b}m  \x1b[0m`, two
  cells, a gray swatch with `level` on all three channels -- and its
  per-value format is `VisualText` for a `Color` swatch and `Text`
  otherwise. The ported `Auto` row is therefore the unset node format:
  color-swatched values decorate, gray component swatches and
  swatchless values print text alone.
- All alignment measures **visible width**: a visual declares its
  width; text measures characters outside ANSI CSI sequences
  (`text_width::visible_width` moves into the crate).

## Value constructors

Each fills text and swatch from one domain-shaped argument; the
`json` column is `TreeGridJsonValue`'s mirrored constructors, which
delegate text and swatch and add the native form.

Core (`ty-math` not required):

| constructor | text | json | swatch |
| --- | --- | --- | --- |
| `new(text)` | as given | none (falls back to `String(text)`) | none |
| `int(i64)` | `Display` | integer | none |
| `float(f64)` | `Display` | integral-collapse number | none |
| `unorm(f64)` | like `float` | like `float` | `Gray(v.to_unorm8())` |
| `unorm8(u8)` | `Display` | integer | `Gray(v)` |
| `bool(bool)` | `true` / `false` | bool | none |
| `json(Value)` | compact rendering | the value | none |
| `srgb8([u8; 3])` | `#RRGGBB` | the hex string | `Color(rgb)` |
| `srgba8([u8; 4])` | `#RRGGBBAA` | the hex string | `Color(rgb)` |

Behind the `ty-math` feature, over the component-generic color family
(T = f32 / f64):

| constructor | text | json | swatch |
| --- | --- | --- | --- |
| `srgb(TySrgb<T>)` | `rgb(r, g, b)` | number array | quantized bytes |
| `srgba(TySrgba<T>)` | `rgba(r, g, b, a)` | number array | quantized bytes |
| `lin_rgb(TyLinSrgb<T>)` | `lrgb(r, g, b)` | number array | transfer + quantize |
| `lin_rgba(TyLinSrgba<T>)` | `lrgba(r, g, b, a)` | number array | transfer + quantize |

- The mirrored json constructors, the `json(Value)` constructor
  itself, and `with_json` live on `TreeGridJsonValue` behind the
  non-default `json` feature; `TreeGridValue`'s own constructors
  fill text and swatch alone.
- The integral collapse is today's `format_number` / `number_json` rule:
  an integral f64 prints and serializes without a fractional part
  (`1.0` -> `1`), so text and JSON read alike.
- Functional notation joins `Display` components with `", "`:
  `lrgba(2, 1, 0.5, 1)`. No precision knob; policy-formatted text goes
  through `new`. The number arrays collapse per component the same
  way: `[2, 1, 0.5, 1]`.
- All color math (quantization, the linear-to-sRGB transfer, CSS Color 4
  out-of-gamut handling) is ty-math's; the crate never reimplements it.

## Labels

- A segment renders as-is (`Bare`) or `{:?}`-quoted (`Quoted`).
- A **path** is the segments from a root to a node, joined with `.`:
  `0."baseColorFactor".a`.
- `TreeGridLabelMode` lives on the `rows` and `columns` payloads
  (`tables` carries its own two-variant `TreeGridTableLabelMode`);
  the `resolve_*` methods map the loose `TreeGridOptions::label` kind
  into them, unset meaning `concat`. A label mode with the
  `hierarchy` or JSON renders, which carry labels structurally, is
  `TreeGridError::LabelModeWithoutLabels`:
  - `none`: no labels anywhere. Under `tables` this is
    `TreeGridError::LabelNoneWithTables`.
  - `concat` (default): each data node is labeled by its full path. On
    `rows` and `columns` the label sits inline on the row or column
    head, with no headings. On `tables`, which cannot spend a column
    header on a long path, the headings follow the same nested walk as
    `header` -- same positions, same increasing levels -- but each
    heading's text is that branch's full concat path instead of its
    leaf segment, so every heading is self-contained.
  - `header`: the ancestor chain becomes nested markdown headings:
    every branch segment that leads to data prints once, depth-first,
    at level `header_level + depth` (a root segment at `header_level`),
    and each branch's group block sits directly under its heading,
    before any deeper subsection headings. Group content is labeled by
    leaf segment alone.
- **Grouping and order** (concat `tables`, and every `header` render): a
  group is one branch's direct data children, in insertion order.
  Groups emit in a depth-first walk -- a branch's own group first, then
  its child branches recursively -- so a group never lands inside a
  deeper subsection's heading. Data nodes that are themselves roots
  (empty parent path) print first, with no heading. A blank line
  separates a heading from its content and content from the next
  heading. A node with both values and children is a column in its
  parent's group *and* a heading over its own.
- The heading level: `header` labels carry it
  (`TreeGridLabelMode::Header(TreeGridHeaderOptions)`), and nested
  tables carry theirs beside the table label. The shallowest heading
  prints at that level, default `1`, so output embedded in a host
  markdown document sits at the right depth under its headings. A
  heading that lands deeper than `6` -- markdown's deepest level --
  renders as a bold label on its own line (`**label**`) instead of a
  `#` run, so depth never errors and never emits invalid `#######`
  markdown; the zero level is unrepresentable (`NonZeroU8`). The
  `resolve_*` methods fold the loose `TreeGridOptions::header_level`
  into those payloads; set on a render that emits no headings --
  label mode `none`, `concat` with `rows` / `columns`, flat tables,
  or a render that takes no label mode -- it is
  `TreeGridError::HeaderLevelWithoutHeaders`, not a silent no-op.
- Annotations never appear in `concat` or `header` labels.

## Layouts

### hierarchy

- Rendered by `render_hierarchy(&TreeGridHierarchyOptions)`.
- Glyphs: connector `├` / `└` before a child, extension `│ ` / `  ` under
  a non-last / last child (today's `tree_glyphs`).
- `TreeGridHierarchyOptions::bare_roots` (the other `resolve_*`
  methods reject it as `TreeGridError::BareRootsWithoutHierarchy`):
  - `false` (default): roots take connectors like any child, siblings of
    one another (`tyt vmax hierarchy`, collapsed-ancestors lists).
  - `true`: each root prints its label alone on an unprefixed line, its
    children below with connectors (`hierarchy show`'s `root` /
    `unplaced` sections, `palette list`'s `palettes` line). Successive
    root sections separate with one blank line, today's gap between
    `root` and `unplaced`.
- A node line is `{label}{ annotation?}` when it has no values, else
  `{label}{ annotation?}: {cells}` with the node's cell separator rule.
  Values are not wrapped in this layout.
- `TreeGridHierarchyOptions::value_children`: when true, a data node
  prints `{label}{ annotation?}` alone and each value prints as its
  own child line beneath, before the node's child nodes -- one cell
  per line, rendered per the node's format, taking a connector like a
  child. Default false, the inline form above; the other `resolve_*`
  methods reject it as `TreeGridError::ValueChildrenWithoutHierarchy`.
- Children render beneath in insertion order.
- Every line ends with `\n`; an empty grid renders as an empty string.
- `resolve_hierarchy` rejects a label mode (`LabelModeWithoutLabels`)
  and a width (`WidthWithoutRows`).

Observed shapes this layout must reproduce (from
`vxl hierarchy show src/vmax/energy-reactor.vmax --show-transforms
--show-layers --show-voxel-counts` in `submodules/tyt-assets`):

```text
root                                      <- bare root (bare_roots: true)
├ "energy-tank-1": {node: 0}              <- Quoted + tag value
│ ├ transform                             <- Bare branch, no values
│ │ ├ position: [12.5, 0.5, 10.0]         <- Bare + one pre-formatted value
│ │ └ ...
│ └ "energy-tank-1": {object: 0, instance: 0}
│   ├ voxel-count: 1604
│   └ layers
│     └ 0: {materials: 10}                <- Bare("0") + tag value
```

and from vmax `hierarchy` (connectored roots, annotation form):

```text
├ energy-tank (Group)
│ └ energy-tank-1 (Object)
```

### rows

Today's `palette show --layout row`:

- Rendered by `render_rows(&TreeGridRowsOptions)`.
- One row per data node: `{label} {cells}`.
- Labels pad to the longest label so every row's first cell aligns; cells
  themselves are never padded. `--label none` drops the label column and
  the indent entirely.
- Rows separate with one blank line. Lines right-trim. Output ends with
  one `\n`; an empty grid renders as an empty string.
- `width: Some(budget)` wraps a row's cells onto continuation lines
  indented to the first cell's column: cells pack greedily by visible
  width (`wrap_cells` semantics), a cell wider than the remaining budget
  takes a line of its own, and at least one cell is always placed per
  line. `width: None` never wraps. Only this render consumes `width`
  (`TreeGridRowsOptions::width`); the other `resolve_*` methods
  reject it (`TreeGridError::WidthWithoutRows`).
- Under `header` mode, label padding is computed per group.

### columns

Today's `palette show --layout column`:

- Rendered by `render_columns(&TreeGridColumnsOptions)`.
- One column per data node, cells padded to the column's max visible
  width (the label widens its column too, unless `none`), columns joined
  with one space, lines right-trimmed.
- Shorter columns leave trailing blanks. Under `header` mode each group
  is its own column block, blocks separated per the header rules.

### tables

Rendered by `render_tables(&TreeGridTableShape)`.
`TreeGridTableShape` picks the shape:
`Nested(TreeGridNestedTableOptions)`, carrying the heading label mode
and level, or `Flat`. `resolve_tables` maps the loose
`TreeGridOptions::table_shape` kind into it, unset meaning `Nested`;
the other `resolve_*` methods reject a set shape as
`TreeGridError::TableShapeWithoutTables`, not a silent no-op.

- `Nested`: tables group (see Labels), under nested headings whose text
  is the branch's full path (`concat`) or its leaf segment (`header`):
  one aligned markdown table per group. Columns: `#` (0-based value
  index) then one column per data node in the group, headed by its leaf
  label; one row per index up to the group's longest series, shorter
  series blank past their end.
- `Flat`: one table over every data node, no headings, columns headed
  by full concat paths -- vxl's old `--layout markdown`, kept as an
  explicit shape because it is the comparison view: two palettes'
  colors line up side by side in one table. Requires `concat`;
  `header` is `TreeGridError::HeaderLabelWithFlatTables`.
- `Records` (lands in phase 6, committed scope): the transpose for
  entity-per-row reports, what `info` and `palette list --layout
  markdown` render today. Rows are one branch's children; columns are
  the union of their descendant data-node paths flattened *relative to
  the row*, so a node's `transform.position` is a prefix-free column
  and the prefix lives in the row label, plus a column for a row's own
  value. Chosen explicitly, never inferred. Column naming, multi-valued
  cells, and heterogeneous-children sparsity are settled at S15 against
  the real adopters.
- `markdown_table` rules: every column pads to its widest cell, minimum
  width 3 so the dash separator stays valid markdown; cell text escapes
  pipes and flattens newlines (`md_cell`); width is visible width, so
  swatch cells align.
- `none` label mode is an error (see Labels).

### json-pretty / json-compact

- Behind the non-default `json` feature; without it these renders do
  not exist. Rendered by `render_json_pretty()` /
  `render_json_compact()` after a `resolve_json()` check that no
  option is set.
- The envelope: a JSON array of root records, where a record is

  ```json
  {
    "label": "baseColorFactor",
    "annotation": "Node",
    "values": [ ... ],
    "children": [ ... ]
  }
  ```

  with `annotation`, `values`, and `children` omitted when absent/empty.
  `label` is the raw string, unquoted-extra, whether `Bare` or `Quoted`.
  `values` carries each value's policy JSON form
  (`TreeGridJsonCells`): `TreeGridJsonValueCells` emits the paired
  native form, falling back to `String(text)`, and the default
  policy emits `String(text)` always; visuals and format are not
  consumed, and `resolve_json` rejects every set option.
- Pretty is `serde_json::to_string_pretty`, compact `to_string`, both
  with a trailing `\n` (today's `to_json_string`).
- Records, not label-keyed objects: sibling labels may repeat and labels
  are arbitrary strings, so object keys would silently merge or collide.

## Errors

`TreeGridError`, one variant per invalid option combination, returned
by the `TreeGridOptions` `resolve_*` methods; each render method
takes a payload in which every such combination is unrepresentable,
and cannot fail. The set is `LabelNoneWithTables`,
`LabelModeWithoutLabels`, `HeaderLevelWithoutHeaders`,
`HeaderLabelWithFlatTables`, `TableShapeWithoutTables`,
`BareRootsWithoutHierarchy`, `ValueChildrenWithoutHierarchy`, and
`WidthWithoutRows`. Commands map it into their own error types (vxl:
`ErrorKind::InvalidInput`).

## Worked example

One palette with two attributes, one of them component-extracted:

```text
TreeGrid
├ Bare("0")
│ ├ Quoted("baseColorFactor")           values: #FF0000FF, #00FF0080 (Color swatches)
│ └ Quoted("metallicFactor")            values: 1, 0.2 (Gray swatches)
└ Bare("1")
  └ Quoted("baseColorFactor")
    └ Bare("a")                       values: 255 (Gray swatch)
```

- `rows` + `concat` (format `Text`):

  ```text
  0."baseColorFactor"   #FF0000FF #00FF0080

  0."metallicFactor"    1 0.2

  1."baseColorFactor".a 255
  ```

- `rows` + `header` (default `header_level` 1):

  ```text
  # 0

  "baseColorFactor" #FF0000FF #00FF0080

  "metallicFactor"  1 0.2

  # 1

  ## "baseColorFactor"

  a 255
  ```

- `tables` + `concat` -- nested, full-path heading text; `# 1` prints
  bare because the structure needs it even though its group is empty:

  ```text
  # 0

  | #   | "baseColorFactor" | "metallicFactor" |
  | --- | ----------------- | ---------------- |
  | 0   | #FF0000FF         | 1                |
  | 1   | #00FF0080         | 0.2              |

  # 1

  ## 1."baseColorFactor"

  | #   | a   |
  | --- | --- |
  | 0   | 255 |
  ```

- `tables` + `header`: identical structure; the deep heading reads
  `## "baseColorFactor"` instead of `## 1."baseColorFactor"`.

- `tables` + `concat` + `table_shape: Flat` -- the comparison view, one
  table over everything with concat column headers:

  ```text
  | #   | 0."baseColorFactor" | 0."metallicFactor" | 1."baseColorFactor".a |
  | --- | ------------------- | ------------------ | --------------------- |
  | 0   | #FF0000FF           | 1                  | 255                   |
  | 1   | #00FF0080           | 0.2                |                       |
  ```

- `hierarchy` (`bare_roots: false`):

  ```text
  ├ 0
  │ ├ "baseColorFactor": #FF0000FF #00FF0080
  │ └ "metallicFactor": 1 0.2
  └ 1
    └ "baseColorFactor"
      └ a: 255
  ```

- `json-compact`:

  ```json
  [{"label":"0","children":[{"label":"baseColorFactor","values":["#FF0000FF","#00FF0080"]},{"label":"metallicFactor","values":[1,0.2]}]},{"label":"1","children":[{"label":"baseColorFactor","children":[{"label":"a","values":[255]}]}]}]
  ```

## Worked example: hierarchy data

The dry run that shaped the grouped-tables design (2026-07-13):
`vxl hierarchy show submodules/tyt-assets/src/vmax/energy-reactor.vmax
--show-transforms`, whose tree is the `hierarchy`-layout output under
"Observed shapes" above -- a `root` section over four scene nodes, each
carrying a tag value, a `transform` branch (`position` / `rotation` /
`scale`, one pre-formatted value each), and a tag-valued object child.
Every data node is single-valued, so every table has one data row.

`tables` + `header` (default `header_level` 1; the first node shown, the
other three repeat the same shape):

```text
# root

| #   | "energy-tank-1" | "energy-tank-2" | "energy-reactor" | "energy-tank" |
| --- | --------------- | --------------- | ---------------- | ------------- |
| 0   | {node: 0}       | {node: 1}       | {node: 2}        | {node: 3}     |

## "energy-tank-1"

| #   | "energy-tank-1"          |
| --- | ------------------------ |
| 0   | {object: 0, instance: 0} |

### transform

| #   | position             | rotation           | scale              |
| --- | -------------------- | ------------------ | ------------------ |
| 0   | [12.50, 0.50, 10.00] | [0.00, 0.00, 0.00] | [1.00, 1.00, 1.00] |

## "energy-tank-2"
...
```

The `root` table holds the four node tags because a node with both
values and children is a column in its parent's group and a heading
over its own; the object table precedes `### transform` because a
branch's own group emits before its child branches (`transform` is a
branch, the object a direct data child).

`tables` + `concat` is the same walk at the same levels; only the
heading text changes, each carrying its full path:

```text
# root

| (the same four-column tag table) |

## root."energy-tank-1"

| (the object-tag table) |

### root."energy-tank-1".transform

| (the transform table) |

## root."energy-tank-2"
...
```

The remaining layouts over the same tree. `rows` + `concat` emits no
headings: one row per data node, labels padded to the longest, and the
best grep target of the layouts:

```text
root."energy-tank-1"                     {node: 0}

root."energy-tank-1".transform.position  [12.50, 0.50, 10.00]

root."energy-tank-1".transform.rotation  [0.00, 0.00, 0.00]

root."energy-tank-1".transform.scale     [1.00, 1.00, 1.00]

root."energy-tank-1"."energy-tank-1"     {object: 0, instance: 0}

root."energy-tank-2"                     {node: 1}
...
```

`columns` + `concat` is the transpose: twenty single-valued columns
under full-path headers, one data row -- columns earn their keep on
long series like a palette's materials, not here:

```text
root."energy-tank-1" root."energy-tank-1".transform.position root."energy-tank-1".transform.rotation ...
{node: 0}            [12.50, 0.50, 10.00]                    [0.00, 0.00, 0.00]                      ...
```

`tables` + `Records` (phase 6) is the shape this data actually wants:
one row per node, descendant paths flattened relative to the row, the
prefix living in the row label (column names illustrative until S15):

```text
# root

| node             | value     | transform.position    | transform.rotation | transform.scale    |
| ---------------- | --------- | --------------------- | ------------------ | ------------------ |
| "energy-tank-1"  | {node: 0} | [12.50, 0.50, 10.00]  | [0.00, 0.00, 0.00] | [1.00, 1.00, 1.00] |
| "energy-tank-2"  | {node: 1} | [-11.50, 0.50, 10.00] | [0.00, 0.00, 0.00] | [1.00, 1.00, 1.00] |
| "energy-reactor" | {node: 2} | [0.50, 14.50, 23.00]  | [0.00, 0.00, 0.00] | [1.00, 1.00, 1.00] |
| "energy-tank"    | {node: 3} | [0.50, 0.50, 10.00]   | [0.00, 0.00, 0.00] | [1.00, 1.00, 1.00] |
```

The differently-named object children would add one sparse column per
name; a command building for records skips or normalizes them. `Flat`
on this tree is the degenerate 21-column, one-row table that motivated
grouping in the first place -- available, not advisable.

The `hierarchy` and JSON layouts ignore label modes: the `hierarchy`
render of this tree is the "Observed shapes" listing above, and the
envelope carries each label structurally.

What this example pins down: single-valued hierarchy data degenerates
to one-row series tables (the `#` column is all zeros), which is why
the record shape (phase 6) exists and why `hierarchy show` exposes only
`hierarchy` + JSON in v1; and the `root` section label leads every
concat heading, so a command exposing tabular layouts may prefer to
build a flatter forest for them -- the command owns the tree it
populates.
