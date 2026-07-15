# treegrid

Hierarchical-data rendering: populate a forest of labeled, data-bearing
nodes, then render it under a chosen layout and label mode.

```rust
let mut grid = TreeGrid::default();

let palette = grid.add_root(TreeGridLabel::bare("0"));

let base = grid.add_child(palette, TreeGridLabel::quoted("baseColorFactor"));
grid.push_value(base, TreeGridValue::new("#FF0000FF"));
grid.push_value(base, TreeGridValue::new("#00FF0080"));

let metallic = grid.add_child(palette, TreeGridLabel::quoted("metallicFactor"));
grid.push_value(metallic, TreeGridValue::new("1"));
grid.push_value(metallic, TreeGridValue::new("0.2"));

let rows = grid.render(&TreeGridLayout::default());
```

The default `rows` layout renders each data node as one labeled row:

```text
0."baseColorFactor" #FF0000FF #00FF0080

0."metallicFactor"  1 0.2
```

The `hierarchy` layout renders the same grid as a box-glyph tree;
`value_children` gives each value its own line:

```rust
let tree = grid.render(&TreeGridLayout::Hierarchy(
    TreeGridHierarchyOptions::default().with_value_children(true),
));
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

A layout only holds the options it consumes, so an invalid
combination cannot be built and `render` never fails. Options can
also be gathered loosely, one field at a time, and resolved; the one
fallible step is `resolve`, which rejects any option the chosen
layout does not consume:

```rust
let options = TreeGridOptions::default()
    .with_layout(TreeGridLayoutKind::Hierarchy)
    .with_value_children(true);
let tree = grid.render(&options.resolve()?);
```

Each value's native JSON form and the `json-pretty` / `json-compact`
layouts ride the optional `json` feature, which pulls in `serde_json`;
without it the crate renders the text layouts only.

Selection, value sampling, precision policy, terminal-width detection,
and IO stay with the caller: the crate only arranges and serializes
what it is handed.
