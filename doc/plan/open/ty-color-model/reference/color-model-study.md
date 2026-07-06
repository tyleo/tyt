# Color-model design study

_Part of the [palette-style color model plan](../README.md)._

A design study (2026-07-06), run as a multi-agent workflow: a usage census across
`ty-math`, `ty-math-serde`, `voxcore`, `voxsmith`, and `vxl`, prior-art research on
five sources, a synthesis of options, and a three-lens adversarial critique. This
is the durable record; the decision and plan are in the [README](../README.md).

## Current model (census)

Six public color families in `ty-math`: `TySrgbaColor` (8-bit sRGB storage),
`TyRgbaColor<T = f32>` (float RGBA), `TyLinearRgbaColor<T = f32>` (linear float),
`TyHsvaColor`, `TyOklabColor`, `TyCielabColor`; `ty-math-serde` adds
`TyRgbaColorSerde`.

- **`TySrgbaColor`** (unambiguous): gamma-encoded 8-bit. Owns `from_hex`/`to_hex`,
  `to_rgba` (normalize only), `to_vector3`, `to_linear_rgba` (real sRGB decode).
- **`TyLinearRgbaColorF64`** (unambiguous): the linear-light hub; the only type
  with `to_srgba` (linear -> sRGB encode), `to_oklab`, `to_cielab`,
  `componentwise_multiply`.
- **`TyRgbaColor<T>`** (the problem): documented only as "component in `[0, 1]`",
  but `to_srgba` quantizes with no transfer ("already sRGB-encoded") and `to_hsva`
  runs plain HSV. De facto the float twin of `TySrgbaColor`, yet also used as a
  raw `[0, 1]` bag.
- `TyOklab` / `TyCielab`: inert one-way sinks. `TyHsvaColor`: only inbound edge is
  `TyRgbaColor::to_hsva`, only test loops back through it.

Driving fact: **no call site stores a `TyRgbaColorF64`.** sRGB uses are
`new(...).to_srgba()` quantize transients (palette_show 450/454, pool_color 14/17,
bake_atlas 138/143, reduce_palette 175/178) or `from_array(b).to_rgba().to_array()`
normalizes into voxcore pools (goxl 86, mvox 106, vmax 270, voxelize_mesh 346). The
only resident/serialized use is the raw fbx path (`create_point_cloud.rs` ->
`tyt-injection` JSON via `TyRgbaColorSerde`) and the raw channel reads in
`sample_material.rs:294` (`.b`/`.g` metallic/roughness) and `:316` (`.r`
occlusion). `voxcore` stores colors as raw `[f64; N]` in named `VoxValuePool::{Srgb,
Srgba, LinearRgb, LinearRgba}` variants, no `ty-math` type.

## Options weighed

1. **Rename `TyRgbaColor` to an explicit sRGB-float sibling** (`TyEncodedSrgba*`).
   Keeps a standing sRGB-float type the census shows is never stored; still forces
   the raw-data reroute; the natural name `TySrgbaColor` is taken. Medium-large.
2. **Unify into a generic `TySrgba<T>`** (palette-style). One sRGB space, storage
   as a generic axis. Largest churn; needs manual `Eq`/`Hash` on the hash-key
   type; buys u8<->float parity the code never exercises. **CHOSEN by the owner.**
3. **Demote `TyRgbaColor` to a documented neutral raw `[0, 1]` container**; express
   the sRGB quantize as `TySrgbaColor::from_unorm` (a boundary constructor, since
   the sRGB float is never stored). Smallest blast radius (~7 files), zero wire
   impact. **The study's recommendation** (not chosen).
4. **Remove `TyRgbaColor` entirely**; sRGB-float becomes conversions on
   `TySrgbaColor`, raw data -> vectors. Breaks the fbx JSON wire (no
   `TyVector4Serde`; keys would flip to `x/y/z/w`) unless a color-named serde is
   re-added.

## Prior art

- **`palette`**: `Rgb<S = Srgb, T = f32>`. Space `S` (an `RgbStandard` bundling
  primaries + transfer) is a parameter; component `T` separate. Aliases `Srgb`,
  `LinSrgb`, `Srgba`, `LinSrgba`. Encoding is the type identity.
- **`bevy_color`**: one struct per space (`Srgba`, `LinearRgba`, `Hsla`,
  `Oklaba`, ...), all `f32`, no u8 type (u8 via `to_u8_array`), no neutral `Rgba`;
  a `Color` enum wraps them for storage.
- **`egui` `ecolor`**: `Color32` (sRGB, 8-bit, premultiplied) vs `Rgba`
  (**linear** f32, premultiplied). Reserves the bare `Rgba` name for linear.
- **`kolor` / `colstodian` / glTF**: color space is data/tag; raw data channels
  (metallic/roughness, normal maps) are "Non-Color Data", never sRGB-decoded, and
  are not colors.
- **Naming**: the dominant idiom tags the space (palette/cint) or names one struct
  per space (bevy). A bare neutral RGB is rare (only the `rgb` crate) and only safe
  when explicitly documented untagged.

## Critique (must-fixes folded into the plan)

Two of three lenses endorsed the study's Option 3; all three raised points that
apply to the chosen Option 2 as well:

- **Name construction to reveal the sRGB precondition.** A neutral name like
  `from_unorm` ("unsigned normalized", a storage term) hides that the input must
  already be sRGB-encoded, re-creating the "neutral name, sRGB in the comment"
  smell on a constructor. Under the generic, the equivalent is the u8<->f64
  component conversion (`into_format` / `to_u8` / `to_f64`) plus the transfer
  conversions `TySrgba<f64>::to_lin_srgba` / `TyLinSrgba<f64>::to_srgba`; keep the
  transfer explicit and off the plain component cast.
- **Do not half-remove HSV.** `to_hsva` has an internal test consumer
  (`ty_hsva_color.rs:125`), so deleting it without editing that file fails to
  compile, and leaving `TyHsvaColor` while dropping its only inbound edge orphans
  three files. Delete the family or keep `to_hsva`; do not split the difference.
- **The wire is not fully "never stored".** sRGB floats ARE persisted as raw
  `[f64; N]` in voxcore `Srgba` / voxj `SrgbaFloat` pools; only the `ty-math`
  wrapper is transient. The generic must not regress those raw pool arrays (it does
  not: voxcore is untouched).

## Open questions

1. Are the fbx per-point colors sRGB, linear, or explicitly untagged? Base-color
   PNGs are usually sRGB, but `create_point_cloud` dumps `pixel/255` with no
   transfer. This gates the fbx serde sub-step.
2. Should voxcore `Srgb`/`Srgba` pools stay sRGB-encoded floats forever? If pool
   ingestion should decode to linear instead, that is a separate, larger decision.
3. Keep or drop the HSV family (see the critique).

## Correction (owner review, 2026-07-06)

The study characterized the fbx per-point colors as "raw data" and open question 1
asked whether to move them to a vector. On review that is wrong: `sample_texture`
(`create_point_cloud.rs:402`) returns `pixel / 255` base-color texels as
`TyRgbaColor::new(r, g, b, a)` -- these are genuine per-point **vertex colors**
(sRGB float), not raw data. They stay a **color type** (`TySrgba<f32>`) with the
serde renamed and the `r`/`g`/`b`/`a` keys kept; no `TyVector4Serde`, no wire
change. A color is not a vector (PLY/LAS/USD name their color channels; only glTF's
binary `COLOR_0` accessor is positional). This makes the fbx colors the one
resident `TySrgba<f32>`, so the generic's `u8` and `f32` variants each have a real
consumer. Only the metallic/roughness/occlusion data channels are the genuine
"Non-Color Data" that leaves the color namespace.
