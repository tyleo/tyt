# `vxl palette show`

*Part of [`vxl palette`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl palette show <input> [--index 0] [--attribute rgba] [options]
```

Prints one palette's selected attribute.

1. `--index <n>` (default `0`): which palette to show.
2. `--attribute <key>[.component]` (default `rgba`): which attribute to show. A
   color attribute reads one component at a time with the
   [`mesh`](../mesh.md#material-and-texture-maps) packing grammar, so
   `--attribute rgba.a` shows straight alpha and `--attribute rgba.r` one color
   channel, each as a scalar, while a bare `--attribute rgba` shows the whole
   color. A scalar attribute names no component, so `--attribute metallic.r` is
   an error, the rule `--texture-map` follows.
3. `--type` `scalar` | `color` (default inferred): how to interpret the
   attribute's values. By default `show` infers the type from the stored value,
   a `#RRGGBBAA` string is a color and a number is a scalar, so the type rarely
   needs stating. Set it to assert the type a custom key carries, matching the
   `type` of a `mesh` `--define-attribute` binding, so a palette preview reads a
   key exactly as the mesh material packing will. `color` enables the
   `.component` access above, and `scalar` rejects it.
4. `--format` `auto` | `swatch` | `string` (default `auto`): how a value is
   rendered. `auto` prints colored swatches for a color attribute and numeric
   values for a scalar or an extracted channel, since a swatch is meaningful
   only for a color. `swatch` forces swatches: a color renders as colored
   swatches and a scalar or channel as grayscale swatches across the `0..1`
   range. `string` prints raw values, one per line, the `#RRGGBBAA` hex for a
   color and the bare number otherwise, the form meant for piping into other
   tools.
5. `--json`: emit the palette as JSON instead.
