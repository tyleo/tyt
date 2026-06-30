# `vxl validate`

*Part of the [Vxl Command-Line Reference](../README.md).*

```
vxl validate <input> [options]
```

Checks a voxel-json document against the spec's
[Validation](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#validation)
checklist and exits non-zero on any failure. The checks include a recognized
`version`; all indices in range; well-formed position data with correct decoded
byte lengths and zero pad bits; unique voxel positions; tight `bounds`; sample
arity matching `paletteRefs` and correct per-channel lengths; well-formed
palette rows and `rgba` strings; an acyclic hierarchy; no zero `scale`
component; unit `rotation` quaternions within tolerance; and, when present, an
`editState` whose edit grid contains each runtime grid. The one item a
validator cannot confirm, that sample order matches the position block's voxel
order, is reported as unverifiable.

1. `--json`: emit a structured report of every check and its result instead of
   human-readable output.
