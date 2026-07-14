# treegrid rendering spec

*Part of the [treegrid plan](../README.md).* The exact contract for the
renderers. Where this spec and an adopting command's current output
disagree during a parity phase, the current output wins and this spec is
amended; after adoption, this spec is the single source of truth.

## The model

- A `TreeGrid` is an ordered forest. Each `TreeGridNode` has:
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
  - `format: TreeGridCellFormat` -- how this node's values render to cells
    (`Auto` default, `Swatch`, `SwatchValue`, `Value`).
  - `values: Vec<TreeGridValue>` -- the node's data series, possibly empty.
  - children, in insertion order.
- A `TreeGridValue` is `{ text: String, json: Option<serde_json::Value>,
  swatch: Option<TreeGridSwatch> }` where `TreeGridSwatch` is
  `Color([u8; 3])` or `Gray(u8)`. A `None` json renders as
  `String(text)` in the JSON layouts. Values are built through the
  typed constructors below, or through the escape hatch
  `new(text)` + `with_json` / `with_swatch` for pairs that genuinely
  diverge (precision-rounded text over full-fidelity numbers, the
  `{node: 0}` tags).
- A **data node** is a node with at least one value. Data nodes enumerate
  in pre-order in every layout, so all layouts agree on order. A node may
  have both values and children.

## Cells

Ported from `palette_show::render_cell` / `abuts` / the swatch functions:

| format \ swatch | `Color(rgb)` | `Gray(level)` | none |
| --- | --- | --- | --- |
| `Auto` | swatch + space + text | text | text |
| `Swatch` | swatch | gray swatch | text |
| `SwatchValue` | swatch + space + text | gray swatch + space + text | text |
| `Value` | text | text | text |

- A color swatch is `\x1b[48;2;{r};{g};{b}m  \x1b[0m` (two cells); a gray
  swatch is the same with `level` on all three channels.
- **Abutting**: a node whose format is `Swatch` and whose every value has
  a swatch joins its cells with no separator (a continuous strip);
  otherwise cells join with one space.
- All alignment measures **visible width**: characters outside ANSI CSI
  sequences (`text_width::visible_width` moves into the crate).

## Value constructors

Each fills text, JSON, and swatch from one domain-shaped argument.

Core (no features):

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
(T = f32 / f64; exact 3-component type names confirmed at the keyboard):

| constructor | text | json | swatch |
| --- | --- | --- | --- |
| `srgb(TySrgb<T>)` | `rgb(r, g, b)` | number array | quantized bytes |
| `srgba(TySrgba<T>)` | `rgba(r, g, b, a)` | number array | quantized bytes |
| `lin_rgb(..)` | `lrgb(r, g, b)` | number array | transfer + quantize |
| `lin_rgba(..)` | `lrgba(r, g, b, a)` | number array | transfer + quantize |

- The integral collapse is today's `format_number` / `number_json` rule:
  an integral f64 prints and serializes without a fractional part
  (`1.0` -> `1`), so text and JSON read alike.
- Functional notation joins `Display` components with `", "`:
  `lrgba(2, 1, 0.5, 1)`. No precision knob; policy-formatted text goes
  through `new`.
- All color math (quantization, the linear-to-sRGB transfer, CSS Color 4
  out-of-gamut handling) is ty-math's; the crate never reimplements it.

## Labels

- A segment renders as-is (`Bare`) or `{:?}`-quoted (`Quoted`).
- A **path** is the segments from a root to a node, joined with `.`:
  `0."baseColorFactor".a`.
- `TreeGridLabelMode` is consumed by `rows`, `columns`, and `tables` only:
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
- `TreeGridOptions::header_level` (`Option<u8>`): the level of the
  shallowest heading, default `1`, valid `1..=6` (else
  `TreeGridError::HeaderLevelOutOfRange`), so output embedded in a host
  markdown document sits at the right depth under its headings. Nested
  `header`-mode levels deeper than `6` clamp to `6`. Setting it on a
  render that emits no headings -- label mode `none`, `concat` with
  `rows` / `columns`, or a layout that ignores label mode -- is
  `TreeGridError::HeaderLevelWithoutHeaders`, not a silent no-op.
- Annotations never appear in `concat` or `header` labels.

## Layouts

### hierarchy

- Glyphs: connector `├` / `└` before a child, extension `│ ` / `  ` under
  a non-last / last child (today's `tree_glyphs`).
- `TreeGridOptions::bare_roots`:
  - `false` (default): roots take connectors like any child, siblings of
    one another (`tyt vmax hierarchy`, collapsed-ancestors lists).
  - `true`: each root prints its label alone on an unprefixed line, its
    children below with connectors (`hierarchy show`'s `root` /
    `unplaced` sections, `palette list`'s `palettes` line).
- A node line is `{label}{ annotation?}` when it has no values, else
  `{label}{ annotation?}: {cells}` with the node's cell separator rule.
  Values are not wrapped in this layout.
- Children render beneath in insertion order.
- Label mode and `width` are ignored.

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
  line. `width: None` never wraps. Only this layout consumes `width`.
- Under `header` mode, label padding is computed per group.

### columns

Today's `palette show --layout column`:

- One column per data node, cells padded to the column's max visible
  width (the label widens its column too, unless `none`), columns joined
  with one space, lines right-trimmed.
- Shorter columns leave trailing blanks. Under `header` mode each group
  is its own column block, blocks separated per the header rules.

### tables (series shape)

Today's `palette show --layout markdown`:

- Tables always group (see Labels), under nested headings whose text is
  the branch's full path (`concat`) or its leaf segment (`header`): one
  aligned markdown table per group. Columns: `#` (0-based value index) then one column per
  data node in the group, headed by its leaf label; one row per index
  up to the group's longest series, shorter series blank past their
  end. There is no cross-group table: vxl's old `--layout markdown`,
  one interleaved table over every collection, has no equivalent by
  design.
- `markdown_table` rules: every column pads to its widest cell, minimum
  width 3 so the dash separator stays valid markdown; cell text escapes
  pipes and flattens newlines (`md_cell`); width is visible width, so
  swatch cells align.
- `none` label mode is an error (see Labels).
- The record shape (one row per branch node, one column per single-valued
  child label -- what `info` and `palette list --layout markdown` render)
  lands in phase 6 (committed scope): a `TreeGridTableShape::Records`
  option, chosen explicitly by the caller, never inferred.

### json-pretty / json-compact

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
  `values` carries each value's `json` form, falling back to
  `String(text)` for a value built without one; `swatch`, format, label
  mode, and `width` are all ignored.
- Pretty is `serde_json::to_string_pretty`, compact `to_string`, both
  with a trailing `\n` (today's `to_json_string`).
- Records, not label-keyed objects: sibling labels may repeat and labels
  are arbitrary strings, so object keys would silently merge or collide.

## Errors

`TreeGridError`, one variant per invalid request; the initial set is
`LabelNoneWithTables`, `HeaderLevelOutOfRange`, and
`HeaderLevelWithoutHeaders`. Commands map it into their own error types
(vxl: `ErrorKind::InvalidInput`).

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

- `rows` + `concat` (format `Value`):

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
