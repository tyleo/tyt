# `vxl vox-doc to`

*Part of the [Vxl Command-Line Reference](../../../README.md).*

```
vxl vox-doc to <format> <input> [output] [options]
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
   objects; see [Object selectors](../../conventions.md#object-selectors). The
   written hierarchy is the smallest that still places every selected object:
   a node survives when its subtree places one, keeping its transform and its
   surviving children in order, and roots, node order, and object order are
   preserved. Palettes and value pools ride through untouched. Given no
   selector, the whole document converts.

`vmax` additionally accepts `--object-size auto|32|64|128|256|512`.
The default `auto` keeps a loaded object's supported workspace size when its
canvas fits; new objects use 256 or 512. A fixed size centers each object's
occupied voxels in that cube, compacts empty canvas margins, and errors when
an object cannot fit. Its scene position, parent transforms and rotation
pivot remain equivalent; the editor camera follows the internal grid.

```sh
vxl vox-doc to vmax scene.voxj --object-size 256
```

See the `voxj` page for its format-specific options.
