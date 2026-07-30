# voxj value kinds

Status: **design settled, unplanned.** One rule for the voxel-json
[value pool kinds](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#value-pool-kinds),
replacing the color vocabulary with plain bounded vectors and giving the
format a home for non-color data. This page is the design; the ordered steps
are not written yet.

## The decision

**The file stores one form of every value.**

A color has two representations that differ only in how they are written
(`#FF0000FF` and `[1, 0, 0, 1]`) and two that differ only in whether the
transfer is applied (sRGB-encoded and linear light). The format stores all
four and makes every producer choose. It stores one and converts at the
boundary, which voxsmith already does in every direction.

The form kept is linear light, so the glTF export boundary needs no
conversion: every glTF material factor is linear. What is left of a color
once the spelling and the transfer are gone is a bounded array of numbers,
which is what glTF's own schema calls it. `baseColorFactor` is
`{type: array, items: {type: number, minimum: 0, maximum: 1}, minItems: 4,
maxItems: 4}`. glTF has no color type, and neither does voxj.

## The vocabulary

Every color in voxj is linear light with sRGB primaries and the D65 white
point. The format states that once; no kind repeats it.

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
only record vocabulary. The six color kinds are gone: `srgb-hex` and
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

// newly spellable: non-color data, which no color kind could hold
{ "kind": "vec-3-float", "min": -1, "max": 1,
  "values": [[0, 0, 1], [1, 0, 0]] }
{ "kind": "vec-2-int", "min": 0, "max": 15, "values": [[3, 7]] }
```

A scalar is not a one-element vector. `0.5` and `[0.5]` are different JSON,
so `int` and `float` stay distinct from the vector kinds rather than becoming
their one-component case.

## Bounds

`min` and `max` are required on every numeric kind and absent from `json`,
`string`, and `bool`. They apply per component on the vector kinds, and a
per-component bound asserts nothing about magnitude: `-1..1` on a
`vec-3-float` does not make it a unit vector.

The string `"none"` is retired. A bound is a finite number, `"inf"`, or
`"-inf"`.

1. A `float` or `vec-*-float` value may be `"inf"` or `"-inf"`. This is what
   makes glTF's `attenuationDistance` writable; its default is `+Infinity`.
2. An `int` or `vec-*-int` value is finite. Its bounds may still be `"inf"`
   or `"-inf"`, since an unbounded integer range is meaningful.
3. `NaN` rejects everywhere. It has no ordering, so it cannot be
   bounds-checked.
4. `int` values and integer bounds lie in `[-(2^53 - 1), 2^53 - 1]` and
   reject beyond, so a JS consumer cannot silently lose one. The Hilbert
   encoding already caps itself at `2^53` for the same reason.

JSON has no infinity literal and serde_json serializes `f64::INFINITY` as
`null`, so without the sentinel an infinite value silently becomes null on
write.

## What still reads a color

No kind says a value is a color. The property name does, per the
[glTF vocabulary](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#gltf-conventions).
Three consequences the implementation must carry:

1. `value_pool_color` stops switching on kind. It accepts `vec-3-float` and
   `vec-4-float` and takes color-ness from the caller, which knows the
   property name. A 3-component value takes opaque alpha as it does today.
2. `palette show` keeps swatches for the built-in property names and loses
   inference for a custom key, since a `vec-3-float` is a color or a normal
   and the pool no longer says which. `--type` already exists to assert it.
3. The transfer moves entirely to the boundaries. Importing an 8-bit palette
   decodes to linear, exporting to glTF passes linear through untouched, and
   displaying a swatch encodes. Alpha is never transfer-encoded in any of
   them. That is an implementation invariant, not a format rule, because the
   format no longer has a transfer in it.

## glTF parity

voxj's palette is glTF's texture. A glTF material stores every spatially
varying property as `factor * texture[uv]`; a voxj palette varies it across
the model indexed by the voxel's material sample. voxj stores the product,
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

Every factor in core glTF and in `emissive_strength`, `ior`, `specular`,
`transmission`, `volume`, `clearcoat`, `sheen`, `iridescence`, `anisotropy`,
`dispersion`, `diffuse_transmission`, `unlit`, and the archived
`pbrSpecularGlossiness` maps onto `float`, `bool`, `string`, `json`, or a
3- or 4-component float vector. `alphaMode` needs no enum kind, because a
`string` pool's values already are its closed set.

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

`VoxjBound` keeps its job and changes its variants: `Finite(f64)`, `Inf`,
`NegInf`, with `None` removed. `f64` is exact for every legal `int` bound
because of the `2^53` cap.

Float values need the same sentinel encoding as bounds, since serde_json
writes `null` for a non-finite `f64`. That is a serde module over the values,
not a new value type; `values` stays `Vec<f64>` and `Vec<[f64; N]>`.

The wire enum stays a plain `#[serde(tag = "kind")]` derive, one variant per
kind with typed value shapes, so a malformed value, a missing bound, and a
stray bound on `json` all still reject at parse. voxcore's
`VoxValuePoolKind`, `VoxValuePool`, `VoxValuePoolValueRef`, and `VoxBound`
take the same treatment.

## Compatibility

Breaking, and safely so: every crossing fails loudly in both directions.

1. Old file, new reader: the six color kinds are unrecognized and reject. A
   surviving `float` or `int` pool rejects too, because `"none"` is no longer
   a legal bound.
2. New file, old reader: `vec-3-float` is an unrecognized kind and rejects,
   and `"inf"` is not a legal bound value.

There is no silent misread in either direction, so `version` stays `1`. The
repo has no external consumers and the voxj redesign already renamed the
property vocabulary in place at `version: 1`, so the recommendation is the
same hard break with no aliases, regenerating the fixtures.

## Blast radius

Paths confirmed at the keyboard.

1. `projects/voxel-codecs/voxj/docs/voxel-json-file-format.md`: the kind
   table, the notes, validation rules 9 and 10, the TypeScript schema, the
   examples, the format-wide sentence fixing the color space, and the texture
   scope boundary.
2. `projects/voxel-codecs/voxj/src/voxj_value_pool.rs` and `voxj_bound.rs`:
   the six color variants, the six new vector variants, the bound sentinels.
3. `projects/voxel-codecs/voxj-codec/src/`: `validate_voxj_file.rs` and
   `check_voxj_file.rs`, the per-kind value checks, the bound rules, the
   `2^53` cap.
4. `projects/utilities/voxcore/src/`: `vox_value_pool_kind.rs`,
   `vox_value_pool.rs`, `vox_value_pool_value_ref.rs`,
   `vox_value_pool_flaw.rs`, `vox_bound.rs`, `vox_main.rs`.
5. `projects/utilities/voxsmith/src/`: `internal/value_pool_color.rs` (see
   [what still reads a color](#what-still-reads-a-color)),
   `internal/voxj/vox_value_pool_from_voxj_value_pool.rs`,
   `internal/voxj/voxj_value_pool_from_vox_value_pool.rs`,
   `convert/voxj/from_voxj_file.rs`, the atlas bake's color reads, and the
   palette reduction's. `convert/voxj/color_format.rs` is deleted.
6. `projects/utilities/vxl/src/`: `utilities/voxj_color_format.rs` deleted
   along with its `--color-format` flag,
   `implementation/palette_show.rs` (the swatch classification),
   `implementation/mesh_object.rs` (its channel-kind classification).
7. `projects/voxel-codecs/voxj/Cargo.toml` and `voxj-codec/Cargo.toml`: add
   `float_roundtrip` to `serde_json`. It is on in `vmax-codec` and off in
   both of these. Every color from an 8-bit source becomes `k/255`, which
   serializes at 17 significant digits, and a 1 ULP parse error makes
   load/save stop being byte-identical.
8. Two tests this change requires rather than merely touches:
   1. All 256 values of `k/255` survive u8 to linear to u8 as identity. An
      8-bit palette used to round-trip through voxj by copy and now round-
      trips through the transfer both ways, so exactness depends on encode
      and decode being exact inverses including the piecewise part of the
      sRGB curve near zero.
   2. A float pool with 17-significant-digit values saves and loads
      byte-identical.
9. Test fixtures and any checked-in `.voxj` / `.voxjz` assets.
