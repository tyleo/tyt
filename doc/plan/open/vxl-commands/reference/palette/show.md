# `vxl palette show`

*Part of [`vxl palette`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl palette show <input> [--index 0] [--attribute rgba] [options]
```

Prints one palette's selected attribute.

1. `--index <n>` (default `0`): which palette to show.
2. `--attribute <key>` (default `rgba`): which attribute to show.
3. `--format` `auto` | `swatch` | `string` (default `auto`): `auto` prints
   colored swatches for `rgba` and numeric values for every other attribute,
   since swatches are meaningful only for color. `swatch` forces colored
   swatches. `string` prints raw values, one per line: the `#RRGGBBAA` hex for
   `rgba` and the literal value otherwise, the form meant for piping into other
   tools.
4. `--json`: emit the palette as JSON instead.
