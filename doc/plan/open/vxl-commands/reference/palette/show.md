# `vxl palette show`

*Part of [`vxl palette`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl palette show <input> [--property <palette> <property> <presentation> <reading>]... [--layout rows]
    [--label concat] [--header-level <level>] [--table-shape nested] [--width terminal]
```

Prints one or more palette value collections. A collection is a property's
values down a palette. One command shows many properties of one palette, or one
property compared across palettes, by repeating a single selector.

## The `--property` selector

`--property` names a whole value collection at once. It takes four required
values and repeats to select several collections from one palette or across
palettes in a single command:

```
--property <palette> <property> <presentation> <reading>
```

1. `palette`: a palette index, or `*` for every palette.
2. `property`: a property key, a key with a trailing component suffix
   (`.r`/`.g`/`.b`/`.a` or `.x`/`.y`/`.z`/`.w`), or `*` for every property
   of the palette.
3. `presentation`: what renders, one of `auto`, `swatch`, `swatch-value`,
   and `value`. See [Presentations](#presentations).
4. `reading`: how the values spell, one of `auto`, `linear-float`, `plain`,
   `srgb-float`, and `srgb-hex`. See [Readings](#readings).

When no `--property` is given the command defaults to a single
`--property '*' '*' auto auto`, every palette's every property auto-rendered,
so a bare `vxl palette show <file>` is useful on its own. The default applies
only in the absence of any `--property`; one or more selectors replace it
rather than adding to it. Examples:

```
vxl palette show model.voxj                    # every property, auto-rendered
vxl palette show model.voxj --property 0 baseColor value auto
vxl palette show model.voxj \
  --property 0 baseColor swatch-value srgb-hex \
  --property 0 metallic value plain \
  --property '*' roughness value plain
```

Four required values is deliberate. clap flattens a repeatable multi-value
option into one list, and the fixed arity is what chunks that list into
selectors; each field is one shell word from a closed vocabulary, so help and
completion can offer it per position, and only `*` needs quoting against the
shell.

There is no type field and no `--type` flag. A value pool carries only a
shape, the property name classifies per the format's glTF vocabulary, and a
color reading is the color assertion for a custom key, per selector instead
of per command.

## Value collections and labels

Each selector resolves to one or more value collections. A `*` palette or `*`
property expands to one collection per match, so `'*' baseColor` yields a
collection for every palette that carries `baseColor`. A property yields
one value per material in material order. Each collection is
labeled by its path, `{palette}."{property}"`, the property quoted, with the
component appended when one is read, as in `0."baseColor"`,
`1."baseColor"`, and `0."baseColor".a`. The
[`--label` flag](#labels) chooses how the text layouts spend that path.

Collections come out in selector order, then palette order, then property order
within a palette. A `*` that matches nothing
yields no collection, while a named
palette or property that is absent is an error, so a typo is caught but a broad
`'*'` quietly skips a palette that lacks the property.

A component suffix is shape addressing, not a color claim: it is legal on any
vector value pool whose width exceeds its index, through either alias set,
and the reading spells what it selects. Under `srgb-hex` a component spells
its two-digit hex pair, the same pair the whole-color spelling carries; under
the float readings it spells a float, and under `plain` the stored number. A
component's grayscale swatch under a color reading maps the channel's byte
from the whole-color quantize, so the hex pair and the swatch agree; under
`plain` it maps the raw `0..1` value, since no color is asserted.

## Presentations

The third selector field chooses what renders:

1. `auto`: a colored swatch beside its text for a whole color, else bare
   text.
2. `swatch`: swatches alone with no text, a colored swatch for a color and a
   grayscale swatch for a scalar or component.
3. `value`: text alone.
4. `swatch-value`: each swatch followed by its text.

`swatch` and `swatch-value` extend the grayscale ramp to scalars and
components. A value with no visual, a `bool` for one, renders its text under
every presentation. Every swatch shows the color's sRGB appearance, whatever
reading spells the text.

## Readings

The fourth selector field spells the values, carrying the two choices a value
pool's shape cannot say: whether the sRGB transfer applies, and hex versus
numbers.

1. `auto`: the key's default per the glTF vocabulary. A vocabulary color name
   reads `srgb-hex` and holds to the vocabulary's standards, erroring on a
   non-color shape or a component outside `[0, 1]`; everything else reads
   `plain`: the vocabulary scalars, every custom key, and every non-vector
   kind.
2. `linear-float`: no transfer; `lin_srgb(...)` / `lin_srgba(...)` functional
   text for a whole color, the stored float for a component.
3. `plain`: no transfer; the stored value as it is, arrays and text included.
4. `srgb-float`: the transfer applied; `srgb(...)` / `srgba(...)` functional
   text for a whole color, the encoded float for a component.
5. `srgb-hex`: the transfer applied; `#RRGGBB` / `#RRGGBBAA` for a whole
   color, the two-digit hex pair for a component.

The three color readings apply to `vec-3-float` and `vec-4-float` value pools
only, whole or component, and error on every other shape; they are the color
assertion a custom key needs. Alpha never transfer-encodes: `srgb-hex`
quantizes it raw and `srgb-float` passes it through. The two sRGB readings
require every spelled component in `[0, 1]`, alpha included, and error
outside it, never clamp.

## Layouts

`--layout` arranges the collections and chooses the serialization. It defaults
to `rows` and is orthogonal to the per-collection presentation and reading.
The text layouts share those and only place the collections; the two JSON
layouts emit the records directly and ignore the presentation.

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

1. `none`: no labels. An error under `tables`, whose columns cannot be headed
   by nothing.
2. `concat` (default): the full dot-joined path, as in `0."baseColor".a`.
   Inline on `rows` and `columns`; under `tables` the headings nest exactly
   like `header` but each carries its full path.
3. `header`: the ancestor path becomes nested markdown headings, `# 0` and
   `## "baseColor"`, and each collection beneath is labeled by its leaf
   segment alone, so palettes read as per-palette sections.

`--header-level` sets the markdown level of the shallowest heading, so
embedded output sits at the right depth under a host document's headings; the
headings start at `#` when it is omitted. It applies to the renders that emit
headings, `--label header` and the nested and records `tables` shapes, and is
an error on a render that emits none. A heading that would nest past markdown's level 6
renders as a bold `**label**` line instead.

## The JSON envelope

The two JSON layouts emit each value as its reading spells it, in its native
JSON type: a `plain` number as a number, a hex whole or pair as a string, a
functional spelling as its text. They ignore the selector's presentation,
which only chooses the swatches the JSON does not carry. The selectors still
choose which palettes and properties appear. There is no `type` field; each
value already carries its type. The payload is the shared read-command
envelope: one record
per node of the collection tree, each `{"label", "annotation"?, "values"?,
"children"?}`, with the raw unquoted segment as the label. A palette is a root
record labeled by its resolved index even when the selector used `*`, a
property nests under its palette, and a component under its property.
Consecutive collections sharing
a palette nest under one record; a palette revisited later starts a fresh
record, so the records keep selector order:

```json
[
  {
    "label": "0",
    "children": [
      {
        "label": "baseColor",
        "values": ["#FF0000FF", "#00FF0080"],
        "children": [{ "label": "a", "values": ["FF", "80"] }]
      },
      { "label": "emissiveStrength", "values": [5] }
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

- [x] One repeatable `--property <palette> <property> <presentation>
      <reading>` selector, four required fields, defaulting to a single
      `'*' '*' auto auto` only when no `--property` is given at all; no
      `--index`, `--attribute`, `--type`, or `--format` flags.
- [x] Support `*` in the palette and property fields, expanding to one labeled
      collection per match, under `{palette}."{property}"[.component]` labels.
- [x] Read a component through either alias set on any wide-enough vector,
      spelled by the reading: the hex pair under `srgb-hex`, floats under the
      float readings, the stored number under `plain`.
- [x] Render the four presentations `auto`, `swatch`, `value`, and
      `swatch-value` and the five readings `auto`, `linear-float`, `plain`,
      `srgb-float`, and `srgb-hex` per collection.
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
