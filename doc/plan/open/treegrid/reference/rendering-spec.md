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
  - `concat` (default): each data node is labeled by its full path.
  - `header`: data nodes group by **parent path** (the path minus the leaf
    segment), in order of each group's first data node. Each group with a
    non-empty parent path prints `## {parent path}`, a blank line, then
    the group's content labeled by leaf segment alone; a blank line
    separates a group's content from the next header. The root-level
    group (empty parent path) prints its content with no header, first.
    Every header in one render is the same level:
    `TreeGridOptions::header_level` (`Option<u8>`) `#`s, default `2`,
    valid `1..=6` (else `TreeGridError::HeaderLevelOutOfRange`), so
    output embedded in a host markdown document sits at the right depth
    under its headings. Setting it on a render that emits no headers --
    a label mode other than `header`, or a layout that ignores label
    mode -- is `TreeGridError::HeaderLevelWithoutHeaders`, not a silent
    no-op. Depth within the grid is expressed in the concatenated parent
    path, never by nesting header levels; the option moves every header
    together.
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

- One aligned markdown table per group (`concat`: one group of all data
  nodes; `header`: one per parent path). Columns: `#` (0-based value
  index) then one column per data node, headed by its label per the label
  mode; one row per index up to the longest series, shorter series blank
  past their end.
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

- `rows` + `header`:

  ```text
  ## 0

  "baseColorFactor" #FF0000FF #00FF0080

  "metallicFactor"  1 0.2

  ## 1."baseColorFactor"

  a 255
  ```

- `tables` + `concat`:

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
