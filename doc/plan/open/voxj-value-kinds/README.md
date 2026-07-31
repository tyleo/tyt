# voxj value kinds

Status: **design settled, unplanned.** One rule for the voxel-json
[value pool kinds](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#value-pool-kinds):
plain bounded vectors replace the color vocabulary, and the format gains a
home for non-color data. This page is the design. The ordered steps are not
written yet.

## The decision

**The file stores one form of every value.**

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

Once the spelling and the transfer are gone, a color is a bounded array of
numbers. That is what glTF's own schema calls it:

```jsonc
// glTF's schema for baseColorFactor
{
  "type": "array",
  "items": { "type": "number", "minimum": 0, "maximum": 1 },
  "minItems": 4,
  "maxItems": 4,
}
```

glTF has no color type. Neither does voxj.

## The vocabulary

Every color in voxj is linear light with sRGB primaries and the D65 white
point. The format states that once. No kind repeats it.

| Kind          | JSON                     | Record       |
| ------------- | ------------------------ | ------------ |
| `json`        | any JSON, including null | -            |
| `string`      | string                   | -            |
| `bool`        | boolean                  | -            |
| `int`         | number                   | `min`, `max` |
| `float`       | number                   | `min`, `max` |
| `vec-2-int`   | number[2]                | `min`, `max` |
| `vec-3-int`   | number[3]                | `min`, `max` |
| `vec-4-int`   | number[4]                | `min`, `max` |
| `vec-2-float` | number[2]                | `min`, `max` |
| `vec-3-float` | number[3]                | `min`, `max` |
| `vec-4-float` | number[4]                | `min`, `max` |

Every kind is one JSON shape with a declared range, and `min`/`max` is the
only record vocabulary. The six color kinds are gone. `srgb-hex` and
`srgba-hex` lose their spelling, `srgb-float` and `srgba-float` lose their
transfer, and all four land on `vec-3-float` / `vec-4-float` alongside
`linear-rgb-float` and `linear-rgba-float`.

```jsonc
// before
{ "kind": "srgba-hex", "values": ["#FF0000FF"] }
{ "kind": "linear-rgb-float", "values": [[2, 0, 0]] }

// after
{ "kind": "vec-4-float", "min": 0, "max": 1, "values": [[1, 0, 0, 1]] }
{ "kind": "vec-3-float", "min": 0, "max": "inf", "values": [[2, 0, 0]] }
```

The vector kinds also hold what no color kind could:

```jsonc
// normals
{ "kind": "vec-3-float", "min": -1, "max": 1,
  "values": [[0, 0, 1], [1, 0, 0]] }

// grid coordinates
{ "kind": "vec-2-int", "min": 0, "max": 15, "values": [[3, 7]] }
```

A scalar is not a one-element vector. `0.5` and `[0.5]` are different JSON,
so `int` and `float` stay distinct from the vector kinds.

## Bounds

`min` and `max` are required on every numeric kind and absent from `json`,
`string`, and `bool`. On a vector kind they apply to each component. A
per-component bound asserts nothing about magnitude: `-1..1` on a
`vec-3-float` does not make it a unit vector.

A bound is a finite number, `"inf"`, or `"-inf"`. The string `"none"` is
retired.

```jsonc
{ "kind": "float", "min": 0, "max": 1, "values": [0, 0.5, 1] }
{ "kind": "float", "min": 0, "max": "inf", "values": [1.5, "inf"] }
{ "kind": "vec-2-int", "min": "-inf", "max": "inf", "values": [[-40, 3]] }
```

The sentinel spelling matters because JSON has no infinity literal.
serde_json writes `f64::INFINITY` as `null`, so without the sentinel an
infinite value silently becomes null on write.

1. A `float` or `vec-*-float` value may be `"inf"` or `"-inf"`, since the float
   domain holds them. This is what makes glTF's `attenuationDistance` writable.
   Its default is `+Infinity`.
2. An `int` or `vec-*-int` value is finite, even when its bounds are infinite.
   An unbounded integer range is meaningful. An infinite integer is not.
3. An `int` value or bound is a JSON integer literal. An integer has one
   spelling, so `3.0` and `3e0` reject.
4. `NaN` rejects everywhere. It has no ordering, so it cannot be
   bounds-checked.
5. `int` values and integer bounds lie in `[-(2^53 - 1), 2^53 - 1]` and
   reject beyond, so a JS consumer cannot silently lose one. The Hilbert
   encoding already caps itself at `2^53` for the same reason.

## What still reads a color

No kind says a value is a color. The property name does, per the
[glTF vocabulary](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#gltf-conventions).
A `vec-3-float` value pool holds colors under one name and normals under
another:

```jsonc
"properties": [
  { "name": "emissiveFactor", "valuePool": 0 }, // read as a color
  { "name": "customNormal", "valuePool": 1 }    // read as numbers
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

Two gaps remain, both narrow:

1. `KHR_materials_ior` permits `0` as a special "does not refract" value
   alongside `>= 1`, and a single `min`/`max` interval cannot express
   `{0} union [1, inf)`. Write `min: 0, max: "inf"` and accept that `(0, 1)`
   validates when glTF would reject it. Confirm the exact schema wording
   before writing the rule.
2. No texture bindings. Any `*Texture` property is a reference into the glTF
   texture array, and voxj has no texture concept because the palette is the
   texture. A glTF material carrying a `baseColorTexture` bakes on the way
   in. Worth stating in the format doc as a scope boundary.

## The Rust shape

A bound lives in its kind's value domain, extended with infinities. `VoxjBound`
is deleted. Its reason to exist was `"none"`, a value no domain can spell. With
`"none"` retired, each domain extends in its own way.

Float kinds extend natively, since `f64` already holds every finite number and
both infinities: `min` and `max` are plain `f64`, with `f64::INFINITY` and
`f64::NEG_INFINITY` for the sentinels. Bounds checks are bare IEEE comparisons.
No value is below `-inf` and none is above `+inf`, so the infinite cases need no
arms of their own:

```rust
// before: every check unwraps the enum
let below = matches!(min, VoxjBound::Number(low) if value < low);

// after: IEEE comparison already knows what an infinite bound means
let below = value < min;
```

The sentinel encoding is one serde module over `f64`, shared by `min`, `max`,
and the float values. It reads a number, `"inf"`, or `"-inf"`. It writes the
sentinel strings for infinities, errors on NaN, and writes an integral number as
a JSON integer so `1` does not round-trip as `1.0`. Bounds and float values
follow the same rules, so one module serves both. `values` stays `Vec<f64>` and
`Vec<[f64; N]>`.

Int kinds have no native infinity, so their extension is an enum:

```rust
/// A bound on an int kind: an integer or an infinity. Variant order gives the
/// derived ordering: NegInf < Finite(n) < Inf.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VoxjIntBound {
    NegInf,
    Finite(i64),
    Inf,
}
```

The bounds check is `min <= Finite(value) && Finite(value) <= max`, exact in
`i64` with no cast through `f64`. The type also absorbs rules the validator used
to carry. A `3.5` bound rejects at parse, because the visitor demands an
integer, so the integer-valued bound check in `check_value_pools` is deleted. An
int value cannot be infinite by construction, since `i64` has no infinity to
hold: the wire strings `"inf"` and `"-inf"` reject as int values while staying
legal as int bounds.

The wire enum stays a plain `#[serde(tag = "kind")]` derive, one variant per
kind with typed value shapes:

```rust
pub enum VoxjValuePool {
    // json, string, and bool keep their shapes
    Float { min: f64, max: f64, values: Vec<f64> },
    Int { min: VoxjIntBound, max: VoxjIntBound, values: Vec<i64> },
    Vec3Float { min: f64, max: f64, values: Vec<[f64; 3]> },
    Vec3Int { min: VoxjIntBound, max: VoxjIntBound, values: Vec<[i64; 3]> },
    // vec-2 and vec-4 follow their vec-3 sibling
}
```

A malformed value, a missing bound, and a stray bound on `json` all still reject
at parse. NaN cannot arrive by parse either: JSON has no NaN literal, and the
serde module admits only the two sentinel strings.

voxcore's `VoxValuePoolKind`, `VoxValuePool`, and `VoxValuePoolValueRef` take
the same treatment. `VoxBound` is deleted the same way, and `VoxIntBound` is
added beside it.

## Compatibility

Breaking, and safely so: every crossing fails loudly in both directions.

1. Old file, new reader: the six color kinds are unrecognized and reject. A
   surviving `float` or `int` value pool rejects too, because `"none"` is no
   longer a legal bound.
2. New file, old reader: `vec-3-float` is an unrecognized kind and rejects,
   and `"inf"` is not a legal bound value.

There is no silent misread in either direction, so `version` stays `1`. The
repo has no external consumers, and the voxj redesign already renamed the
property vocabulary in place at `version: 1`. The recommendation is the same
hard break with no aliases, regenerating the fixtures.

## palette show

`palette show` is the one display surface that infers color from the format, so
the change lands hardest here. Today its `classify` switches on the value pool's
kind, and the color kinds are gone:

```rust
// before: the value pool's kind says color
fn classify(value_pool: &VoxValuePool) -> Kind

// after: the property name says color, per the glTF vocabulary
fn classify(property_name: &str, value_pool: &VoxValuePool) -> Kind
```

The requirement: every idiomatic factor renders exactly as it does today.
`baseColorFactor` and `emissiveFactor` draw color swatches, with hex text and
per-channel reads intact, and the swatch encodes linear to sRGB at display. The
numeric factors keep their grayscale swatches through the untouched `float`
kind. A custom key defaults to plain numbers, since a `vec-3-float` is a color
or a normal and the value pool no longer says which. `--type` already exists to
assert color for it.

## Blast radius

Every path below is confirmed at the keyboard. The list holds only what has
been confirmed so far. Expect the implementation to surface more.

1. `projects/voxel-codecs/voxj/docs/voxel-json-file-format.md`: the kind
   table, the notes, validation rules 9 and 10, the TypeScript schema, the
   examples, the format-wide sentence fixing the color space, and the
   texture scope boundary.
2. `projects/voxel-codecs/voxj/src/`: `voxj_value_pool.rs` carries the six
   color variants to delete and the six vector variants to add. Two files
   arrive, the sentinel serde module and `voxj_int_bound.rs`, and
   `voxj_bound.rs` is deleted. `voxj_file.rs` uses color kinds in its examples.
3. `projects/voxel-codecs/voxj-codec/src/`: `validate_voxj_file.rs`,
   `check_voxj_file.rs`, and `internal/voxj_validation/check_value_pools.rs`
   hold the per-kind value checks, the bound rules, and the `2^53` cap. The
   integer-valued bound check moves from validation into the parse.
4. `projects/utilities/voxcore/src/`: `vox_value_pool_kind.rs`,
   `vox_value_pool.rs`, `vox_value_pool_value_ref.rs`,
   `vox_value_pool_flaw.rs`, `vox_main.rs`. `vox_bound.rs` is deleted and
   `vox_int_bound.rs` is added.
5. `projects/utilities/voxsmith/src/`: `internal/value_pool_color.rs` (see
   [what still reads a color](#what-still-reads-a-color)),
   `internal/voxj/vox_value_pool_from_voxj_value_pool.rs`,
   `internal/voxj/voxj_value_pool_from_vox_value_pool.rs`,
   `convert/voxj/from_voxj_file.rs`, `convert/gltf/from_gltf_bytes.rs`, the
   atlas bake's color reads, and the palette reduction's.
   `convert/voxj/color_format.rs` is deleted.
6. `projects/utilities/vxl/src/`: `utilities/voxj_color_format.rs` deleted
   along with its `--color-format` flag, and `implementation/mesh_object.rs`
   (its channel-kind classification). `implementation/palette_show.rs` reworks
   `classify` to key on the property name, per [palette show](#palette-show).
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
