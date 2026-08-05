# palette show design

The target design for the `palette show` selector, drafted for the
owner's review; iteration 10 lands it in code once approved. The
rulings that shaped it are in the decisions log.

## The two axes

A selector answers two questions: which values to show, and how to read
and render them. Today a third thing intrudes: `--type color` asserts
what a custom property is, and the rendering follows from that
classification. The format made color a reading rather than a fact of
the value pool, and the selector follows the same rule the rest of the
way: asking for a color rendering is the color reading, so no type
assertion survives.

## The selector

`--property <palette> <property> <presentation> <reading>` carries four
fixed fields: clap flattens a repeatable multi-value option into one
list, and the fixed arity is what chunks that list into selectors. Each
field is one shell word from a closed vocabulary, so help documents and
completion offers them per position, which a composite
presentation-and-reading token could never do.

1. `<palette>`: a palette index, or `*` for every palette.
2. `<property>`: a property key with an optional component suffix, or
   `*` for every property.
3. `<presentation>`: what renders, per the presentation table.
4. `<reading>`: how the numbers spell, per the reading table.

## The component suffix

`<key>.<component>` selects one component of a vector property. Two
alias sets name the same indices:

| Index | Color | Vector |
| ----- | ----- | ------ |
| 0     | `.r`  | `.x`   |
| 1     | `.g`  | `.y`   |
| 2     | `.b`  | `.z`   |
| 3     | `.a`  | `.w`   |

A component suffix is shape addressing, not a color claim. It is legal
on any vector value pool, float or int, whose width exceeds the index,
and the reading formats the component it selects: `normal.y` reads a
`vec-3-float` with no color reading anywhere, and `tint.w` errors on
the same pool because the width is three. A component on a scalar or
non-vector value pool errors.

The component grammar is shared with the mesh channel expressions, so
the aliases land in the shared parser and a mesh packing accepts
`normal.x` the day show does.

## The presentation and the reading

`<presentation>` keeps today's four values.

| Presentation   | Renders                                                    |
| -------------- | ---------------------------------------------------------- |
| `auto`         | a swatch beside the text for a whole color, else bare text |
| `swatch`       | swatches alone                                             |
| `swatch-value` | each swatch followed by its text                           |
| `value`        | text alone                                                 |

`swatch` and `swatch-value` extend the grayscale ramp to scalars and
components. A value with no visual, a `bool` for one, renders its text
under every presentation.

`<reading>` spells the value's numbers, carrying the two choices the
value pool kinds no longer say: whether the sRGB transfer applies, and
hex versus numbers.

| Reading        | Transfer | Whole vector                       | Component         |
| -------------- | -------- | ---------------------------------- | ----------------- |
| `auto`         | by key   | the key's default                  | the key's default |
| `linear-float` | no       | `lin_srgb(...)` / `lin_srgba(...)` | stored float      |
| `plain`        | no       | the stored array                   | the stored number |
| `srgb-float`   | yes      | `srgb(...)` / `srgba(...)`         | encoded float     |
| `srgb-hex`     | yes      | `#RRGGBB` / `#RRGGBBAA`            | encoded hex pair  |

1. The three color readings apply to float vectors only and error on
   every other kind. `plain` applies to every kind: the stored value as
   it is, arrays and text included.
2. A color reading is the color assertion. A custom `vec-3-float` under
   `value srgb-hex` renders hex; under the defaults it renders numbers.
   This is what replaces `--type`, per selector instead of per command.
3. Alpha never transfer-encodes: `srgb-hex` quantizes it raw, and
   `srgb-float` passes it through.
4. The two sRGB readings require each spelled component in `[0, 1]` and
   error outside it, never clamp: the transfer is defined on `[0, 1]`,
   and a byte cannot spell `2`.
5. `srgb-hex` spells a component as its two-digit hex pair, the same
   pair the whole-vector spelling carries: `tint.b` under `srgb-hex` is
   `89`, never the decimal `137`. The reading says hex, so nothing
   quietly respells as decimal.
6. The swatch always shows the color's sRGB appearance, whatever
   reading spells the text.

The examples fix the palette field and read one custom `tint` property
bound to a `vec-4-float` value pool holding `[1, 0, 0.25, 0.5]`; each
row is the selector's last three fields as typed.

| Property, presentation, reading | Output                       |
| ------------------------------- | ---------------------------- |
| `tint value auto`               | `[1,0,0.25,0.5]`             |
| `tint value srgb-hex`           | `#FF008980`                  |
| `tint value srgb-float`         | `srgba(1, 0, 0.537099, 0.5)` |
| `tint value linear-float`       | `lin_srgba(1, 0, 0.25, 0.5)` |
| `tint swatch-value srgb-hex`    | the swatch, then `#FF008980` |
| `tint.b value srgb-hex`         | `89`                         |
| `tint.z value auto`             | `0.25`                       |

The first row is the custom-key default, `plain`. The next three spell
one color: the sRGB readings encode the stored `0.25` to `0.537099`,
byte `0x89`, and the linear reading keeps the stored numbers. The last
two address the same component through either alias set: the hex
reading spells its pair, and the default reads the stored float.

## The `auto` defaults

The `auto` reading resolves per key from the glTF vocabulary:

1. A vocabulary color name reads `srgb-hex`. A pool holding any
   component outside `[0, 1]` reads `linear-float` instead, so an HDR
   emissive stays exact; only `auto` falls back, an explicit reading
   errors per rule 4.
2. Everything else reads `plain`: the vocabulary scalars, every custom
   key, and every non-vector kind.

Bare `vxl palette show` renders every idiomatic property exactly as
today; the bare selector renders whole properties only. An explicitly
selected component respells under `srgb-hex`, its hex pair where
today prints a decimal byte. A custom vector's components become
readable without any assertion, and its color rendering moves from
`--type color` to a reading on its own selector.

## What deletes

The `--type` flag, `PaletteShowType`, and the type parameter on
`Dependencies::palette_show`. The name-keyed classification survives
only as the `auto` defaults above.
