# Palette-style color model checklist

Ordered migration for the [plan](README.md): collapse the two sRGB storages into a
generic `TySrgba<T>`, rename the linear type to `TyLinSrgba<T>`, and move raw
non-color channels off color types. Line numbers are from the design-study census;
confirm at the keyboard. The strategy is add-new / migrate / remove-old so each
step is a reviewable, staged chunk; the workspace stays green throughout.

## Ground rules

- One reviewable, staged chunk per step. `cargo fmt --all` + `cargo clippy
  --workspace --all-targets -- -D warnings` + `cargo test` before staging; the
  pre-commit hook enforces fmt + clippy.
- No wire format changes except the one deliberately-gated fbx point-color serde
  (step 7). Every other serde/golden stays byte-identical; if one moves, stop.
- Repo style: Rust edition 2024, consolidated nested `use`, one public type per
  file in snake_case, doc comments on public items, 80-column ASCII comments.

## Carried over from the ty-math adoption plan

Color-touching adoptions were moved out of the [ty-math adoption
plan](../ty-math-adoption/checklist.md) to land here against the new types, rather
than adopt the old ones first and redo them:

- The three-component qbcl `color_floats` (`from_qb_file.rs:185`,
  `from_qbt_file.rs:284`, `from_qbcl_file.rs:307`) -> resolved in step **S5** with
  the new sRGB type (no synthetic alpha needed once RGB has an honest home).
- vxl `fill_color.rs:39` `parse_rgba_hex` -> `TySrgba<u8>::from_hex` in step **S6**.
- `sample_material.rs` `CellAccum` retype onto vectors -> step **S6**, done with the
  base-color decode so `sample_material` is touched once.
- The reverted adoption-plan C8 (`to_linear_rgba` + voxj decode) -> step **S9**.

## Steps

### Phase 1: land the new types in ty-math (additive)

- [ ] **S1. Add `TySrgba<T = f32>`** and aliases (`TySrgbaU8 = TySrgba<u8>`,
      `TySrgbaF32`, `TySrgbaF64`) beside the existing types. Component-generic RGBA,
      documented as the sRGB-encoded space. `Eq` + `Hash` impl'd for `TySrgba<u8>`
      only (`f64` gets `PartialEq`). Port `from_hex`/`to_hex` to `TySrgba<u8>`, the
      array conversions, and `Mul<T>`. Unit tests.
- [ ] **S2. Component + transfer conversions on `TySrgba`.** `into_format` / `to_u8`
      / `to_f64` between `TySrgba<u8>` and `TySrgba<f64>` (normalize / quantize, no
      transfer), and `TySrgba<f64>::to_lin_srgba` (the sRGB decode, re-homing the
      reverted C8 logic). Keep the transfer OFF the plain component cast; name it so
      the sRGB step is explicit. Unit tests including the u8 round-trip.
- [ ] **S3. Add `TyLinSrgba<T = f32>`** beside `TyLinearRgbaColor`, with
      `to_srgba` (linear -> sRGB encode, returning `TySrgba<u8>` or `TySrgba<f64>`
      per call need), `to_oklab`, `to_cielab`, `componentwise_multiply`. Unit tests.
- [ ] **S4. Decide the HSV family.** Either delete `ty_hsva_color{,_f32,_f64}` and
      the `to_hsva` edge (updating the `ty_hsva_color.rs:125` round-trip test), or
      port `to_hsva` onto `TySrgba<f64>` and keep the family. Do not half-remove it.

### Phase 2: migrate consumers off the old types

- [ ] **S5. voxsmith `convert/**`**: `TySrgbaColor` -> `TySrgba<u8>` (~17 sites),
      the `TyRgbaColorF64` quantize transients -> `TySrgba<f64>` / `into_format`.
      Keep the `MaterialKey` dedup (`voxelize_mesh.rs:424`) on `TySrgba<u8>`. Gate:
      `cargo test -p voxsmith` green, no golden churn.
- [ ] **S6. voxsmith `internal/**`**: base-color decode -> `to_lin_srgba`; the raw
      metallic/roughness/occlusion reads (`sample_material.rs:294,316`) move to
      `TyVector4`/`[f64; 4]` component reads, out of the color namespace. `vxl`
      color sites -> the new types. Gate: `cargo test -p voxsmith -p vxl` green.

### Phase 3: serde, fbx wire, and removal

- [ ] **S7. Rename the fbx color serde (no wire change).** The fbx per-point colors
      are genuine vertex colors (base-color texels, sRGB float), so retype
      `TyRgbaColor<f32>` -> `TySrgba<f32>` across `tyt-fbx` (`create_point_cloud.rs`,
      `dependencies*.rs`) and `tyt-injection`, and rename `TyRgbaColorSerde` ->
      `TySrgbaSerde` (or a generic `TySrgbaSerde<T>`) keeping the `r`/`g`/`b`/`a`
      keys. They stay a color type -- no `TyVector4Serde`. Assert the fbx JSON is
      byte-identical.
- [ ] **S8. Remove the old types.** Delete `TySrgbaColor`, `TyRgbaColor`,
      `TyRgbaColorSerde` (and `TyLinearRgbaColor` once `TyLinSrgba` is adopted
      everywhere); fix `lib.rs` re-exports. Gate: `cargo check --workspace` clean,
      no references remain.
- [ ] **S9. Re-land the reverted C8.** Adopt `TySrgba<f64>::to_lin_srgba` at the
      voxj `decode_rgb` / `decode_rgba` in
      `voxj_value_pool_from_vox_value_pool.rs`, now that the 3-component and
      4-component both have honest homes under the new model. Gate: `cargo test -p
      voxsmith` green.

Gate (whole plan): workspace green, clippy clean, and every wire format identical
except the deliberately-decided fbx point-color serde. `voxcore` untouched.
