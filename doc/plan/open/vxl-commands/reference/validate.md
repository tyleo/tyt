# `vxl validate`

*Part of the [Vxl Command-Line Reference](../README.md).*

```
vxl validate <input> [options]
```

Checks a voxel-json document against the spec's
[Validation](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#validation)
checklist and exits non-zero on any failure. The checks include:

1. a recognized `version`.
2. all indices in range.
3. well-formed position data with correct decoded byte lengths and zero pad
   bits.
4. unique voxel positions.
5. tight `bounds`.
6. one sample channel per layer with correct per-channel lengths.
7. well-formed `valuePools` whose values match their declared `kind`.
8. palettes:
   1. property names are unique;
   2. row-major `materials` rows hold in-range value-indices;
   3. every palette has at least one material.
9. an acyclic hierarchy.
10. no zero `scale` component.
11. unit `rotation` quaternions within tolerance.
12. when present, an `editState` whose edit grid contains each runtime grid.

The one item a validator cannot confirm, that sample order matches the position
block's voxel order, is reported as unverifiable.

1. `--layout` `tables` | `json-pretty` | `json-compact` (default `tables`):
   how to render the report. `tables` is a human-readable per-check list; the
   JSON forms emit the report in the shared read-command envelope, one
   `{"label", "annotation"?, "values"?, "children"?}` record per node: `name`
   and `valid` roots, then a `checks` root with one child per check bearing
   its result (`passed`, `failed`, `unverifiable`) as a string value, a
   failed check's messages under a `failures` child.
