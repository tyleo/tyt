# `vxl hierarchy show`

*Part of [`vxl hierarchy`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl hierarchy show <input> [pattern] [options]
```

Prints the scene graph as a tree with box-drawing glyphs, modeled on the FBX
hierarchy view. The graph is a DAG, not a tree: a node may have multiple
parents, which is instancing, and the roots are exactly the nodes listed in
`rootHierarchyNodes`. `show` marks shared and instanced nodes and lists
unplaced library nodes, defined as nodes that are neither a root nor a child,
so the structure stays visible rather than implying a strict tree. Each node
shows its name and its referenced child objects. See
[Hierarchy Nodes](../../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#hierarchy-nodes).

1. `pattern`: an optional glob matched against node paths. When set, only
   matched nodes and their ancestors print, or only matched nodes with
   `--collapse-ancestors`. `**/` is auto-prepended when the pattern does not
   already start with it. See [Glob patterns](../conventions.md#glob-patterns).
2. `--show-transforms [space] [rot-unit] [precision]`: prepend each node's
   transform as a nested subtree. `space` is `local` (default) or `world`;
   `rot-unit` is `rad` (default) or `deg`; `precision` is the decimal precision
   for alignment (default `2`).
3. `--show-bounds [space] [precision]`: append each object's grid `bounds`
   subtree. In `world` space the bounds are reported as the axis-aligned box
   after the placing node's transform.
4. `--show-extents [space] [precision]`: append an extents subtree
   (`max - min`), with the same arguments as `--show-bounds`.
5. `--collapse-ancestors` (flag): with a `pattern`, hide the ancestor chain
   above each match and replace it with an `(ANCESTORS)` marker, omitted when
   the match is a root. No effect without a `pattern`.
6. `--collapse-descendants` (flag): with a `pattern`, hide the descendants of
   each match and replace them with a `(DESCENDANTS)` marker, omitted when the
   match has no descendants. No effect without a `pattern`.
7. `--json`: emit the graph as JSON, including root, instanced, and unplaced
   flags.
