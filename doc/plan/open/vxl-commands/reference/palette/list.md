# `vxl palette list`

*Part of [`vxl palette`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl palette list <input> [options]
```

Gives a one-line-per-palette overview of the whole document so you can see what
is there before printing any colors. Each row shows the palette index, its
ordered attribute keys, its cell count, and which objects reference it, which is
exactly the index and attribute that [`show`](show.md), [`quantize`](quantize.md),
and [`remap`](remap.md) ask for. Example:

| index | attributes          | cells | used by            |
| ----- | ------------------- | ----- | ------------------ |
| 0     | rgba, metallic      | 12    | Object A, Object B |
| 1     | rgba                | 2     | Object B           |
| 2     | metallic, roughness | 1     | Object B           |

From there, `vxl palette show <input> --index 1` prints palette 1's colors.

1. `--layout` `markdown` | `pretty-json` | `compact-json` (default `markdown`):
   how to render the listing. `markdown` is the table above; the JSON forms emit
   the listing, including per-palette attribute keys, cell count, and referencing
   object indices, as pretty or compact JSON.
