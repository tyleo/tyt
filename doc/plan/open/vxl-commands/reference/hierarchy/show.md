# `vxl hierarchy show`

*Part of [`vxl hierarchy`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl hierarchy show <input> [pattern...] [options]
```

Prints the scene graph as a tree with box-drawing glyphs, modeled on the FBX
hierarchy view. The graph is a DAG, not a tree: a node may have multiple
parents, which is instancing, and the roots are exactly the nodes listed in
`rootHierarchyNodes`. `show` marks shared and instanced nodes and lists
unplaced library nodes, defined as nodes that are neither a root nor a child,
so the structure stays visible rather than implying a strict tree. Each node
shows its name and its referenced child objects. See
[Hierarchy Nodes](../../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#hierarchy-nodes).

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
3. `--show-bounds [space] [precision]`: append each object's grid `bounds`
   subtree. In `world` space the bounds are reported as the axis-aligned box
   after the placing node's transform.
4. `--show-extents [space] [precision]`: append an extents subtree
   (`max - min`), with the same arguments as `--show-bounds`.
5. `--show-palettes` (flag): append each object's referenced palettes as a
   nested subtree, one child per palette in the object's palette-reference
   order, reading `index: {cells: <count>}`. An object that references no
   palette prints an empty `palettes: []` array.
6. `--collapse-ancestors` (flag): hide the ancestor chain above each match root
   and replace it with an `ancestors` marker, omitted when the match root is a
   top-level node. Requires a `pattern`.
7. `--collapse-descendants` (flag): hide the descendants of each match root and
   replace them with a `descendants` marker, omitted when the match root has no
   descendants. Requires a `pattern`.
8. `--collapse-instances` (flag): expand each shared node's first placement and
   print each later placement as a non-expanded stub, rather than expanding every
   placement in full. This command prints only the tree; it has no `--layout`.
