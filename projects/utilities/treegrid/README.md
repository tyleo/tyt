# treegrid

Hierarchical-data rendering: populate a forest of labeled, data-bearing
nodes, then render it under a chosen layout and label mode.

```rust
let mut grid = TreeGrid::new();

let palette = grid.add_root(TreeGridLabel::bare("0"));

let base = grid.add_child(palette, TreeGridLabel::quoted("baseColorFactor"));
grid.push_value(base, TreeGridValue::new("#FF0000FF"));
grid.push_value(base, TreeGridValue::new("#00FF0080"));

let metallic = grid.add_child(palette, TreeGridLabel::quoted("metallicFactor"));
grid.push_value(metallic, TreeGridValue::new("1"));
grid.push_value(metallic, TreeGridValue::new("0.2"));

let rows = grid.render_rows(&TreeGridRowsOptions::default());
```

The default `rows` layout renders each data node as one labeled row:

```text
0."baseColorFactor" #FF0000FF #00FF0080

0."metallicFactor"  1 0.2
```

The `hierarchy` layout renders the same grid as a box-glyph tree;
`value_children` gives each value its own line:

```rust
let tree = grid.render_hierarchy(
    &TreeGridHierarchyOptions::default().with_value_children(true),
);
```

```text
└ 0
  ├ "baseColorFactor"
  │ ├ #FF0000FF
  │ └ #00FF0080
  └ "metallicFactor"
    ├ 1
    └ 0.2
```

The other layouts arrange the same grid as aligned columns, markdown
tables, or JSON, and a label mode decides whether the text layouts
label data with full dot-joined paths, with leaf segments under nested
markdown headings, or not at all.

Each render method takes only the options its layout consumes, so
every combination that compiles is valid and rendering always
succeeds. Options can also be gathered loosely, one field at a time;
the one
fallible step is the matching `resolve_*` method, which rejects any
option that render does not consume:

```rust
let options = TreeGridOptions::default().with_value_children(true);
let tree = grid.render_hierarchy(&options.resolve_hierarchy()?);
```

Values can be any type: the grid renders them through its cell
policy (`TreeGridCells`), and the default policy reads the
pre-rendered `TreeGridValue`s above. A custom policy stores the
caller's own value type -- even a foreign one -- by answering three
questions per value: its text, an optional visual (opaque
pre-rendered bytes with a declared display width, like the stock ANSI
color swatches), and the cell format a node with no explicit format
uses for it.

The `json-pretty` / `json-compact` layouts ride the optional `json`
feature, which pulls in `serde_json`; without it the crate renders
the text layouts only. The JSON renders exist for grids whose policy
provides JSON forms (`TreeGridJsonCells`): the default policy emits
each value's text as a JSON string, and the feature-gated
`TreeGridJsonValue` / `TreeGridJsonValueCells` pair carries a native
JSON form beside a value for pairs that genuinely diverge.

Selection, value sampling, precision policy, terminal-width detection,
and IO stay with the caller: the crate only arranges and serializes
what it is handed.
