# Implementation decisions

Durable log of non-obvious code-level calls made while executing the
[plan](../README.md). Each entry is a decision a later session must stay
consistent with; read this before picking up work.

## S1: add `TySrgba<T = f32>` (2026-07-07)

- **File / alias layout.** `TySrgba` lives in `ty_srgba.rs`; the aliases are
  `TySrgbaU8` / `TySrgbaF32` / `TySrgbaF64` in `ty_srgba_u8.rs` /
  `ty_srgba_f32.rs` / `ty_srgba_f64.rs`, matching the one-type-per-file rule and
  the existing `Ty*ColorF32/F64` alias-file convention. The `u8` alias sorts
  after `f64` in `lib.rs` because the module list is strictly alphabetical
  (`ty_srgba`, `ty_srgba_color`, `ty_srgba_f32`, `ty_srgba_f64`, `ty_srgba_u8`).

- **`Eq` / `Hash` for `TySrgba<u8>` only.** `PartialEq` is derived generically;
  `Eq` is an empty impl on `TySrgba<u8>` and `Hash` is a hand-written field-order
  impl on `TySrgba<u8>` that mirrors what a derive would emit. `f32` / `f64` get
  `PartialEq` alone. This keeps the dedup key (`MaterialKey`,
  `voxelize_mesh.rs:424`) on the 8-bit storage. clippy is clean: the
  `derived_hash_with_manual_eq` lint fires on derived-`Hash` + manual-`PartialEq`,
  which is the opposite arrangement, so manual `Hash` + derived `PartialEq` is
  fine as long as they agree (they do).

- **Ported surface is exactly the checklist's: `new`, the array conversions (the
  generic `ty_array_conversions!` form: `to_array` / `from_array` / `from_slice`
  / `write_to_slice` + `From<[T; 4]>`, but NOT the reverse `From<Self> for
  [T; 4]`, which the macro emits only for the concrete form), `from_hex` /
  `to_hex` on `TySrgba<u8>`, and the forward `Mul<T>` (color * scalar).**

- **Omitted the scalar-first `Mul` (`scalar * color`).** The old
  `TyRgbaColor` carries a `impl Mul<TyRgbaColor<$t>> for $t` reverse impl; a
  workspace grep (`* Ty(Rgba|Srgba|Linear)`) found zero call sites, so it is dead
  and not ported. Re-add it additively only if a consumer migration turns one up.

- **Deferred `to_vector3` out of S1.** It is not in the checklist's S1 list, and
  its semantics are component-type-dependent: `TySrgbaColor::to_vector3`
  normalizes (u8 -> `TyVector3<f64>` via `to_rgba`) while
  `TyRgbaColor::to_vector3` is a plain drop-alpha. A uniform generic method
  cannot express both, so the call sites (reduce_palette, voxelize_mesh,
  from_qbcl_file) are migrated with the component conversions in S2 / the
  consumer steps, not here.

- **Doc header avoids linking `to_lin_srgba`.** That method lands in S2, so the
  S1 doc describes the decode conceptually ("decode to linear light before
  compositing") rather than with a `[`...`](Self::...)` intra-doc link that would
  not resolve yet. Add the link when the method exists.

## S2, part 1: component conversions (2026-07-07)

- **S2 is split at the dependency boundary; only the component conversions
  landed this chunk.** S2 has two independent pieces: the transfer-free
  component conversions (`to_u8` / `to_f64`), which need nothing new, and
  `TySrgba<f64>::to_lin_srgba` (the sRGB transfer decode), which needs a linear
  return type. To have `to_lin_srgba` return `TyLinSrgba<f64>` directly instead
  of the old `TyLinearRgbaColorF64` (which S8 removes, forcing a re-point),
  `to_lin_srgba` is deferred to the S3 chunk where `TyLinSrgba` is born. **S2
  stays unchecked; its remaining piece is `to_lin_srgba`, done with S3.**

- **Concrete `to_u8` / `to_f64`, not a generic `into_format<U>`.** The plan lists
  the palette vocabulary "`into_format` / `to_u8` / `to_f64`", but a genuine
  generic `into_format<U>` needs a `FromStimulus`-style component-conversion
  trait over `{u8, f32, f64}` -- disproportionate machinery for the one pair the
  census exercises, and against this crate's concrete-`impl` idiom (every type
  has explicit `F32` / `F64` impls, not generic-over-component). So the two
  directional methods `TySrgba<u8>::to_f64` (normalize, `/255`) and
  `TySrgba<f64>::to_u8` (quantize via `TyFloatExt::to_unorm8`, clamp + round) are
  the whole surface. `into_format` is treated as the concept, not a required
  method name. If a generic form is wanted later it is additive.

- **u8 <-> f64 only; no f32 conversion.** The census conversion sites are all
  u8 <-> f64 (`TySrgbaColor::to_rgba` normalizes, `TyRgbaColorF64::to_srgba`
  quantizes). The fbx `TySrgba<f32>` colors are stored / serialized, never run
  through these transient conversions, so no `to_f32` is added. Additive if a
  site turns up.

- **`to_u8` / `to_f64` are additive beside the old `to_rgba` / `to_srgba`.** The
  old `TySrgbaColor::to_rgba` and `TyRgbaColorF64::to_srgba` still exist and are
  removed at S8; consumers migrate to the new pair in S5 / S6.

## S3 + S2 transfer decode (2026-07-07)

- **`TyLinSrgba<f64>::to_srgba` returns `TySrgba<f64>`, transfer only.** The
  checklist wrote "returning `TySrgba<u8>` or `TySrgba<f64>` per call need". The
  chosen shape keeps the sRGB transfer (the encode) fully separate from the byte
  quantize (the component cast), matching README friction 2 ("keep the transfer
  explicit and off the plain component cast") and palette's linear -> encoded ->
  format pipeline. So `to_srgba` applies only the transfer and yields
  `TySrgba<f64>`; a caller needing bytes chains `to_srgba().to_u8()` (that is the
  "or `TySrgba<u8>`"). An ergonomic byte-returning convenience can be added
  additively if consumer migration (S5 / S6) wants it.

- **The float transfer odd-extends out-of-gamut (owner decision, 2026-07-07).**
  The old `TyLinearRgbaColor<f64>::to_srgba` clamped to `[0, 1]` before the
  transfer because it produced bytes. The new `to_srgba` instead preserves
  out-of-gamut values via the CSS Color 4 sign extension `sign(x) * f(|x|)`, and
  `to_lin_srgba` decodes the same way, so the two stay true inverses past the
  gamut. Both `linear_to_srgb` and `srgb_to_linear` run `x.abs()` through the
  piecewise curve and restore the sign with `copysign`. The byte path is
  unchanged: `to_srgba().to_u8()` still equals the old `to_srgba()` for any
  linear input, since out-of-gamut floats clamp to the same `0` / `255` endpoint
  at `to_u8`; no golden changes. A naive branch (linear slope for all negatives)
  was rejected as off-standard below `-0.0031308`.

- **`to_srgba` / `to_lin_srgba` are inverse transfer functions in the float
  domain.** `to_lin_srgba` (on `TySrgba<f64>`, in `ty_srgba.rs`) inverts the sRGB
  transfer on `[0, 1]` floats and returns `TyLinSrgba<f64>`; `to_srgba` (on
  `TyLinSrgba<f64>`) re-encodes. Alpha passes straight both ways (no gamma). The
  decode re-homes the reverted C8 logic from the byte-domain
  `TySrgbaColor::to_linear_rgba` into the float domain.

- **OKLab / CIELAB math is copied, not delegated to `TyLinearRgbaColor`.** During
  the additive phase both linear types coexist; `TyLinSrgba::to_oklab` /
  `to_cielab` duplicate the matrices from `ty_linear_rgba_color.rs` rather than
  couple new to old. The old file (and this duplication) is deleted at S8. No
  `Eq` / `Hash` on `TyLinSrgba` (linear colors are not hash keys), mirroring the
  old linear type.

- **The `to_lin_srgba` doc uses an intra-doc link `[`TyLinSrgba::to_srgba`]`;**
  it resolves (verified with `RUSTDOCFLAGS="-D warnings" cargo doc -p ty-math
  --no-deps`). Cross-instantiation method links like this one resolve against the
  inherent method, so they are safe to use once the target method exists.

## S4: delete the HSV family (owner decision, 2026-07-07)

- **Deleted, not ported.** Removed `ty_hsva_color.rs` / `ty_hsva_color_f32.rs` /
  `ty_hsva_color_f64.rs`, `TyRgbaColor::to_hsva` (and its `TyHsvaColor` import),
  and the three `lib.rs` mod / re-export lines. The 4 HSV tests went with the
  file. Nothing else changed: a workspace grep confirmed zero external producers
  or consumers, so there were no call sites to fix.

- **The README undercounted the family; confirmed at the keyboard.** It framed
  the HSV surface as just `TyRgbaColor::to_hsva` plus a loop-back test. In fact
  the family was a self-contained toolkit: `TyHsvaColor::to_rgba` (HSV -> RGB, 2
  tests), `TyHsvaColor::lerp` (shortest-arc hue lerp, 1 test), and
  `TyRgbaColor::to_hsva` (RGB -> HSV, 1 round-trip test). All dead. This is the
  "one-file blast-radius undercount" the study critique flagged; delete-all was
  still clean because none of it had external users.

- **`impl_ty_rgba_color_float!` now emits only the reverse `Mul` (`scalar *
  color`).** Left in place; the whole `TyRgbaColor` type is removed at S8, so the
  now-unused reverse `Mul` is not separately pruned here.

## S5, part 1: standalone value-pool converters (2026-07-07)

- **Consumers use the alias `TySrgbaU8`, not `TySrgba<u8>`.** The convert code
  imports the ty-math aliases everywhere (`TyVector3U32`, `TyTransformF64`, ...),
  never the inline generic form, so the migration follows suit. The plan's
  `TySrgba<u8>` shorthand maps to `TySrgbaU8` at call sites.

- **The mechanical shape.** `TySrgbaColor` -> `TySrgbaU8`, and the transient
  `TySrgbaColor::from_array(bytes).to_rgba().to_array()` (u8 -> normalized
  `[f64; 4]` into a `VoxValuePool::Srgba`) becomes `.from_array(bytes).to_f64()
  .to_array()`. Same for the `from_hex(...).to_rgba().to_array()` test helpers.
  Byte-for-byte identical output (`to_f64` is the exact `to_rgba` normalize);
  `cargo test -p voxsmith` stays green, no golden churn.

- **S5 split; this chunk = `goxl` / `mvox` / `vmax` only.** Those three
  `from_*_file` converters (plus `to_vmax_file`) use `TySrgbaColor` purely
  locally to build float pools, so they migrate self-contained. Deferred:
  - `qbcl/from_qbcl_file.rs`: its production `color_floats` never used
    `TySrgbaColor`; only a test helper does, and via `.to_vector3()` (RGB ->
    `[f64; 3]`). That pairs with the `to_vector3` decision below.
  - `gltf/from_gltf_bytes.rs` and `voxelize/voxelize_mesh.rs`: NOT self-contained.

- **S5/S6 coupling: the mesh types straddle the convert/internal boundary.**
  `MeshMaterial` (`base_color`/`emissive_factor: TySrgbaColor`, in
  `internal/mesh/mesh_material.rs`) and `MeshTexture` (`texels:
  Vec<TySrgbaColor>`, in `internal/mesh/mesh_texture.rs`) are S6 (`internal/**`)
  but are consumed by `voxelize_mesh` and `gltf` (S5, `convert/**`). So those two
  convert files cannot fully retype off `TySrgbaColor` without their internal
  mesh types changing too. They should migrate together with the internal mesh
  types as one mesh-pipeline chunk that spans S5 + S6, rather than by the strict
  phase boundary. The `MaterialKey` (`voxelize_mesh.rs`) keys on
  `base_color.to_array()` -> `[u8; 4]`, which `TySrgbaU8::to_array` still yields,
  so the dedup stays on the 8-bit bytes as planned.

- **`to_vector3` superseded by `TySrgb` (owner guidance, 2026-07-07).** The
  drop-alpha sites do NOT go to a `TyVector3`. The owner prefers more color types
  over collapsing colors to vectors, so channel-narrowing a color yields a color:
  a new `TySrgb<T>` (3-channel sRGB, the companion to `TySrgba`, mirroring
  palette's `Srgb` vs `Srgba`). See [[prefer-color-types-over-vectors]].

## S5/S6 prep: add `TySrgb<T>` (2026-07-07)

- **`TySrgb<T = f32>`** in `ty_srgb.rs`, aliases `TySrgbU8` / `TySrgbF32` /
  `TySrgbF64`. Three components (`r` / `g` / `b`), no alpha. `new` + array
  conversions (`ty_array_conversions!(TySrgb, 3, ...)`) + `Eq` / `Hash` on
  `TySrgb<u8>` only (mirrors `TySrgba`, so a byte RGB can key a dedup map).
  `TySrgba<T>::to_srgb(&self) -> TySrgb<T>` drops alpha. Additive; no consumers
  yet (the mesh chunk uses it).

- **This refines the plan's "raw data -> vectors" wording.** The README and S6
  said the non-color channels move to `TyVector4` / `[f64; 4]`. Per the owner,
  the rule is finer: a color that loses a channel stays a color (`TySrgb`); only
  values *actually* used as a vector or raw datum become vectors / scalars. So in
  the mesh pipeline: the emissive 3-channel pool uses `TySrgb` (not `to_vector3`);
  metallic / roughness / occlusion stay plain `f64` scalars (they are single
  values, genuinely not colors); `MeshTexture`'s dual-use store stays neutral
  `[u8; 4]` bytes, re-wrapped in `TySrgbaU8` at the color sample sites.

## Mesh-pipeline migration (2026-07-07)

Atomic migration of the mesh color types + their `convert/` consumers, six
files. Byte-identical; `cargo test -p voxsmith` stayed green (117 tests).

- **Types.** `MeshMaterial.base_color` / `emissive_factor`: `TySrgbaColor` ->
  `TySrgbaU8`. `MeshBaseColorMap.factor`: `TyLinearRgbaColorF64` ->
  `TyLinSrgbaF64`. `MeshTexture` store / `sample()`: `TySrgbaColor` -> neutral
  `[u8; 4]` (the owner's choice; the store is genuinely dual-use, so it holds
  raw bytes and each sample site interprets).

- **`sample_material` color sites decode via `.to_f64().to_lin_srgba()`.** The
  old byte-domain `TySrgbaColor::to_linear_rgba()` becomes
  `TySrgbaU8::from_array(texel).to_f64().to_lin_srgba()`. Byte-identical: for a
  `[0, 1]` (in-gamut) input the normalize-then-decode equals the old
  byte-domain decode. The linear encode `TyLinearRgbaColor::to_srgba()` (u8)
  becomes `TyLinSrgbaF64::new(...).to_srgba().to_u8()`, also byte-identical.

- **`sample_material` data sites read raw normalized scalars.** metallic /
  roughness read `texel.map(|b| b as f64 / 255.0)` then index `[2]` / `[1]`;
  occlusion reads `texel[0] as f64 / 255.0`. No color type, matching the old
  `.to_rgba().b` / `.g` / `.r` exactly (`to_rgba` was a pure `/255` normalize).

- **`voxelize_mesh` pools.** `srgba_pool` value `color.to_rgba().to_array()` ->
  `color.to_f64().to_array()`. `srgb_pool` value `color.to_vector3().to_array()`
  -> `color.to_f64().to_srgb().to_array()` (drop alpha through the `TySrgb` color,
  not a vector). Dedup keys stay on the raw `[u8; 4]` / `[u8; 3]` bytes;
  `MaterialKey` still keys `base_color.to_array()` -> `[u8; 4]`.

- **`DEFAULT_FILL` uses the alias in a const struct literal** (`const
  DEFAULT_FILL: TySrgbaU8 = TySrgbaU8 { r: 255, .. }`), which compiles: a
  fully-applied type alias is allowed in a struct expression, and the fields are
  `pub`.

- **`from_gltf_bytes`.** `linear_rgba` returns `TyLinSrgbaF64`; base color and
  `emissive_srgb` encode with `.to_srgba().to_u8()`; texture texels build a
  `Vec<[u8; 4]>` (`texels.push([r, g, b, a])`).
