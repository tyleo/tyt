# vxl palette edit

Status: **open.** The design was settled 2026-10-01. Nothing is built yet. The
steps live in [checklist.md](checklist.md). Code-level choices are logged in
[implementation-decisions.md](implementation-decisions.md).

## Model

`palette edit` runs a [value language](../../../ref/mesh/value-language.md)
program over a palette and writes the results back into its properties. Each
property enters the program as a swatch array holding one entry per material,
in material order. The program sees no geometry, so it works on the plain and
swatch rungs alone. A palette is edited on its own, outside layer resolution.
Every object sampling the palette sees the edit.

## Rules

1. `--value` fragments and the [profiles](#profiles)' values join into one
   program under the
   [object mesh rules](../../../ref/mesh/value-language.md#programs)
2. A property binds under its name with the type the [kind table](#kinds)
   gives its pool. A json property has no type and stays unbound
3. `--write-property <dst-property> <src-expr>` evaluates `src-expr` at the
   program's end and writes the result to the property. A plain value lands
   on every material. A swatch array lands one entry per material. A voxel,
   face, or corner value errors
4. A property the palette already has keeps its pool. The value's kind has to
   match that pool's kind. A property the palette lacks is added on a new pool
   of the value's kind
5. A written value lands in the property's pool. An equal value already in the
   pool is reused. Otherwise the value is appended. A value no palette draws
   afterward is released. Palettes sharing the pool keep their cells
6. A write checks a vocabulary property's values against its range, as
   `mesh-doc voxelize` does on import. Writing `1.2` to `roughness` errors
7. Two `--write-property` flags for one property error
8. The program is checked and evaluated once per selected palette. A name the
   palette lacks errors, and the error reports which palette.
   `default(name, fallback)` covers a name some palettes lack
9. Every selected palette is evaluated against the unchanged document before
   anything is written. Palette order never changes a result. An error leaves
   every palette untouched

### Kinds

| Value                           | Pool kind                                            |
| ------------------------------- | ---------------------------------------------------- |
| `f64` vec1 to vec4              | `float`, `vec-2-float`, `vec-3-float`, `vec-4-float` |
| `u8`, `u16`, `u32` vec1 to vec4 | `int`, `vec-2-int`, `vec-3-int`, `vec-4-int`         |
| `bool` vec1                     | `bool`                                               |
| `string`                        | `string`                                             |

An int pool reads as `u32`. An int outside the `u32` range errors when the
program reads it. A bool wider than vec1 has no pool kind and errors on write.

## Palette selection

`palette edit`, `palette quantize`, and `palette remap` share one `--index`
rule.

1. `--index` takes a palette index `#`, an inclusive range `a-b`, or `*` for
   every palette. The flag repeats, and the union of its values selects each
   palette once
2. `--index` defaults to `*`
3. An index or range past the palette count errors
4. Each command acts on every selected palette in palette order.
   `palette quantize` reduces each palette to `--max-materials` separately.
   `palette remap` snaps each palette onto the target
5. A command errors on a selected palette it cannot act on, even when `*`
   selected it. `palette quantize` errors on a palette no live voxel samples.
   `palette remap` errors when the selection includes its in-document
   `--target-index` palette. An in-document remap therefore selects its sources
   with explicit `--index` values

## Commands

Each usage follows `vxl <command> <input> [output]`. `<palettes>` takes `#`,
`a-b`, or `*`.

| Command                     | Usage                                                                                                                                                   | Step |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ---- |
| `palette edit`              | `[--index <palettes>]... [--profile <profile>]... [--value <bindings>]... [--values-from <profile>]... [--write-property <dst-property> <src-expr>]...` | S6   |
| `profile palette edit list` | the flags of the other profile lists                                                                                                                    | S6   |
| `palette quantize`          | `[--index <palettes>]...`, every other flag unchanged                                                                                                   | S4   |
| `palette remap`             | `[--index <palettes>]...`, built from the [vxl-commands checklist](../vxl-commands/checklist.md)                                                        | S7   |

## Profiles

Profiles live in `.vxlconfig`'s `palette.edit.profiles` and cascade the way
[object mesh profiles](../../../ref/mesh/profile-language.md#loading) do. No
built-in profiles sit beneath them. `--index` stays on the command line.

```ts
interface PaletteEditProfile {
  /** Printed beside the profile name in the profile listings. */
  description?: string;

  /** Mirrors `--values-from` per entry; properties never travel. */
  valuesFrom?: string[];

  /** Mirrors `--value` per entry. */
  values?: string[];

  /** Mirrors `--write-property`, a property per key. */
  properties?: Record<string, string>;
}
```

Two flags read a profile:

1. `--profile <profile>` applies a whole profile, values and properties. The
   profile's values join the program ahead of every `--value` and
   `--values-from` binding, regardless of where the flag sits. Repeated
   `--profile` flags stack in line order. A property two stacked profiles both
   write errors. A `--write-property` flag replaces the stack's write of that
   property
2. `--values-from <profile>` appends only a profile's values, at the flag's
   position among the `--value` flags. The profile's properties stay behind

`valuesFrom` imports the listed profiles' values, depth first, ahead of the
importing profile's values. Their properties stay behind. A profile's values
land once per run, at the profile's first arrival. A profile reached again
through an import or a second flag adds nothing. An import cycle errors with
the chain in its message. The `.vxlconfig` layers merge into one namespace
before `valuesFrom` resolves. Imports resolve within `palette.edit.profiles`
only.

```jsonc
{
  "palette": {
    "edit": {
      "profiles": {
        "tags": {
          "values": ["rust = tag == \"rust\"", "chrome = tag == \"chrome\""],
        },

        "weathered": {
          "valuesFrom": ["tags"],
          "properties": {
            "roughness": "mix(roughness, 0.9, rust)",
          },
        },

        "polished": {
          "valuesFrom": ["tags"],
          "properties": {
            "metallic": "mix(metallic, 1.0, chrome)",
          },
        },
      },
    },
  },
}
```

```sh
# Stacks both profiles. tags lands once. Each profile writes a different property.
vxl palette edit robot.voxj --profile weathered --profile polished
```

## Example

```sh
# Gives rusty materials roughness 0.9 in every palette. Each palette needs tag and roughness.
vxl palette edit robot.voxj \
  --value 'rust = tag == "rust"' \
  --write-property roughness 'mix(roughness, 0.9, rust)'

# Dims palette 2's colors in Oklab and keeps their alpha.
vxl palette edit robot.voxj --index 2 \
  --value 'dim = rgbFromOklab(oklabFromRgb(baseColor.rgb) * rgb(0.8, 1, 1))' \
  --write-property baseColor 'rgba(dim.r, dim.g, dim.b, baseColor.a)'

# Adds a custom wear property to palettes 0 through 3.
vxl palette edit robot.voxj --index 0-3 \
  --write-property wear 'mix(0.0, 1.0, tag == "rust")'
```

## Language changes

`vox-value-language` and `object mesh` change with the command.

1. The float type becomes `f64`. The type name, the `2f64` suffix, and the
   `f64(e)` conversion all use that spelling. `f64(e)` converts an unsigned
   value exactly
2. A glTF material factor or float vertex attribute holds `f32`, so a value
   written there rounds to the nearest `f32`. A value past the `f32` range
   errors
3. `--write-file-json-value` and the JSON extras write `f64`
4. `Groupings` carries the swatch count because a palette edit has no voxels
   to derive it from. A voxel whose swatch id is at or past the count errors
5. A run binds a property only when the program or a write expression mentions
   it. A negative int or an infinite float in an unread property never blocks
   the run. `object mesh` binds the same way

## voxcore

`VoxMain` gains two setters.

1. `retain_value_pool_value(value_pool_id, value)` appends a value and returns
   its id. The setter errors on a kind mismatch or an out-of-domain value.
   `value` has the new type `VoxValuePoolValue`, which mirrors
   `VoxValuePoolValueRef` as an owned value
2. `set_material_value(palette_id, material_id, property_id, value_id)` points
   one material's cell for one property at `value_id`. The setter errors on an
   unknown id or a value outside the property's pool

Neither setter fires a hook, matching `retain_property` and
`repoint_value_pool_value`. Adding a property uses the existing
`retain_value_pool` and `retain_property`.

## Design notes

1. The language is reused whole. A palette edit uses only the swatch rung of
   the ladder. Bindings only compute, and every write comes from a flag. The
   mesh writers follow the same rule.
2. `--index` sits on the command. A mesh's per-flag material index selects a
   destination fed by one environment. A palette supplies the program's
   environment. Two palettes' arrays differ in length, so one evaluation can
   hold only one palette.
3. `--write-property` also adds a missing property. A misspelled name or a
   wrong `--index` therefore adds a property silently. On an overlay palette,
   the added property overrides the layers below. The single flag was chosen
   for simplicity anyway.
4. The float type widens to `f64` so palette values round-trip exactly. Each
   f32 destination rounds at its edge because its format holds `f32` alone.
5. `*` selects every palette with no filtering. An unsampled palette under
   `palette quantize` points at a document problem. Emptying the palette would
   hide the problem.
