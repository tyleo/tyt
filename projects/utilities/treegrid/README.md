# treegrid

Hierarchical-data rendering: populate a forest of labeled, data-bearing
nodes, then render it under a chosen layout and label mode.

```rust
let mut grid = TreeGrid::new();

let palette = grid.retain_root(TreeGridLabel::bare("0"));

let base = grid.retain_child(palette, TreeGridLabel::quoted("baseColorFactor"));
grid.push_value(base, TreeGridValue::new("#FF0000FF"));
grid.push_value(base, TreeGridValue::new("#00FF0080"));

let metallic = grid.retain_child(palette, TreeGridLabel::quoted("metallicFactor"));
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

Each render method takes only the options its layout consumes, so every
combination that compiles is valid and rendering always succeeds.
Options can also be gathered loosely, one field at a time; the one
fallible step is the matching `resolve_*` method, which rejects any
option that render does not consume:

```rust
let options = TreeGridOptions::default().with_value_children(true);
let tree = grid.render_hierarchy(&options.resolve_hierarchy()?);
```

Values can be any type: a cell policy (`TreeGridCells`) turns them into
cells. The default policy reads the pre-rendered `TreeGridValue`s above;
a custom policy renders the caller's own type:

```rust
struct TextCells;

impl TreeGridCells for TextCells {
    type Value = String;

    fn text<'a>(&self, value: &'a String) -> Cow<'a, str> {
        Cow::Borrowed(value)
    }
}

let mut grid = TreeGrid::with_cells(TextCells);
let position = grid.retain_root(TreeGridLabel::bare("position"));
grid.push_value(position, "[12.5, 0.5, 10.0]".to_owned());

let tree = grid.render_hierarchy(&TreeGridHierarchyOptions::default());
```

```text
└ position: [12.5, 0.5, 10.0]
```

Each layout rides its own default-on cargo feature named for its
render module (`render_hierarchy`, `render_rows`, `render_columns`,
`render_tables`), its render method arriving on a small extension
trait, so an adopter that renders only one layout can trim the rest. The `json-pretty` / `json-compact` layouts ride the optional
`json` feature, which pulls in `serde_json`; `TreeGridJsonValue` pairs a
value with a native JSON form when its text and JSON diverge:

```rust
let mut grid = TreeGrid::with_cells(TreeGridJsonValueCells);
let metallic = grid.retain_root(TreeGridLabel::quoted("metallicFactor"));
grid.push_value(metallic, TreeGridJsonValue::new("1.00").with_json(json!(1.0)));

let json = grid.render_json_compact();
```

```json
[{"label":"metallicFactor","values":[1.0]}]
```

Float-component colors ride the optional `ty-math` feature: `srgb` /
`srgba` / `lin_srgb` / `lin_srgba` constructors take the
component-generic ty-math color family and render functional-notation
text with a quantized (for linear colors, transfer-encoded) swatch,
all color math through ty-math:

```rust
let value = TreeGridValue::lin_srgba(TyLinSrgbaF64::new(2.0, 1.0, 0.5, 1.0));
// text "lin_srgba(2, 1, 0.5, 1)", HDR red clamped in the swatch alone
```

Selection, value sampling, precision policy, terminal-width detection,
and IO stay with the caller: the crate only arranges and serializes
what it is handed.
