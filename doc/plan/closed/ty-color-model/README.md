# Palette-style color model for ty-math

Status: **closed.** All nine steps (S1-S9) landed; the color types are on the
palette-style model (`TySrgba<T>` / `TyLinSrgba<T>` / `TySrgb<T>`, non-color
channels off the color namespace) and the old types are removed. A follow-up
from the [ty-math adoption
plan](../../closed/ty-math-adoption/README.md), which surfaced that `ty-math`'s RGBA color
types are ambiguous: `TySrgbaColor` (8-bit) and `TyRgbaColor<T>` (float) are the
same color space (sRGB) under unrelated names, and `TyRgbaColor` is documented as
a neutral `[0, 1]` value yet its methods (`to_srgba`, `to_hsva`, the reverted C8
`to_linear_rgba`) assume sRGB. This plan restructures the color types on the
`palette` crate's model: the color space is the type identity, the component type
is a separate generic parameter.

## The model in one paragraph

Every color has a color space: here, sRGB (gamma-encoded: 8-bit texels, hex,
palette bytes, fbx vertex colors) or linear (material factors, lighting math).
There is no "unknown-space color" -- we never hold a color and not know its space.
The `[0, 1]` channels that feel space-less -- metallic, roughness, occlusion -- are
NOT colors; they are data stored in image channels (glTF "Non-Color Data"), so they
are plain numbers / vectors, never a color type, in either direction. `TyRgbaColor`
read as "neutral" only because it quietly served two roles at once: an sRGB float
color (via `to_srgba`, and the fbx vertex colors) and a raw data bag (the
metallic/roughness reads). Splitting those -- colors are spaced, data is numbers --
is the entire fix. This plan then makes the sRGB color a component-generic
`TySrgba<T>` (u8/f32/f64), matching how every other `ty-math` type is a generic with
`F32`/`F64` aliases, so color stops being the odd one out.

## Decision

**Adopt the full palette-style generic (owner's choice, 2026-07-06).** A design
study weighed four options (see [Background](#background)); the owner chose the
generic over the study's lighter recommendation, accepting the larger churn for
the textbook-correct shape.

- **`TySrgba<T = f32>`** replaces both `TySrgbaColor` (becomes `TySrgba<u8>`) and
  the sRGB `TyRgbaColor` (becomes `TySrgba<f64>`/`<f32>`). One sRGB type; storage
  is an orthogonal generic axis, matching `palette`'s `Srgb<T>` and `cint`'s
  `EncodedSrgb<T>`.
- **`TyLinSrgba<T = f32>`** replaces `TyLinearRgbaColor`, taking the `palette`
  name `LinSrgb` the owner prefers.
- **Only genuinely-non-color channels leave the color namespace.** The metallic /
  roughness / occlusion reads in `sample_material` are linear data channels, not
  colors, so they read as plain scalars / `TyVector4<T>` (glTF "Non-Color Data": a
  data channel is not a color). The fbx per-point vertex colors, by contrast, ARE
  colors (base-color texels, `pixel / 255` sRGB float), so they STAY a color type
  (`TySrgba<f32>`), not a vector. No neutral RGBA color type remains.

## Background

A codebase-wide census plus prior-art research (`palette`, `bevy_color`, `egui`
`ecolor`, `kolor`/`colstodian`, and Rust naming conventions) produced the study
recorded in [reference/color-model-study.md](reference/color-model-study.md). The
load-bearing findings:

1. **The one resident sRGB float is the fbx vertex color** (`TyRgbaColor<f32>`
   today, `TySrgba<f32>` under the new model). Every OTHER sRGB use is a throwaway
   `new(...).to_srgba()` quantize transient (vxl `palette_show`, voxsmith
   `pool_color`, `bake_atlas`, `reduce_palette`) or a `to_rgba().to_array()`
   normalize into a voxcore pool. So the generic's `u8` and `f32` variants each have
   a real consumer -- `u8` for 8-bit storage / hex / the hash key, `f32` for the fbx
   vertex colors -- not just symmetry.
2. **voxcore already names the space at the pool level.** `VoxValuePool::{Srgb,
   Srgba, LinearRgb, LinearRgba}` store raw `[f64; N]` with the space in the
   variant name and validation, using no `ty-math` color type. voxcore is
   untouched by this plan; the sRGB-float concept lives there as raw pool arrays.
3. **A genuine raw-`[0, 1]` role exists**: fbx per-point colors serialized to JSON
   (via `TyRgbaColorSerde` through `tyt-injection`), and the
   metallic/roughness/occlusion reads in `sample_material` (`to_rgba().b/.g/.r` as
   linear data). These are not sRGB and not colors; they move to vectors.
4. **Prior art**: `palette` = `Rgb<S, T>` (space `S` as a parameter, component `T`
   separate; aliases `Srgb` / `LinSrgb` / `Srgba` / `LinSrgba`). `bevy_color` =
   one struct per space, all `f32`, no u8 type, no neutral `Rgba`. `egui` reserves
   the bare name `Rgba` for **linear**. The cross-crate rule: never leave a bare
   "Rgba" that implies a hidden space.

## Friction to resolve (eyes-open costs of the generic)

1. **`Eq` / `Hash` (and a contingent `Ord`).** `TySrgbaColor` derives `Eq` + `Hash`
   and is a `MaterialKey` hash key (`voxelize_mesh.rs:424`). A generic `TySrgba<T>`
   cannot derive them uniformly (`f64` is neither), so impl `Eq` / `Hash` for
   `TySrgba<u8>` only; `TySrgba<f64>` gets `PartialEq` alone. Keep the dedup key on
   `TySrgba<u8>`. If the optional `cell_color` / `pool_color` retype (carried over
   from the adoption plan) lands, the vmax `to_vmax_file.rs:438`
   `BTreeSet<([i32; 3], color)>` key also needs `Ord` on `TySrgba<u8>` (a sound
   derive over its u8 fields); otherwise leave that one site on raw bytes.
2. **Component conversion.** Replace `TySrgbaColor::to_rgba` (u8 -> f64 normalize)
   and `TyRgbaColorF64::to_srgba` (f64 -> u8 quantize) with a `palette`-style
   `into_format` / `to_u8` / `to_f64` pair between `TySrgba<u8>` and
   `TySrgba<f64>`. The encode/decode transfer stays on
   `TySrgba<f64>::to_lin_srgba` and `TyLinSrgba<f64>::to_srgba`.
3. **Serde and the fbx wire (no regression).** `TyRgbaColorSerde` is the only
   serialized color, on the fbx per-point-color path. Those are genuine vertex
   colors (`sample_texture` returns `pixel / 255` base-color texels, sRGB float),
   so they become `TySrgba<f32>` and the serde is simply RENAMED (`TySrgbaSerde`,
   `r`/`g`/`b`/`a` keys kept) rather than replaced by a vector. The wire stays
   byte-identical; no `TyVector4Serde` is added and there is no format regression.
   A color is not a vector: point-cloud formats (PLY, LAS) and USD name their color
   channels `r`/`g`/`b`/`a` too, so that shape is the convention, not `x`/`y`/`z`/`w`
   (glTF packs `COLOR_0` as a positional `VEC4` accessor in its binary buffer, which
   is a buffer-layout detail, not a JSON key choice). This retires the earlier
   "move colors to a vector" idea and its regression risk.
4. **The HSV family.** `TyHsvaColor` has no external producer or consumer; its only
   inbound edge is `TyRgbaColor::to_hsva`, and its only test loops back through it.
   Decide in the plan: delete `ty_hsva_color{,_f32,_f64}` with `to_hsva`, or keep
   `to_hsva` on `TySrgba<f64>`. Do not half-remove it (the study's critique caught
   this as a dead-API trap and a one-file blast-radius undercount).

## Blast radius

From the census: ~17 `TySrgbaColor` sites -> `TySrgba<u8>` across voxsmith
`convert` + `internal/mesh` + vxl; the `TyRgbaColorF64` transients -> `TySrgba<f64>`
or removed; the serde parity + fbx chain (`tyt-fbx`, `tyt-injection`); the
raw-data reads in `sample_material`. `voxcore` is untouched. Every wire format
except the fbx point-color serde stays byte-for-byte identical.

## Relationship to the adoption plan's C8

The reverted C8 (`TyRgbaColorF64::to_linear_rgba` + the voxj `decode_rgba`
adoption) folds into this plan: the sRGB decode lands here as
`TySrgba<f64>::to_lin_srgba`, and the voxj `decode_rgb`/`decode_rgba` (the
deferred 3-component and the reverted 4-component) resolve against the new types.
The current adoption plan does not re-land C8.

## Not in scope

- The `voxcore` `VoxValuePool` model and its serde stay as-is (raw arrays, named
  pool variants). No wire change there.
- `TyOklabColor` / `TyCielabColor` stay as they are (one-way perceptual sinks).
- The fbx point colors are vertex colors, typed `TySrgba<f32>` (sRGB float, the
  undecoded base-color texels). Their wire values are preserved byte-for-byte, so
  whether a downstream consumer re-interprets them as linear is a separate,
  pre-existing question this plan does not touch.
