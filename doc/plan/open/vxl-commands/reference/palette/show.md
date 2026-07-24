# `vxl palette show`

*Part of [`vxl palette`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl palette show <input> [--property <palette> <property> <format>]... [--layout rows]
    [--label concat] [--header-level <level>] [--table-shape nested] [--width terminal]
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

## Value collections and labels

Each selector resolves to one or more value collections. A `*` palette or `*`
property expands to one collection per match, so `'*' baseColorFactor` yields a
collection for every palette that carries `baseColorFactor`. An array property
yields one value per material in material order; a scalar property yields a
one-value collection, its pinned palette-wide value, read through the same
pool-kind classification. Each collection is
labeled by its path, `{palette}."{property}"`, the property quoted, with the
component appended when one is read and a ` (scalar)` annotation suffixed on a
scalar property, as in `0."baseColorFactor"`, `1."baseColorFactor"`,
`0."baseColorFactor".a`, and `0."emissiveStrength" (scalar)`. The
[`--label` flag](#labels) chooses how the text layouts spend that path.

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

Wherever a format shows a color's text, an sRGB pool renders `#RRGGBBAA` hex
(`#RRGGBB` without alpha) and a linear pool renders `lrgb(...)` / `lrgba(...)`
functional notation, whose float components carry HDR values no hex can hold.

## Layouts

`--layout` arranges the collections and chooses the serialization. It defaults to
`rows` and is orthogonal to the per-collection format. The text layouts share
that format and only place the collections; the two JSON layouts emit the
records directly and ignore the format.

1. `hierarchy`: a box-glyph tree of palettes, properties, and components, each
   collection's values inline on its node.
2. `rows` (default): each collection is one row prefixed by its label, the
   labels padded to the longest so the first value of each row lines up, and
   the rows separated by a blank line. Only the label is padded; the values are
   not column-aligned. Swatch cells abut into a continuous strip; the other
   formats, and a swatch value with no swatch such as a bool, separate their
   values with a single space.
3. `columns`: each collection is its own column beneath its label, padded to a
   common width so a `value` rendering reads straight down.
4. `tables`: the collections fill aligned markdown tables led by a `#` column
   of 0-based material indices, one column per collection headed by its label,
   and one row per material index. A shorter palette leaves its column blank
   past its last material. `--table-shape` picks the shape: `nested` (default)
   groups one table per palette under nested headings; `flat` is one table
   over every collection with full-path column headers, the cross-palette
   comparison view; and `records` transposes to one row per property under
   each palette's heading, with a `label` column, a `value` column of the
   row's own values, and one column per component path.
5. `json-pretty`: the collection tree as indented JSON.
6. `json-compact`: the collection tree as single-line JSON.

Alignment for the `value` and `swatch-value` forms is measured by the visible
width of a cell, since the swatch escape codes carry no width of their own.

## Labels

`--label` chooses how the text layouts `rows`, `columns`, and `tables` label
each collection. The `hierarchy` and JSON layouts carry the labels
structurally, so setting `--label` with them is an error rather than a silent
no-op.

1. `none`: no labels; the ` (scalar)` annotation drops with the label it
   rides. An error under `tables`, whose columns cannot be headed by nothing.
2. `concat` (default): the full dot-joined path, as in `0."baseColorFactor".a`.
   Inline on `rows` and `columns`; under `tables` the headings nest exactly
   like `header` but each carries its full path.
3. `header`: the ancestor path becomes nested markdown headings, `# 0` and
   `## "baseColorFactor"`, and each collection beneath is labeled by its leaf
   segment alone, so palettes read as per-palette sections.

`--header-level` sets the markdown level of the shallowest heading, so
embedded output sits at the right depth under a host document's headings; the
headings start at `#` when it is omitted. It applies to the renders that emit
headings, `--label header` and the nested and records `tables` shapes, and is
an error on a render that emits none. A heading that would nest past markdown's level 6
renders as a bold `**label**` line instead.

## The JSON envelope

The two JSON layouts emit the stored values in their native JSON types, a number
as a number and a `#RRGGBBAA` as a string, so they ignore the selector's format
field. The selectors still choose which palettes and properties appear, and a
color component emits its byte. There is no `type` field; each value already
carries its type. The payload is the shared read-command envelope: one record
per node of the collection tree, each `{"label", "annotation"?, "values"?,
"children"?}`, with the raw unquoted segment as the label. A palette is a root
record labeled by its resolved index even when the selector used `*`, a
property nests under its palette, a component under its property, and a scalar
collection carries `"annotation": "(scalar)"`. Consecutive collections sharing
a palette nest under one record; a palette revisited later starts a fresh
record, so the records keep selector order:

```json
[
  {
    "label": "0",
    "children": [
      {
        "label": "baseColorFactor",
        "values": ["#FF0000FF", "#00FF0080"],
        "children": [{ "label": "a", "values": [255, 128] }]
      },
      { "label": "emissiveStrength", "annotation": "(scalar)", "values": [5] }
    ]
  }
]
```

## Width

`--width` wraps the `rows` layout so a wide palette folds onto continuation
lines, each indented under the row's first value, rather than running off as one
line the terminal mangles. It takes one of:

1. `terminal` (default): wrap to the terminal width. When stdout is not a
   terminal, as when the output is piped or redirected, it does not wrap, so a
   pager or file gets the full line.
2. `unlimited`: never wrap; one line per collection.
3. a column count, such as `--width 80`: wrap to that many columns.

It applies to `rows`; the other layouts ignore it.

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
   and `info`. Settled by the [treegrid plan](../../../treegrid/README.md) as
   the record envelope above, which `show` now emits; the remaining read
   commands adopt it as they migrate to the shared renderer.

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
- [x] Share one JSON envelope across the read commands, settled as the treegrid
      record envelope `show` emits; the remaining read commands adopt it as
      they migrate.
- [ ] Accept a bare palette `.json` input on the palette commands, sharing the
      shape with `quantize` and `remap`.
