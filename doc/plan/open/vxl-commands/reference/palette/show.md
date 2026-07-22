# `vxl palette show`

*Part of [`vxl palette`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl palette show <input> [--property <palette> <property> <format>]... [--layout row] [--width terminal]
```

Prints one or more palette value collections. A collection is a property's
values down a palette. One command shows many properties of one palette, or one
property compared across palettes, by repeating a single selector.

## The `--property` selector

`--property` names a whole value collection at once. It takes three required
values and repeats to select several collections from one palette or across
palettes in a single command:

```
--property <palette> <property> <format>
```

1. `palette`: a palette index, or `*` for every palette.
2. `property`: a property key, a key with a trailing `.r`, `.g`, `.b`, or
   `.a` color component, or `*` for every property of the palette.
3. `format`: how each value in the collection renders, one of `auto`, `swatch`,
   `value`, and `swatch-value`.

When no `--property` is given the command defaults to a single
`--property '*' '*' auto`, every palette's every property auto-rendered, so a bare
`vxl palette show <file>` is useful on its own. The default applies only in the
absence of any `--property`; one or more selectors replace it rather than adding
to it. Examples:

```
vxl palette show model.voxj                    # every property, auto-rendered
vxl palette show model.voxj --property 0 baseColorFactor value
vxl palette show model.voxj \
  --property 0 baseColorFactor swatch-value \
  --property 0 metallicFactor value \
  --property '*' roughnessFactor value
```

Three required values is deliberate. clap groups a repeated option per occurrence
only at a fixed arity, so three fixed fields parse unambiguously where an
optional trailing field would not, and only `*` needs quoting against the shell.

There is no type field. `show` reads concrete values and classifies each by
its bound value pool's kind, a color from a color-kind pool and a scalar from a
number pool, and a number prints as it reads.

## Value collections and headers

Each selector resolves to one or more value collections. A `*` palette or `*`
property expands to one collection per match, so `'*' baseColorFactor` yields a
collection for every palette that carries `baseColorFactor`. An array property
yields one value per material in material order; a scalar property yields a
one-value collection, its pinned palette-wide value, read through the same
pool-kind classification. Each collection is
labeled by a header reading `{palette}."{property}"`, the property quoted, with
the component appended when one is read and a ` (scalar)` marker on a scalar
property, as in `0."baseColorFactor"`, `1."baseColorFactor"`,
`0."baseColorFactor".a`, and `0."emissiveStrength" (scalar)`.

Collections come out in selector order, then palette order, then property order
within a palette, array properties before scalar. A `*` that matches nothing
yields no collection, while a named
palette or property that is absent is an error, so a typo is caught but a broad
`'*'` quietly skips a palette that lacks the property. A value that does not
match its inferred type, such as a non-hex string under an inferred color, prints
as its raw text.

A color component is read as a byte from `0` to `255`, the value as stored in the
hex, everywhere it appears. Alpha `FF` reads as `255`, not a `0..1` fraction, in
both the rendered output and the JSON, which keeps the displayed channel faithful
to the file and avoids long fractions. A grayscale swatch maps the byte to its
gray level.

## Formats

The third selector field renders each value:

1. `auto`: a colored swatch beside its hex for a color, a bare number for a
   scalar or a color component.
2. `swatch`: swatches alone with no text, a colored swatch for a color and a
   grayscale swatch for a scalar or component.
3. `value`: raw values, the `#RRGGBBAA` hex for a color and the bare number
   otherwise.
4. `swatch-value`: each swatch followed by its value text, the colored swatch
   with its hex and the grayscale swatch with its number.

## Layouts

`--layout` arranges the collections and chooses the serialization. It defaults to
`row` and is orthogonal to the per-collection format. The text layouts share that
format and only place the collections; the two JSON layouts emit the records
directly and ignore the format.

1. `row` (default): each collection is one row prefixed by its
   `{palette}."{property}"` header, the headers padded to the longest so the first
   value of each row lines up, and the rows separated by a blank line. Only the
   header is padded; the values are not column-aligned. Swatch cells abut into a
   continuous strip; the other formats, and a swatch value with no swatch such as
   a bool, separate their values with a single space.
2. `row-no-header`: `row` with the header column dropped.
3. `column`: each collection is its own column beneath its header, padded to a
   common width so a `value` rendering reads straight down.
4. `column-no-header`: `column` with the header row dropped.
5. `markdown`: the collections fill an aligned markdown table, one column per
   collection, the `{palette}."{property}"` labels as the header row, and one row
   per material index. A shorter palette leaves its column blank past its last
   material.
6. `pretty-json`: the collection records as indented JSON.
7. `compact-json`: the collection records as single-line JSON.

Alignment for the `value` and `swatch-value` forms is measured by the visible
width of a cell, since the swatch escape codes carry no width of their own.

The two JSON layouts emit the stored values in their native JSON types, a number
as a number and a `#RRGGBBAA` as a string, so they ignore the selector's format
field. The selectors still choose which palettes and properties appear, and a
color component emits its byte. There is no `type` field; each value already
carries its type. The payload is one record per collection, in render order, the
resolved palette index even when the selector used `*`; a scalar collection
additionally carries `"scalar": true`:

```json
[
  { "palette": 0, "property": "baseColorFactor", "values": ["#FF0000FF", "#00FF0080"] },
  { "palette": 0, "property": "baseColorFactor.a", "values": [255, 128] },
  { "palette": 0, "property": "emissiveStrength", "scalar": true, "values": [5] }
]
```

## Width

`--width` wraps the `row` layouts so a wide palette folds onto continuation
lines, each indented under the row's first value, rather than running off as one
line the terminal mangles. It takes one of:

1. `terminal` (default): wrap to the terminal width. When stdout is not a
   terminal, as when the output is piped or redirected, it does not wrap, so a
   pager or file gets the full line.
2. `unlimited`: never wrap; one line per collection.
3. a column count, such as `--width 80`: wrap to that many columns.

It applies to `row` and `row-no-header`; the other layouts ignore it.

## Deferred

These belong to every read command, not just `show`, and land with the suite of
read commands so they are settled once and shared:

1. A locked input format on `--from`, splitting today's single `voxj` value into
   `voxj` and `voxjz` for the two container forms and adding `palette` for a bare
   palette `.json`, adopted by every read command. Specified once in
   [conventions](../conventions.md).
2. A bare palette `.json` input holding just a `palettes` array, recognized by
   its content the way the document forms are, shared with `quantize` and
   `remap`.
3. One shared JSON envelope across `list`, `show`, `hierarchy show`, `validate`,
   and `info`. The per-record shape above is the template; the envelope wrapping
   it is settled when the other read commands land.

## Checklist

- [x] Replace `--index`, `--attribute`, `--type`, and `--format` with one
      repeatable `--attribute <palette> <attribute> <format>` selector, three
      required fields, defaulting to a single `'*' '*' auto` only when no
      `--attribute` is given at all.
- [x] Support `*` in the palette and attribute fields, expanding to one labeled
      collection per match, under `{palette}.{attribute}[.component]` headers.
- [x] Read a color component as a `0`-to-`255` byte everywhere, in both the
      rendered output and the JSON.
- [x] Render the four formats `auto`, `swatch`, `value`, and `swatch-value` per
      collection.
- [x] Add `--layout`, defaulting to `row`, with `column` and a `markdown` table
      arranging the collections and `pretty-json` and `compact-json` emitting the
      records.
- [x] Add `--width terminal|unlimited|<columns>` to wrap the `row` layouts,
      defaulting to the terminal width and not wrapping when stdout is not a
      terminal.
- [ ] Specify a locked input format on `--from` in conventions, distinguishing
      voxj and voxjz and naming a bare palette json, adopted by every read
      command.
- [ ] Share one JSON envelope across the read commands, wrapping the per-record
      shape `show` emits.
- [ ] Accept a bare palette `.json` input on the palette commands, sharing the
      shape with `quantize` and `remap`.
