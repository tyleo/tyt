# voxj value kinds

Status: **closed** (2026-08-05). The spec landed the shape-only kind
vocabulary in one commit, the `Factor` rename and the kind rework landed
crate by crate in dependency order (voxj, voxj-codec, voxcore, voxsmith,
vxl), and the palette show selector landed on the approved four-field
design. The closeout regenerated the one on-disk asset, swept the open
vxl-commands pages, and read both gate greps clean; the workspace builds,
lints, and tests green. The executable steps lived in
[checklist.md](checklist.md), the code-level rulings in
[reference/implementation-decisions.md](reference/implementation-decisions.md),
and the grep residue in
[reference/survey.md](reference/survey.md#closeout-grep-residue).

One rule for the voxel-json
[value pool kinds](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#value-pool-kinds):
a kind is a JSON shape and nothing more. Plain vectors replace the color
vocabulary, range checks move out to the tools, and the format gains a home
for non-color data.

## The decision

**The file stores one form of every value, and a kind is only a shape.**

Today the format has three kinds that all hold the same opaque red:

```jsonc
{ "kind": "srgb-hex", "values": ["#FF0000"] }
{ "kind": "srgb-float", "values": [[1, 0, 0]] }
{ "kind": "linear-rgb-float", "values": [[1, 0, 0]] }
```

The first two differ only in spelling. The third differs only in whether the
sRGB transfer is applied. Each has an alpha-carrying sibling, so six kinds
carry one concept, and every producer must pick among them. voxsmith already
converts in every direction.

The change: the file stores one form, and the boundaries convert. The form
kept is linear light. Every glTF material factor is linear, so the glTF
export boundary needs no conversion.

Once the spelling and the transfer are gone, a color is an array of numbers.
That is all glTF's own schema has to say about it:

```jsonc
// glTF's schema for baseColorFactor
{
  "type": "array",
  "items": { "type": "number", "minimum": 0, "maximum": 1 },
  "minItems": 4,
  "maxItems": 4,
}
```

glTF has no color type. Neither does voxj. The shape becomes the kind, and
the range stays with the property (see
[where ranges live](#where-ranges-live)).

## The vocabulary

Every color in voxj is linear light with sRGB primaries and the D65 white
point. The format states that once. No kind repeats it.

| Kind          | JSON                     | Rust        |
| ------------- | ------------------------ | ----------- |
| `json`        | any JSON, including null | `VoxjValue` |
| `string`      | string                   | `String`    |
| `bool`        | boolean                  | `bool`      |
| `int`         | number                   | `i64`       |
| `float`       | number                   | `f64`       |
| `vec-2-int`   | number[2]                | `[i64; 2]`  |
| `vec-3-int`   | number[3]                | `[i64; 3]`  |
| `vec-4-int`   | number[4]                | `[i64; 4]`  |
| `vec-2-float` | number[2]                | `[f64; 2]`  |
| `vec-3-float` | number[3]                | `[f64; 3]`  |
| `vec-4-float` | number[4]                | `[f64; 4]`  |

Every kind is one JSON shape, and a value pool is its `kind` and its
`values`. Nothing else rides it. The Rust column is one value; the kind's
variant holds a `Vec` of it (see [the Rust shape](#the-rust-shape)). The
six color kinds are gone: `srgb-hex` and `srgba-hex` lose their spelling,
`srgb-float` and `srgba-float` lose their transfer, and all four land on
`vec-3-float` / `vec-4-float` alongside `linear-rgb-float` and
`linear-rgba-float`.

```jsonc
// before
{ "kind": "srgba-hex", "values": ["#FF0000FF"] }
{ "kind": "linear-rgb-float", "values": [[2, 0, 0]] }

// after
{ "kind": "vec-4-float", "values": [[1, 0, 0, 1]] }
{ "kind": "vec-3-float", "values": [[2, 0, 0]] }
```

The vector kinds also hold what no color kind could:

```jsonc
// normals
{ "kind": "vec-3-float", "values": [[0, 0, 1], [1, 0, 0]] }

// grid coordinates
{ "kind": "vec-2-int", "values": [[3, 7]] }
```

A scalar is not a one-element vector. `0.5` and `[0.5]` are different JSON,
so `int` and `float` stay distinct from the vector kinds.

## Where ranges live

No kind carries `min`/`max`. The format checks every value's shape and never
its range.

The format already draws that line. Structure is a hard contract: kinds,
encodings, indices, counts. Property meaning is convention: names are
advisory, and a consumer ignores one it does not recognize. A range sits on
the convention side: `0..1` is a fact about `metallic`, not a fact about
the value pool it binds.

A `min`/`max` on the value pool cannot carry that fact, for three reasons:

1. Nobody vouches for it. A bound restates the property's range per file,
   and nothing checks the restatement, so the validator would enforce
   values against a claim with no authority behind it.
2. It answers to no one property. The format's own example binds one
   `float` value pool to both `metallic` and `roughness`.
3. An interval is too weak. `KHR_materials_ior` permits `0` for "does not
   refract" alongside `>= 1`, and no `min`/`max` spells
   `{0} union [1, inf)`.

So the ranges live in voxsmith, which already owns the glTF conventions,
and they live once: one vocabulary check walks every bound property and
errors on any value outside that property's range. Code spells each range
exactly, `ior`'s union included. Confirm the exact schema wording before
writing it. Every boundary calls that one function instead of growing its
own:

1. The glTF export runs it before writing, so nothing out of range reaches
   a `.glb`.
2. The glTF import can run it on what it read, so a bad source file errors
   at entry instead of at the next export.
3. An 8-bit palette import never trips it: a component cannot decode
   outside `[0, 1]`.
4. `palette show` needs classification, not ranges: it keys on the property
   name, with `--type` to assert what a name alone cannot (see
   [palette show](#palette-show)).

Loading a voxj file calls none of this. A load checks shape and structure,
so an out-of-range value loads without complaint and fails at the first
boundary that reads it:

```jsonc
// loads: the shape is right, and shape is all a load checks
{ "kind": "float", "values": [7] }

// fails in the glTF export, where a palette binds it to metallic:
// 7 is outside [0, 1]
```

A custom property has no checkable range at all: the format does not
understand its meaning, and only its producer knows the intended domain.
That check belongs in the producer's tooling, next to whatever writes the
value.

The late failure is the trade, and it is a scope statement, not a hole:
shape is the format's contract, and ranges ride the property vocabulary. If
it ever hurts, the cure is one more call site for the same vocabulary
check, a lint command, not bounds in the format.

## The value domains

Dropping bounds does not loosen the domains. Every numeric kind keeps its
values exact:

1. A `float` or `vec-*-float` value is a finite number, `"inf"`, or
   `"-inf"`. JSON has no infinity literal and serde_json writes
   `f64::INFINITY` as `null`, so without the sentinels an infinite value
   silently becomes null on write. They are what make glTF's
   `attenuationDistance`, default `+Infinity`, writable.
2. An `int` or `vec-*-int` value is finite. An infinite integer means
   nothing, so `"inf"` and `"-inf"` reject as int values.
3. An `int` value is a JSON integer literal. An integer has one spelling, so
   `3.0` and `3e0` reject.
4. `NaN` rejects everywhere. JSON cannot spell it, and the write side errors
   instead of inventing a spelling.
5. `int` values lie in `[-(2^53 - 1), 2^53 - 1]` and reject beyond, so a JS
   consumer cannot silently lose one. The Hilbert encoding already caps
   itself at `2^53` for the same reason.

```jsonc
{ "kind": "float", "values": [0, 0.5, "inf"] } // fine
{ "kind": "int", "values": [3.0] }             // rejects: 3.0 is not 3
{ "kind": "int", "values": ["inf"] }           // rejects: no infinite int
```

## What still reads a color

No kind says a value is a color. The property name does, per the
[glTF vocabulary](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#gltf-conventions).
A `vec-3-float` value pool holds colors under one name and normals under
another:

```jsonc
"properties": [
  { "name": "emissiveColor", "valuePool": 0 }, // read as a color
  { "name": "customNormal", "valuePool": 1 }   // read as numbers
]
```

Three consequences the implementation must carry:

1. `value_pool_color` stops switching on kind. It accepts `vec-3-float` and
   `vec-4-float` and takes color-ness from the caller, which knows the
   property name. A 3-component value takes opaque alpha as it does today.
2. `palette show` classifies by property name instead of kind (see
   [palette show](#palette-show)).
3. The transfer moves entirely to the boundaries. Importing an 8-bit palette
   decodes to linear. Exporting to glTF passes linear through untouched.
   Displaying a swatch encodes. Alpha is never transfer-encoded in any of
   them. That is an implementation invariant, not a format rule, because the
   format no longer has a transfer in it.

## glTF parity

voxj's palette is glTF's texture. A glTF material stores every spatially
varying property as `factor * texture[uv]`. A voxj palette varies it across
the model, indexed by the voxel's material sample. voxj stores the product,
not the factorization, so conversion is directional and the rendered result
is unchanged.

| glTF                       | voxj                                     |
| -------------------------- | ---------------------------------------- |
| material                   | palette row                              |
| material factor            | palette property + value pool cell       |
| texture                    | the palette itself                       |
| UV coordinate              | the voxel's material index               |
| texture image              | atlas baked from the palette at export   |
| POSITION / NORMAL accessor | voxel positions, normals from the mesher |

Every factor in core glTF maps onto `float`, `bool`, `string`, `json`, or a
3- or 4-component float vector. So does every factor in the material
extensions: `emissive_strength`, `ior`, `specular`, `transmission`,
`volume`, `clearcoat`, `sheen`, `iridescence`, `anisotropy`, `dispersion`,
`diffuse_transmission`, `unlit`, and the archived `pbrSpecularGlossiness`.
`alphaMode` needs no enum kind, because a `string` value pool's values
already are its closed set.

`KHR_materials_ior` is the one factor whose range is not an interval. The
split never touches the file; the vocabulary check spells it exactly (see
[where ranges live](#where-ranges-live)).

One gap remains, and it is narrow: no texture bindings. Any `*Texture`
property is a reference into the glTF texture array, and voxj has no
texture concept because the palette is the texture. A glTF material
carrying a `baseColorTexture` bakes on the way in. Worth stating in the
format doc as a scope boundary.

## The property names

The properties whose glTF field carries a `Factor` suffix drop it in voxj:

| glTF field           | voxj property   |
| -------------------- | --------------- |
| `baseColorFactor`    | `baseColor`     |
| `metallicFactor`     | `metallic`      |
| `roughnessFactor`    | `roughness`     |
| `emissiveFactor`     | `emissiveColor` |
| `transmissionFactor` | `transmission`  |

`occlusionStrength`, `emissiveStrength`, and `ior` already name the thing
itself and do not change.

In glTF, `Factor` names one term of `factor * texture[uv]`. voxj stores the
product, so the suffix asserts a factorization the format does not have. It
does not even buy field identity at the boundary: the export bakes the
palette into an atlas, so a value authored under `baseColorFactor` would
land in `baseColorTexture`'s texels while the exported `baseColorFactor`
stays `1`.

The vocabulary already names one resolved value: `occlusionStrength` is
glTF's `occlusionTexture.strength` flattened to what it means. The rename
generalizes that convention. Nothing is invented either: base color,
metallic, and roughness are the PBR parameter names, and `metallicFactor`
is glTF's serialization field for one of them. The conventions table keeps
one glTF citation per row, so the by-reference lookup survives the
spelling.

Alone, this rename is the one crossing in the redesign that would fail
silently: unknown property names are ignored by design, so an old file's
`baseColorFactor` would quietly no-op and render defaults. It rides the
same hard break as the kinds instead: a realistic old file already rejects
on its color kinds or its `min`/`max` keys, and the fixtures regenerate
once for everything.

## The Rust shape

`VoxjValuePool` is an enum on `Vec`: one variant per kind, the `Vec` as its
whole payload. No variant carries a `values` field. The adjacently tagged
derive spells that key once for all of them, and `deny_unknown_fields`
keeps the wire closed. The vector kinds spell their own names, because
kebab-case breaks before a capital and never before a digit:

```rust
#[serde(
    tag = "kind",
    content = "values",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
pub enum VoxjValuePool {
    // json, string, and bool keep their shapes
    Int(Vec<i64>),
    Float(Vec<f64>),
    #[serde(rename = "vec-3-int")]
    Vec3Int(Vec<[i64; 3]>),
    #[serde(rename = "vec-3-float")]
    Vec3Float(Vec<[f64; 3]>),
    // vec-2 and vec-4 follow their vec-3 sibling
}
```

`VoxjBound` is deleted, and no bound type replaces it. There is nothing
left to spell.

The float variants are not serde-bare. One serde module over `f64` serves
every float payload, with array forms for the `[f64; N]` shapes. It
reads a number, `"inf"`, or `"-inf"`; it writes the sentinel strings for
infinities, errors on NaN, and writes an integral number as a JSON integer
so `1` does not round-trip as `1.0`.

The int visitor says what `i64` alone cannot: a value outside
`[-(2^53 - 1), 2^53 - 1]` rejects, and so does a fractional or exponent
spelling, so `3.0` fails in the parse rather than in a validation rule.
`values` non-empty stays a validation rule, since a `Vec` cannot say it.

A stray `min` on any kind rejects in the parse itself, because the derive
denies unknown keys, so validation keeps no bounds arm at all. NaN cannot
arrive by parse either: JSON has no NaN literal, and the serde module
admits only the two sentinel strings.

voxcore's `VoxValuePoolKind`, `VoxValuePool`, and `VoxValuePoolValueRef`
take the same treatment, and `VoxBound` is deleted the same way.

## Compatibility

Breaking, and safely so: every crossing fails loudly in both directions.

1. Old file, new reader: the six color kinds are unrecognized and reject. A
   surviving `float` or `int` value pool rejects too, because its `min` and
   `max` are unknown keys.
2. New file, old reader: `vec-3-float` is an unrecognized kind and rejects,
   and a `float` or `int` value pool arrives without the `min`/`max` an old
   reader requires.

There is no silent misread in either direction, so `version` stays `1`. The
repo has no external consumers, and the voxj redesign already renamed the
property vocabulary in place at `version: 1`. The recommendation is the same
hard break with no aliases, regenerating the fixtures. The property rename
rides inside it; alone it would fail silently (see
[the property names](#the-property-names)).

## palette show

`palette show` is the one display surface that infers color from the format,
so the change lands hardest here. Today its `classify` switches on the value
pool's kind, and the color kinds are gone:

```rust
// before: the value pool's kind says color
fn classify(value_pool: &VoxValuePool) -> Kind

// after: the property name says color, per the glTF vocabulary
fn classify(property_name: &str, value_pool: &VoxValuePool) -> Kind
```

The requirement: every idiomatic property renders exactly as it does today.
`baseColor` and `emissiveColor` draw color swatches, with hex text and
per-channel reads intact, and the swatch encodes linear to sRGB at display.
The numeric properties keep their grayscale swatches through the `float`
kind. A custom key defaults to plain numbers, since a `vec-3-float` is a
color or a normal and the value pool no longer says which. `--type` already
exists to assert color for it.

## Blast radius

Every path below is confirmed at the keyboard. The list holds only what has
been confirmed so far. Expect the implementation to surface more.

1. `projects/voxel-codecs/voxj/docs/voxel-json-file-format.md`: the kind
   table, the notes, validation rules 9 and 10, the TypeScript schema, the
   examples, the property names in the glTF conventions table, the
   format-wide sentence fixing the color space, the sentence scoping ranges
   to the property vocabulary, and the texture scope boundary.
2. `projects/voxel-codecs/voxj/src/`: `voxj_value_pool.rs` carries the six
   color variants to delete, the six vector variants to add, and the
   `min`/`max` fields to drop. One file arrives, the sentinel serde module,
   and `voxj_bound.rs` is deleted. `voxj_file.rs` uses color kinds in its
   examples.
3. `projects/voxel-codecs/voxj-codec/src/`: `validate_voxj_file.rs`,
   `check_voxj_file.rs`, and `internal/voxj_validation/check_value_pools.rs`
   hold the per-kind value checks and the bound rules. The bound rules
   delete, the per-kind checks shrink to shape checks, and the int rules,
   one spelling and the `2^53` cap, move from validation into the parse.
4. `projects/utilities/voxcore/src/`: `vox_value_pool_kind.rs`,
   `vox_value_pool.rs`, `vox_value_pool_value_ref.rs`,
   `vox_value_pool_flaw.rs`, `vox_main.rs`. `vox_bound.rs` is deleted.
5. `projects/utilities/voxsmith/src/`: `internal/value_pool_color.rs` (see
   [what still reads a color](#what-still-reads-a-color)),
   `internal/voxj/vox_value_pool_from_voxj_value_pool.rs`,
   `internal/voxj/voxj_value_pool_from_vox_value_pool.rs`,
   `convert/voxj/from_voxj_file.rs`, `convert/gltf/from_gltf_bytes.rs`, the
   atlas bake's color reads, and the palette reduction's.
   `convert/voxj/color_format.rs` is deleted. One file also arrives: the
   vocabulary check from [where ranges live](#where-ranges-live), the
   single function the boundaries call; its exact home is not confirmed
   yet.
6. `projects/utilities/vxl/src/`: `utilities/voxj_color_format.rs` deleted
   along with its `--color-format` flag, and `implementation/mesh_object.rs`
   (its channel-kind classification). `implementation/palette_show.rs`
   reworks `classify` to key on the property name, per
   [palette show](#palette-show).
7. `projects/voxel-codecs/voxj/Cargo.toml` and `voxj-codec/Cargo.toml`: add
   `float_roundtrip` to `serde_json`. It is on in `vmax-codec` and off in
   both of these. An 8-bit color component decodes to a linear value that
   serializes at 17 significant digits, and without the feature a 1 ULP
   parse error makes load/save stop being byte-identical.
8. Two new tests the change requires:
   1. All 256 values of `k/255` survive u8 to linear to u8 as identity. An
      8-bit palette used to round-trip through voxj by copy. Now it goes
      through the transfer both ways, so exactness depends on encode and
      decode being exact inverses, including the piecewise part of the sRGB
      curve near zero.
   2. A `float` value pool holding 17-significant-digit values saves and
      loads byte-identical.
9. Test fixtures and any checked-in `.voxj` / `.voxjz` assets. The one
   checked-in `.voxj` lives in the `tyt-assets` submodule, so regenerating
   it is a submodule commit.
