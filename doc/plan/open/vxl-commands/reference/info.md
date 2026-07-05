# `vxl info`

*Part of the [Vxl Command-Line Reference](../README.md).*

```
vxl info <input> [options]
```

Reports what a document contains, surfacing the format internals that voxel
counts and bounds alone miss: the `version`; per-object `bounds`, voxel count,
and the position and sample encodings in use; each palette's attribute set and
material count; whether `editState` and `ext` namespaces are present; and the root,
instanced, and unplaced nodes in the hierarchy.

1. `--layout` `markdown` | `pretty-json` | `compact-json` (default `markdown`):
   how to render the report. `markdown` is the human-readable tables; the JSON
   forms emit the same report as pretty or compact JSON.
