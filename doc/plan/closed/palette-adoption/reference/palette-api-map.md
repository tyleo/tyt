# palette 0.7 API map

The verified mapping from ty-math's hand-rolled color surface to `palette` 0.7.6
(source-verified against the 0.7.6 tag / docs.rs; re-confirm at the keyboard).
This is the factual backbone for the [checklist](../checklist.md); the
[README](../README.md) records the decisions.

## Crate facts

- `palette = "0.7.6"`, edition 2021, MSRV `1.71.0`. Compatible with the
  edition-2024 workspace (an older edition dep is fine).
- ty-math depends on it slim: `palette = { version = "0.7", default-features =
  false, features = ["std"] }`. Add `"approx"` only if ty-math tests want approx
  asserts (it is in palette's default set anyway).
- Do NOT enable `serializing` (would flip the wire to `red/green/blue/alpha` and
  pull serde into palette), `named` / `named_from_str` (pulls `phf`), or
  `random`.
- ty-math is the ONLY crate that names the `palette` crate. It re-exports every
  type alias and the two conversion traits, so no consumer writes `palette::`.
  This also sidesteps the domain "palette" collision (voxcore `BVoxPalette`,
  `reduce_palette`, `palette_show`): those are unrelated and untouched.

## Type aliases (all high confidence)

palette's crate-root aliases carry the encoding/white-point as leading generics
and default the component to `f32`, so tyt's `T = f32` default is preserved.

```
pub type TySrgb<T = f32>      = palette::Srgb<T>;      // Rgb<Srgb, T>
pub type TySrgba<T = f32>     = palette::Srgba<T>;     // Alpha<Srgb<T>, T>
pub type TyLinSrgb<T = f32>   = palette::LinSrgb<T>;   // Rgb<Linear<Srgb>, T>
pub type TyLinSrgba<T = f32>  = palette::LinSrgba<T>;  // Alpha<LinSrgb<T>, T>
pub type TyOklabColor<T = f32> = palette::Oklaba<T>;   // Alpha<Oklab<T>, T>
pub type TyCielabColor<T = f32> =
    palette::Laba<palette::white_point::D65, T>;        // Alpha<Lab<D65, T>, T>
```

- Fields rename: `r/g/b -> red/green/blue`; `a -> alpha`. On the alpha types the
  color part is reached through `Alpha`'s `Deref` (`c.red`) and `alpha` is a
  direct field (`c.alpha`). Oklab / Lab keep `l/a/b` and carry `alpha` on the
  `Alpha` wrapper. Owner has waived field-rename concerns.
- CRITICAL: `Laba<Wp = D65, T = f32>` puts the white point FIRST. The alias MUST
  pin `D65` (`palette::Laba<D65, T>`); `palette::Laba<T>` would bind `Wp = T`.
  ty-math re-exports `palette::white_point::D65`.
- The `*U8 / *F32 / *F64` aliases retarget to the palette type (e.g.
  `TySrgbaU8 = palette::Srgba<u8>`). ADD f64 aliases for the perceptual types
  (`TyOklabColorF64 = palette::Oklaba<f64>`, `TyCielabColorF64 = palette::Laba<D65,
  f64>`), because palette defaults these to f32 and every tyt conversion runs on
  f64 - consumers turbofish the f64 alias to avoid inferring f32.

## Method / operator map

| tyt                              | palette                                                   | import        |
|----------------------------------|-----------------------------------------------------------|---------------|
| `TySrgba<u8>::to_f64`            | `.into_format::<f64, f64>()` (Alpha: TWO params)          | none (inherent) |
| `TySrgb<u8>::to_f64`            | `.into_format::<f64>()` (Rgb: one param)                  | none          |
| `TySrgba<f64>::to_u8`           | `.into_format::<u8, u8>()`                                 | none          |
| `TySrgb<f64>::to_u8`            | `.into_format::<u8>()`                                     | none          |
| `TySrgba<f64>::to_lin_srgba`    | `.into_linear()` -> `LinSrgba<f64>`                       | none          |
| `TyLinSrgba<f64>::to_srgba`     | `TySrgbaF64::from_linear(lin)` (or `.into_encoding()`)    | none          |
| `TyLinSrgba<f64>::to_oklab`     | `.into_color::<TyOklabColorF64>()`                        | `IntoColor`   |
| `TyLinSrgba<f64>::to_cielab`    | `.into_color::<TyCielabColorF64>()`                       | `IntoColor`   |
| `to_srgb()` DROP-ALPHA          | `.color` (the `Alpha.color` field) / `TySrgb::from(..)`  | none          |
| `componentwise_multiply(&o)`    | `a * o` (component-wise `Mul`, scales alpha too)          | none (operator) |
| `Mul<T>` scalar (scales alpha)  | `c * k` (identical semantics)                             | none (operator) |
| `to_array()`                    | `<[T; N]>::from(c)` / `c.into()`                          | none          |
| `from_array(a)`                 | `TySrgb::from(a)` / `TySrgba::from(a)`                    | none          |
| `from_hex` / `to_hex`           | ty-math glue over palette `FromStr` (see below)          | `TyHexColor`  |
| `to_vector3()`                  | ty-math glue over `into_components()` (see below)         | `TyColorToVector3` |
| `from_slice` / `write_to_slice` | inline `<[T;N]>::try_from(&s[..N])` + `copy_from_slice`   | none          |

- `into_format` recasts only the component number type via `FromStimulus` and
  never touches the transfer function - the exact analog of tyt's transfer-free
  `to_u8` / `to_f64`. On `Alpha` the alpha recasts the same way.
- `into_linear` / `from_linear` apply the sRGB transfer to r/g/b only, alpha
  straight. In-gamut `[0, 1]` these are bit-comparable to tyt's `srgb_transfer`.
- LinSrgb -> Oklab uses palette's FUSED matrix, byte-identical to tyt's M1/M2, so
  Oklab results match tyt to full f64 precision.

## Traits and derives

- `PartialEq` / `Eq`: palette matches tyt exactly - `Eq` for the u8 storage,
  `PartialEq` only for float. Drop tyt's manual `impl Eq for TySrgba<u8>` etc.
- `Hash`: palette has NONE on any color type. This does NOT block us:
  `voxelize_mesh` `MaterialKey` is `([u8;4], u64, ...)` and every dedup already
  keys on `[u8; N]` arrays, never a color. Only ty-math's own unit test
  `u8_color_keys_a_hash_set` uses a color `HashSet`; it is deleted with the impl.
- `Ord` / `PartialOrd`: palette has NONE. Not exercised in production (the vmax
  color path is test-only via `color_floats`; `MaterialKey` uses arrays).
- `Default`: BEHAVIORAL DELTA - palette `Srgba::default()` is OPAQUE (alpha =
  max), tyt's derive gave transparent (alpha = 0). Audited clear: no consumer
  calls `::default()` on a color type and no consumer struct derives `Default`
  with a color field. Note it in the plan and re-audit at the keyboard.
- `Copy` / `Clone` / `Debug`: palette provides all three; `MeshMaterial`'s
  `PartialEq` / `Copy` derives keep working through the alias.

## Serde / wire (hard keep)

Keep the hand-rolled `TySrgbaSerde` (`r/g/b/a` f32) in ty-math-serde and its two
`From` impls. palette's own serde emits `red/green/blue/alpha`, so serializing a
palette type would break the pinned JSON contract
(`serialize_points_and_colors_json.rs` test). Orphan rule is safe both ways: the
DTO is local on one side of each `From`. Only the `From<TySrgba>` body changes to
read `c.red/.green/.blue/.alpha` (or `c.into_components()`).

## Re-exports ty-math must add

```
pub use palette::{FromColor, IntoColor};          // for .into_color()
pub use palette::white_point::D65;                // to pin the Lab alias
// plus the eight type aliases above and the *U8/*F32/*F64 family
```

Consumers then need only `use ty_math::{..aliases.., IntoColor, TyHexColor,
TyColorToVector3}`. The recast/encode/decode methods (`into_format`,
`into_linear`, `from_linear`) are inherent and need no import.

## Friction resolutions (see README "Friction")

1. Out-of-gamut transfer: palette applies the sRGB formula directly (NO CSS
   Color 4 sign extension); a negative component's power branch yields NaN.
   In-gamut identical. tyt's out-of-gamut sign-extension unit tests are dropped
   (behavior abandoned). Positive HDR (`> 1`) is fine (positive base, then u8
   clamp). Negative linear components do not occur on the production paths.
2. CIELAB numeric drift: palette's D65 tristimulus (0.95047 / 1.08883) and
   primaries-derived matrix differ from tyt's hardcoded constants, so Lab results
   match only to ~2-3 sig figs. Owner accepts drift; re-baseline any exact Lab
   assertions. Oklab is unaffected (identical matrix).
3. `into_format::<u8>()` half-boundary rounding vs `TyFloatExt::to_unorm8`
   (round-half-away-from-zero, then clamp): expected to match; at most +/-1 LSB
   on exact `.5`. Verify at the keyboard; accept drift and re-baseline goldens if
   it differs (owner relaxed byte-exactness).
4. Hex: palette `Srgba<u8>: FromStr` takes only 4/8-digit RGBA (rejects 6-digit),
   returns `Result`, accepts 3/4-digit shorthand; `UpperHex` omits `#`. tyt's
   contract is 6-or-8 digit with opaque default, `Option`, and `#RRGGBBAA`
   uppercase. RESOLUTION: keep a thin ty-math `TyHexColor` extension wrapping
   palette + the `#` prefix + opaque default. The one sanctioned "our own"
   helper; it is glue, not color math.
5. `const DEFAULT_FILL: TySrgbaU8 = TySrgbaU8 { .. }` (voxelize_mesh:16): palette
   `Srgba` has a `PhantomData` field and no const ctor. Change the const to
   `const DEFAULT_FILL: [u8; 4] = [..]` and construct at the 3 use sites via
   `TySrgbaU8::from(DEFAULT_FILL)`.
6. `to_vector3` / `from_slice` / `write_to_slice`: no palette equivalent.
   `to_vector3` -> a `TyColorToVector3` ext trait (bridge, palette cannot know
   `TyVector3`). `from_slice` / `write_to_slice` (few sites) -> inline at call
   sites.
