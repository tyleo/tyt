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
plan](../../closed/ty-math-adoption/checklist.md) to land here against the new types, rather
than adopt the old ones first and redo them:

- The three-component qbcl `color_floats` (`from_qb_file.rs:185`,
  `from_qbt_file.rs:284`, `from_qbcl_file.rs:307`) -> resolved in step **S5** with
  the new sRGB type (no synthetic alpha needed once RGB has an honest home).
- vxl `fill_color.rs:39` `parse_rgba_hex` -> `TySrgba<u8>::from_hex` in step **S6**.
- `sample_material.rs` `CellAccum` retype onto vectors -> step **S6**, done with the
  base-color decode so `sample_material` is touched once.
- `internal/cell_color.rs` and `internal/pool_color.rs` (both return raw sRGB
  `[u8; 4]`) -> optionally return `TySrgba<u8>` in step **S6**. `pool_color` is
  already touched by S5/S6 (its body builds `TyRgbaColorF64`/`TyLinearRgbaColorF64`
  and calls `to_srgba`). Nine `cell_color` consumers repack the bytes into format
  voxels; two key a container on the color: the mvox `HashMap` needs `Eq`/`Hash`
  (covered by `TySrgba<u8>`) and the vmax `to_vmax_file.rs:438` `BTreeSet` needs
  `Ord` (NOT currently planned -- derive it on `TySrgba<u8>` or leave that one site
  on raw bytes).
- The reverted adoption-plan C8 (`to_linear_rgba` + voxj decode) -> step **S9**.

## Steps

### Phase 1: land the new types in ty-math (additive)

- [x] **S1. Add `TySrgba<T = f32>`** and aliases (`TySrgbaU8 = TySrgba<u8>`,
      `TySrgbaF32`, `TySrgbaF64`) beside the existing types. Component-generic RGBA,
      documented as the sRGB-encoded space. `Eq` + `Hash` impl'd for `TySrgba<u8>`
      only (`f64` gets `PartialEq`). Port `from_hex`/`to_hex` to `TySrgba<u8>`, the
      array conversions, and `Mul<T>`. Unit tests.
- [x] **S2. Component + transfer conversions on `TySrgba`.** `into_format` / `to_u8`
      / `to_f64` between `TySrgba<u8>` and `TySrgba<f64>` (normalize / quantize, no
      transfer), and `TySrgba<f64>::to_lin_srgba` (the sRGB decode, re-homing the
      reverted C8 logic). Keep the transfer OFF the plain component cast; name it so
      the sRGB step is explicit. Unit tests including the u8 round-trip.
      Done: `to_u8` / `to_f64` are the concrete pair (no generic `into_format`);
      `to_lin_srgba` landed with S3, returning `TyLinSrgba<f64>`. See
      reference/implementation-decisions.md.
- [x] **S3. Add `TyLinSrgba<T = f32>`** beside `TyLinearRgbaColor`, with
      `to_srgba` (linear -> sRGB encode, returning `TySrgba<u8>` or `TySrgba<f64>`
      per call need), `to_oklab`, `to_cielab`, `componentwise_multiply`. Unit tests.
      Done: `to_srgba` returns `TySrgba<f64>`, transfer only, out-of-gamut
      odd-extended by sign per CSS Color 4; byte output is `to_srgba().to_u8()`.
      Aliases `TyLinSrgbaF32/F64`. See reference/implementation-decisions.md.
- [x] **S4. Decide the HSV family.** Either delete `ty_hsva_color{,_f32,_f64}` and
      the `to_hsva` edge (updating the `ty_hsva_color.rs:125` round-trip test), or
      port `to_hsva` onto `TySrgba<f64>` and keep the family. Do not half-remove it.
      Done: deleted the family (owner decision). Removed `ty_hsva_color{,_f32,_f64}`,
      `TyRgbaColor::to_hsva` + its import, and the `lib.rs` mod/re-exports. The
      family was fully dead workspace-wide (also had `to_rgba` + `lerp`, not just
      `to_hsva`). See reference/implementation-decisions.md.

### Phase 2: migrate consumers off the old types

- [x] **S5. voxsmith `convert/**`**: `TySrgbaColor` -> `TySrgba<u8>` (~17 sites),
      the `TyRgbaColorF64` quantize transients -> `TySrgba<f64>` / `into_format`.
      Keep the `MaterialKey` dedup (`voxelize_mesh.rs:424`) on `TySrgba<u8>`. Gate:
      `cargo test -p voxsmith` green, no golden churn.
      Done: `goxl` / `mvox` / `vmax` value-pool converters via `TySrgbaU8` +
      `.to_f64()`; `gltf` (`from_gltf_bytes`) and `voxelize_mesh` in the
      mesh-pipeline chunk with the S6 mesh types; `qbcl` in the qbcl chunk -- the
      `qb` / `qbt` / `qbcl` production `color_floats` adopt
      `TySrgbU8::from_array(..).to_f64()`, and the `from_qbcl_file` test helper
      moves off `.to_rgba().to_vector3()` to `.to_f64().to_srgb()`. Added
      `TySrgb<u8>::to_f64` for the 3-channel byte normalize. No golden churn; the
      whole `convert/` tree is off the old color types. See
      reference/implementation-decisions.md.
- [x] **S6. voxsmith `internal/**`**: base-color decode -> `to_lin_srgba`; the raw
      metallic/roughness/occlusion reads (`sample_material.rs:294,316`) move to
      `TyVector4`/`[f64; 4]` component reads, out of the color namespace. `vxl`
      color sites -> the new types. Optionally retype `pool_color`/`cell_color` to
      return `TySrgba<u8>` (see the carried-over note; the vmax `BTreeSet` key needs
      `Ord`). Gate: `cargo test -p voxsmith -p vxl` green.
      Progress: the `internal/mesh/**` types done in the mesh-pipeline chunk:
      `MeshMaterial.base_color`/`emissive_factor` -> `TySrgbaU8`,
      `MeshBaseColorMap.factor` -> `TyLinSrgbaF64`, `MeshTexture` store -> neutral
      `[u8; 4]`; `sample_material` decodes color sites via
      `.to_f64().to_lin_srgba()`, encodes via `.to_srgba().to_u8()`, and reads
      metallic/roughness/occlusion as raw normalized scalars (out of the color
      namespace, per the owner's finer rule -- scalars, not `TyVector4`).
      The rest of `internal/**` done in the internal-pools chunk: `pool_color`
      and `bake_atlas` encode pools via `TySrgbaF64::to_u8` (sRGB) /
      `TyLinSrgbaF64::to_srgba().to_u8()` (linear); `write_vmax` decodes via
      `TySrgbaU8::from_array(..).to_f64().to_lin_srgba()`. The whole `internal/`
      tree is now off the old types. Return types are unchanged: the optional
      `pool_color`/`cell_color` -> `TySrgba<u8>` retype (and the vmax `BTreeSet`
      `Ord` question) is still deferred. The `vxl` `palette_show` pool decode
      migrated identically (same 4-arm shape, `cargo test -p vxl` green). Top-level
      `reduce_palette` done in the reduce-palette chunk: `material_color` is the same
      4-arm decode, and `to_space` maps a color to a distance coordinate via the new
      `TySrgb::to_vector3` (owner's call: the RGB arm is a genuine coordinate here,
      beside the Oklab/Lab vectors). `voxsmith` and `vxl` are now fully off the old
      color types; the fbx serde chain (S7) and the old type definitions (S8)
      remain. See reference/implementation-decisions.md.

### Phase 3: serde, fbx wire, and removal

- [x] **S7. Rename the fbx color serde (no wire change).** The fbx per-point colors
      are genuine vertex colors (base-color texels, sRGB float), so retype
      `TyRgbaColor<f32>` -> `TySrgba<f32>` across `tyt-fbx` (`create_point_cloud.rs`,
      `dependencies*.rs`) and `tyt-injection`, and rename `TyRgbaColorSerde` ->
      `TySrgbaSerde` (or a generic `TySrgbaSerde<T>`) keeping the `r`/`g`/`b`/`a`
      keys. They stay a color type -- no `TyVector4Serde`. Assert the fbx JSON is
      byte-identical.
      Done: concrete `TySrgbaSerde` (f32 keys) in `ty_srgba_serde.rs` replacing
      `ty_rgba_color_serde.rs`, matching the sibling `TyVector3Serde` shape; the fbx
      chain uses bare `TySrgba` (the crates' local idiom, defaulting to f32, not the
      `F32` alias). A new `serializes_to_stable_json` test in `tyt-injection` pins
      the exact bytes -- the rename is wire-invisible since serde keys off the
      `r`/`g`/`b`/`a` field names, not the struct name. See
      reference/implementation-decisions.md.
- [x] **S8. Remove the old types.** Delete `TySrgbaColor`, `TyRgbaColor`,
      `TyRgbaColorSerde` (and `TyLinearRgbaColor` once `TyLinSrgba` is adopted
      everywhere); fix `lib.rs` re-exports. Gate: `cargo check --workspace` clean,
      no references remain.
      Done: deleted the seven old-type files (`ty_srgba_color.rs`,
      `ty_rgba_color{,_f32,_f64}.rs`, `ty_linear_rgba_color{,_f32,_f64}.rs`) and
      their fourteen `lib.rs` mod / re-export lines. `TyRgbaColorSerde` was already
      gone (renamed in S7). Cleared the one dangling comment reference in
      tyt-injection's `serialize_points_and_colors_json` test. `cargo check
      --workspace` clean, clippy clean, `cargo test --workspace` green (ty-math 84,
      down the 7 tests that lived in the deleted files). See
      reference/implementation-decisions.md.
- [x] **S9. Re-land the reverted C8.** Adopt `TySrgba<f64>::to_lin_srgba` at the
      voxj `decode_rgb` / `decode_rgba` in
      `voxj_value_pool_from_vox_value_pool.rs`, now that the 3-component and
      4-component both have honest homes under the new model. Gate: `cargo test -p
      voxsmith` green.
      Done: `decode_rgba` = `TySrgbaF64::from_array(..).to_lin_srgba().to_array()`;
      `decode_rgb` decodes through the same `to_lin_srgba` with a discarded
      placeholder alpha (no 3-channel linear type exists). Removed the inlined
      production `srgb_to_linear`; kept it in the test module as an independent
      reference the pool-decode tests cross-check against. Value-identical for the
      `[0, 1]` sRGB-float domain (the shared decode adds only CSS Color 4 sign
      extension, a no-op in gamut). `cargo test -p voxsmith` green (117);
      `--workspace` green. See reference/implementation-decisions.md.

Gate (whole plan): workspace green, clippy clean, and every wire format identical
except the deliberately-decided fbx point-color serde. `voxcore` untouched.
