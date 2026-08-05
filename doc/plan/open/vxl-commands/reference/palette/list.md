# `vxl palette list`

*Part of [`vxl palette`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl palette list <input> [filter]... [options]
```

Gives a per-palette overview of the whole document so you can see what is there
before printing any colors. Each palette shows its index and, unless turned off,
its ordered property keys, its material count, and which objects reference it, which
is exactly the index and property that [`show`](show.md),
[`quantize`](quantize.md), and [`remap`](remap.md) ask for. By default it prints
a tree:

```
palettes
├ 0
│ ├ materials: 12
│ ├ properties
│ │ ├ "baseColor"
│ │ └ "metallic"
│ └ objects
│   ├ "Object A"
│   └ "Object B"
└ 1
  ├ materials: 2
  ├ properties
  │ ├ "baseColor"
  │ └ "emissiveStrength"
  └ objects
    └ "Object B"
```

From there, `vxl palette show <input> --property 1 '*' auto auto` prints
palette 1's colors.

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

1. `--show-properties` (default `true`): the ordered property keys.
2. `--show-materials` (default `true`): the material count.
3. `--show-objects` (default `true`): the objects that reference the palette.

`--show-objects false` drops the `objects` column in the table and the `objects`
branch in the tree, and the other two behave the same way.

## Layout

`--layout` `hierarchy` | `tables` | `json-pretty` | `json-compact` (default
`hierarchy`): how to render the listing.

1. `hierarchy` (default): the indented tree above, in the
   [`hierarchy show`](../hierarchy/show.md) idiom, a `palettes` header over one
   branch per palette index, with the material count as a `materials: <n>`
   leaf and
   `properties` and `objects` as subtrees. Property keys and object names are
   user-entered, so they print quoted.
2. `tables`: a `# palettes` heading over one aligned record table, one row
   per palette labeled by its index and one column per enabled field:

   ```
   # palettes

   | label | properties                  | materials | objects            |
   | ----- | --------------------------- | --------- | ------------------ |
   | 0     | baseColor, metallic         | 12        | Object A, Object B |
   | 1     | baseColor, emissiveStrength | 2         | Object B           |
   | 2     | metallic, roughness         | 1         | Object B           |
   ```

3. `json-pretty` and `json-compact`: the listing tree as pretty or compact
   JSON in the shared read-command envelope, one `{"label", "annotation"?,
   "values"?, "children"?}` record per tree node: the `palettes` root over one
   record per palette index, the material count as a native number under
   `materials`, and the property and object names as child records, an
   empty subtree the `"[]"` string value its tree leaf shows.
