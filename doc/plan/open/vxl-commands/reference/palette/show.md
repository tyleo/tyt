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
4. `--format` `auto` | `swatch` | `swatch-value` | `value` (default `auto`):
   how a value is rendered. `auto` prints a colored swatch beside its hex for a
   color attribute and a bare number for a scalar or an extracted channel, since
   a swatch is meaningful only for a color. `swatch` prints swatches alone with
   no text, a colored swatch for a color and a grayscale swatch across the
   `0..1` range for a scalar or channel. `swatch-value` prints those same
   swatches each followed by its value text, so the exact hex or number reads
   beside the color. `value` prints raw values, one per line, the `#RRGGBBAA`
   hex for a color and the bare number otherwise, the form meant for piping into
   other tools.
5. `--json`: emit the palette as JSON instead.

## V2 follow-ups

The first version of `show` prints one attribute of one palette through separate
`--index`, `--attribute`, `--type`, and `--format` flags. The redesign below
turns `show` into a multi-collection inspector: many attributes of one palette,
and one attribute compared across palettes, with the per-collection choices
bundled into a single repeatable selector and the cross-command pieces pulled
into shared conventions. It is recorded here as one later pass so the first
version can land first.

### The `--attribute` selector

Replace the V1 `--index`, `--attribute`, `--type`, and `--format` flags with one
repeatable `--attribute` option that names a whole value collection at once. It
takes three required values and repeats to select several collections from one
palette or across palettes in a single command:

```
--attribute <palette> <attribute> <format>
```

1. `palette`: a palette index, or `*` for every palette.
2. `attribute`: an attribute key, a key with a trailing `.r`, `.g`, `.b`, or
   `.a` color component, or `*` for every attribute of the palette.
3. `format`: how each value renders, one of `auto`, `swatch`, `value`, and
   `swatch-value`.

When no `--attribute` is given at all the command defaults to a single
`--attribute '*' rgba swatch`, every palette's colors as swatches, so a bare
`vxl palette show <file>` is useful on its own. The default applies only in the
absence of any `--attribute`; one or more selectors replace it rather than adding
to it. Examples:

```
vxl palette show model.voxj                    # every palette's rgba as swatches
vxl palette show model.voxj --attribute 0 rgba value
vxl palette show model.voxj \
  --attribute 0 rgba swatch-value \
  --attribute 0 metallic value \
  --attribute '*' roughness value
```

Three required values is deliberate. clap groups a repeated option per occurrence
only at a fixed arity, so three fixed fields parse unambiguously where an
optional trailing field would not, and only `*` needs quoting against the shell.

The type field is gone. `show` reads concrete cells, so it always infers a color
from a `#RRGGBBAA` value and a scalar from a number, and no `--type` is needed.
This also retires the integer and float types considered earlier; a number prints
as it reads.

### Value collections and headers

Each selector resolves to one or more value collections, one collection being an
attribute's values down a palette. A `*` palette or `*` attribute expands to one
collection per match, so `'*' rgba` yields a collection for every palette that
carries `rgba`. Each collection is labeled by a header reading
`{palette}.{attribute}`, with the component appended when one is read, as in
`0.rgba`, `1.rgba`, and `0.rgba.a`.

Collections come out in palette order, then attribute order within a palette. A
`*` that matches nothing yields no collection, while a named palette or attribute
that is absent is an error, so a typo is caught but a broad `'*'` quietly skips a
palette that lacks the attribute. A value that does not match its inferred type,
such as a non-hex string under an inferred color, prints as its raw text.

A color component is read as a byte from `0` to `255`, the value as stored in the
hex, everywhere it appears. Alpha `FF` reads as `255`, not a `0..1` fraction, in
both the rendered output and the JSON, which keeps the displayed channel faithful
to the file and avoids long fractions. A grayscale swatch maps the byte to its
gray level.

### Formats

The third selector field renders each value:

1. `auto`: a colored swatch beside its hex for a color, a bare number for a
   scalar or a color component.
2. `swatch`: swatches alone with no text, a colored swatch for a color and a
   grayscale swatch for a scalar or component.
3. `value`: raw values, the `#RRGGBBAA` hex for a color and the bare number
   otherwise.
4. `swatch-value`: each swatch followed by its value text, the colored swatch
   with its hex and the grayscale swatch with its number.

### Layouts

A `--layout` option arranges the collections and is orthogonal to the
per-collection format. It defaults to `row` for a single collection and `table`
for several, so the default `'*' rgba` selector prints a row on a one-palette
document and a table on a many-palette one:

1. `row`: each collection is one row prefixed by its `{palette}.{attribute}`
   header, with several collections stacked as rows and whitespace aligning the
   value columns so a `value` rendering lines up.
2. `column`: each collection is its own column beneath its header, padded to a
   common width so a `value` rendering reads straight down.
3. `table`: the collections fill an aligned markdown table, one column per
   collection, the `{palette}.{attribute}` labels as the header row, and one row
   per cell index. A shorter palette leaves its column blank past its last cell.

Alignment for the `value` and `swatch-value` forms is measured by the visible
width of a cell, since the swatch escape codes carry no width of their own.

### Shared conventions

Two parts of this belong to every command, not just `show`, and should be
specified once in [conventions](../conventions.md) so the read commands behave
alike.

A locked input format. Reading infers the container from the file extension and
its leading bytes. `--from` already overrides that inference, so it is the lock.
Its values are `voxj` and `voxjz` for the two container forms, the existing
`vmax`, `mvox`, `goxl`, and `qbcl`, and `palette` for a bare palette `.json`,
splitting today's single `voxj` value that covered both containers. Every read
command honors it.

One JSON form. `--json` takes a required `compact` or `pretty` rendering, with no
bare form and no default. It always emits the stored values in their native JSON
types, a number as a number and a `#RRGGBBAA` as a string, so it ignores the
selector's format field and the `--layout`. The selectors still choose which
palettes and attributes appear, and a color component emits its byte. There is no
`type` field; each value already carries its type. For `show` the payload is one
record per collection, in render order, the resolved palette index even when the
selector used `*`:

```json
[
  { "palette": 0, "attribute": "rgba", "values": ["#FF0000FF", "#00FF0080"] },
  { "palette": 0, "attribute": "rgba.a", "values": [255, 128] }
]
```

Give `list`, `show`, `hierarchy show`, `validate`, and `info` one shared envelope
so they all report the same way; this per-record shape is the template, and the
envelope wrapping it is settled when the other read commands land.

### A bare palette input

A palette command should read a standalone palette as readily as a whole
document. Accept a `.json` file holding just a `palettes` array, named by the
`--from` form above and recognized by its content the way the document forms
already are. The `quantize` and `remap` commands take the same bare palette, so
the shape is defined once and shared.

### Checklist

- [ ] Replace `--index`, `--attribute`, `--type`, and `--format` with one
      repeatable `--attribute <palette> <attribute> <format>` selector, three
      required fields, defaulting to a single `'*' rgba swatch` only when no
      `--attribute` is given at all.
- [ ] Support `*` in the palette and attribute fields, expanding to one labeled
      collection per match, under `{palette}.{attribute}[.component]` headers.
- [ ] Read a color component as a `0`-to-`255` byte everywhere, in both the
      rendered output and the JSON.
- [ ] Render the four formats `auto`, `swatch`, `value`, and `swatch-value` per
      collection.
- [ ] Add `--layout row|column|table`, default row for one collection and table
      for several, with collections as table columns and one row per cell index.
- [ ] Specify a locked input format on `--from` in conventions, distinguishing
      voxj and voxjz and naming a bare palette json, adopted by every read
      command.
- [ ] Make `--json` take a required `compact` or `pretty` value, emit native JS
      types with no `type` field, ignore the format and layout, and share one
      envelope across the read commands.
- [ ] Accept a bare palette `.json` input on the palette commands, sharing the
      shape with `quantize` and `remap`.
