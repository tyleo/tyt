# `vxl to`

*Part of the [Vxl Command-Line Reference](../../README.md).*

```
vxl to <format> <input> [output] [options]
```

Converts a voxel file from any supported input format to `<format>`. This
command exists today. Output is optional and defaults to the input stem with
the target's extension; the source format is recognized by leading bytes or
inferred from the extension, and overridden with `--from`.

The format targets are:

- [`voxj`](voxj.md): the voxel-json document. Also the canonical way to
  re-encode, pack, and unpack a document.
- `vmax`: the Voxel Max package.
- `goxl`: the Goxel `.gox` file.
- `mvox`: the MagicaVoxel `.vox` file.
- `qbcl`: the Qubicle `.qbcl` file.

Every target takes:

1. `--from <format>`: source voxel format. Inferred from the input extension
   when omitted.
2. `--select <glob>` / `--select-index <index>`: write only the selected
   objects; see [Object selectors](../conventions.md#object-selectors). The
   written hierarchy is the smallest that still places every selected object:
   a node survives when its subtree places one, keeping its transform and its
   surviving children in order, and roots, node order, and object order are
   preserved. Palettes and value pools ride through untouched. Given no
   selector, the whole document converts.

Only `voxj` carries format-specific options in this plan; see its page.
