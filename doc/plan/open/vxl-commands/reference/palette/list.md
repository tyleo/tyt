# `vxl palette list`

*Part of [`vxl palette`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl palette list <input> [filter]... [options]
```

Gives a per-palette overview of the whole document so you can see what is there
before printing any colors. Each palette shows its index and, unless turned off,
its ordered attribute keys, its cell count, and which objects reference it, which
is exactly the index and attribute that [`show`](show.md),
[`quantize`](quantize.md), and [`remap`](remap.md) ask for. By default it prints
a tree:

```
palettes
├ 0
│ ├ cellCount: 12
│ ├ attributes
│ │ ├ rgba
│ │ └ metallic
│ └ objects
│   ├ Object A
│   └ Object B
└ 1
  ├ cellCount: 2
  ├ attributes
  │ └ rgba
  └ objects
    └ Object B
```

From there, `vxl palette show <input> --index 1` prints palette 1's colors.

## Filters

Trailing positional filters narrow the listing to particular palettes. A filter
is a palette index such as `1`, or an inclusive range such as `5-10`. Repeat them
and their matches union, so `vxl palette list model.voxj 1 3 4` lists palettes 1,
3, and 4, and `vxl palette list model.voxj 1-5 10` lists 1 through 5 and 10. Given
no filter, every palette is listed. A filter that matches no palette is an error,
so a stray index is caught rather than silently listing nothing.

## Fields

Three settable booleans choose which fields render beside the always-shown index,
each defaulting to shown so a bare `palette list` prints them all:

1. `--show-attributes` (default `true`): the ordered attribute keys.
2. `--show-cells` (default `true`): the cell count.
3. `--show-objects` (default `true`): the objects that reference the palette.

`--show-objects false` drops the `used by` column in the table and the `objects`
branch in the tree, and the other two behave the same way.

## Layout

`--layout` `hierarchy` | `markdown` | `pretty-json` | `compact-json` (default
`hierarchy`): how to render the listing.

1. `hierarchy` (default): the indented tree above, in the
   [`hierarchy show`](../hierarchy/show.md) idiom, a `palettes` header over one
   branch per palette index, with the cell count as a `cellCount: <n>` leaf and
   `attributes` and `objects` as subtrees.
2. `markdown`: an aligned table, one column per enabled field:

   | index | attributes          | cells | used by            |
   | ----- | ------------------- | ----- | ------------------ |
   | 0     | rgba, metallic      | 12    | Object A, Object B |
   | 1     | rgba                | 2     | Object B           |
   | 2     | metallic, roughness | 1     | Object B           |

3. `pretty-json` and `compact-json`: the listing as pretty or compact JSON, one
   record per palette carrying its index and each enabled field, the referencing
   objects as their indices under `used_by`.
