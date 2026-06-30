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

Only `voxj` carries format-specific options in this plan; see its page. The
other targets convert with the shared options above.
