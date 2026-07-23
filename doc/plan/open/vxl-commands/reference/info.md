# `vxl info`

*Part of the [Vxl Command-Line Reference](../README.md).*

```
vxl info <input> [options]
```

Reports what a document contains, surfacing the format internals that voxel
counts and bounds alone miss: the `version`; per-object `bounds`, voxel count,
layer and sampled-layer counts, and the position and sample encodings in use;
each palette's property set, scalar pins marked, and
material count; whether `editState` and `ext` namespaces are present; and the root,
instanced, and unplaced nodes in the hierarchy.

1. `--layout` `markdown` | `pretty-json` | `compact-json` (default `markdown`):
   how to render the report. `markdown` is the human-readable form, a
   `# {input}` title over `## Document`, `## Palettes`, and `## Objects`
   record tables, each row labeled under the fixed `label` column. The JSON
   forms emit the same report in the shared read-command envelope, one
   `{"label", "annotation"?, "values"?, "children"?}` record per node:
   `document`, `palettes`, and `objects` roots whose fields carry snake_case
   labels and values in their native JSON types, a bounds or
   origin triple as a three-number series, a scalar property carrying
   `"annotation": "(scalar)"`, and absent fields (`voxj_version`,
   `edit_bounds`) omitted.
