# `vxl hierarchy show`

*Part of [`vxl hierarchy`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl hierarchy show <input> [pattern...] [options]
```

Prints the scene graph as a tree with box-drawing glyphs, modeled on the FBX
hierarchy view. The graph is a DAG, not a tree: a node may have multiple
parents, which is instancing, and the roots are exactly the nodes listed in
`rootNodes`. `show` marks shared and instanced nodes and lists
unplaced library nodes, defined as nodes that are neither a root nor a child,
so the structure stays visible rather than implying a strict tree. Each node
shows its name and its referenced child objects. See
[Hierarchy Nodes](../../../../../../projects/voxel-formats/voxj/docs/voxel-json-file-format.md#hierarchy-nodes).

1. `pattern...`: optional gitignore-style patterns matched against the path of
   every node and object. When set, only selected nodes and objects and the
   nodes leading to them print, or only the match roots with
   `--collapse-ancestors`. A plain pattern selects, a leading `!` deselects, a
   trailing `/` selects nodes only, and the last pattern to match a path wins.
   Selecting a node selects its whole subtree; an excluded node prunes its
   subtree. Matching nothing is an error. See
   [Glob patterns](../conventions.md#glob-patterns).
2. `--show-transforms [space] [rot-unit] [precision]`: prepend each node's
   transform as a nested subtree. `space` is `local` (default) or `world`;
   `rot-unit` is `rad` (default) or `deg`; `precision` is the decimal precision
   for alignment (default `2`).

Each object carries two grids: the runtime grid, the tight box around its live
voxels, and the edit grid, the author's build volume, which may add margin
around the runtime grid. The next six flags append one of `origin`, `bounds`, or
`extents` for one of those grids, each measured relative to the placing node. An
edit row is `null` when the build volume matches the runtime grid, so there is no
distinct edit grid. The runtime grid is always shown; an object with no live
voxels reports a zero-size box at its origin.

3. `--show-edit-origins [space] [precision]`: append each object's edit-grid
   origin, its build-volume min corner. `space` is `local` (default), the offset
   from the placing node, or `world`, that corner through the node's world
   transform. `precision` is the decimal places (default `2`).
4. `--show-edit-bounds [precision]`: append the edit grid's `min`/`max` subtree,
   node-relative.
5. `--show-edit-extents [precision]`: append the edit grid's extents
   (`max - min`).
6. `--show-runtime-origins [space] [precision]`: append each object's
   runtime-grid origin, its tight live-box min corner, with the same
   `[space] [precision]` arguments as `--show-edit-origins`.
7. `--show-runtime-bounds [precision]`: append the runtime grid's `min`/`max`
   subtree, node-relative.
8. `--show-runtime-extents [precision]`: append the runtime grid's extents
   (`max - min`).
9. `--show-layers` (flag): append each object's referenced layers as a
   nested subtree, one child per layer in the object's `layers` order, back
   to front, reading `<palette index>: {materials: <count>}`. Two layers may
   share a palette. An object that references no layer prints an empty
   `layers: []` array.
10. `--collapse-ancestors` (flag): hide the ancestor chain above each match root
    and replace it with an `ancestors` marker, omitted when the match root is a
    top-level node. Requires a `pattern`.
11. `--collapse-descendants` (flag): hide the descendants of each match root and
    replace them with a `descendants` marker, omitted when the match root has no
    descendants. Requires a `pattern`.
12. `--collapse-instances` (flag): expand each shared node's first placement and
    print each later placement as a non-expanded stub, rather than expanding
    every placement in full.
13. `--layout <layout>`: how to render the scene graph, and the serialization
    to emit. `hierarchy` (default) is the box-glyph tree; `json-pretty` and
    `json-compact` emit the same tree as the shared read-command envelope, one
    record per node, each `{"label", "values"?, "children"?}`, with the raw
    unquoted name as the label and the `{node: 0}`-style tags, view rows, and
    layer entries as pre-formatted string values. The pattern, collapse, and
    `--show-*` flags shape the tree the same way under every layout.
