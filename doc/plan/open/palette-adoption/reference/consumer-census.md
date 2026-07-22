# Consumer census

Every consumer of a ty-math color type, from a workspace grep plus a per-cluster
read (2026-07-22). Line numbers are from that snapshot; confirm at the keyboard.
`voxcore` is untouched (its `VoxValuePool` stores raw `[f64; N]` / `[u8; N]`, no
color type). The `palette` crate name appears nowhere today.

## By symbol (consumers only, outside ty-math / ty-math-serde)

`TySrgba` 20, `TySrgb` 5, `TyLinSrgba` 5, `TyLinSrgb` 5, `TySrgbaSerde` 3,
`TyOklabColor` / `TyCielabColor` 0 by name (only reached via `.to_oklab()` /
`.to_cielab()` return values in `reduce_palette`).

## Cluster A - voxsmith/convert (9 files)

- `gltf/from_gltf_bytes.rs` - `TyLinSrgbaF64::new` then `.to_srgba().to_u8()`
  (L247, L268-272); plain `TyLinSrgbaF64::new` -> `MeshBaseColorMap.factor`
  (L342-349); field reads `.r/.g/.b`; test `to_hex` asserts `"#FF0000FF"` (L729),
  `from_hex` (L857).
- `goxl/from_goxl_file.rs` - `TySrgbaU8::from_array(color).to_f64().to_array()`
  (L86); test `from_hex` (L289). NON-SITES: `voxel.r/.g/.b/.a` is `GoxlVoxel`
  (L71, L126, L555) - leave alone.
- `mvox/from_mvox_file.rs` - same `[u8;4]->[f64;4]` builder (L107). NON-SITES:
  `MVoxColor` `.r/.g/.b/.a` (L102, L680).
- `qbcl/from_qb_file.rs` - `TySrgbU8::from_array(color).to_f64().to_array()`
  ([u8;3]->[f64;3], L181). NON-SITES: `QbVoxel` fields.
- `qbcl/from_qbcl_file.rs` - `TySrgbU8` builder (L303); test `TySrgbaU8::from_hex
  ...to_f64().to_srgb().to_array()` - `to_srgb` is DROP-ALPHA (L397). NON-SITES:
  `QbclColor` / `QbclVoxel`.
- `qbcl/from_qbt_file.rs` - `TySrgbU8` builder (L280). NON-SITES: `QbtColor` /
  `QbtVoxel`.
- `vmax/from_vmax_file.rs` - `TySrgbaU8` builder (L272); emissive drops alpha via
  array destructure `let [r,g,b,_] = ...` (L316). CAUTION: many `to_f64` /
  `from_array` / `componentwise_multiply` here are on `TyVector3*`, NOT color
  (L349, L541, L593, L631, L660, L665, L674, ...). Do not conflate.
- `vmax/to_vmax_file.rs` - color use is TEST-ONLY: `color_floats()` =
  `TySrgbaU8::from_hex(hex).to_f64().to_array()` (L585), fanned across most tests.
  Production delegates to `write_vmax` with no color type.
- `voxelize/voxelize_mesh.rs` - `const DEFAULT_FILL: TySrgbaU8 { r,g,b,a }`
  (L16, see api-map friction 5); `fill_srgba` (L543); color-pool builders
  `.to_f64()` (L389, L414) and `.to_srgb()` drop-alpha (L414); `MaterialKey`
  (L461) is `([u8;4], u64, ...)` - keys on BYTES, not a color, so no Hash needed.

Cluster A frictions: `.r/.g/.b/.a` renames only at from_gltf L858 and
voxelize_mesh L411/L470 (const literal); every foreign voxel/color `.r/.g/.b`
must be left. Hex assertions compare literal `#RRGGBBAA` (needs the `TyHexColor`
glue). No color is a Hash / Ord / serde / scalar-Mul key here.

## Cluster B - voxsmith/internal + reduce_palette (8 files)

- `internal/mesh/mesh_material.rs` - `base_color` / `emissive_factor: TySrgbaU8`
  (pure alias swap); `MeshMaterial` derives `PartialEq` + `Copy` (both hold on
  palette). `MeshMaterial::flat(TySrgbaU8)`.
- `internal/mesh/mesh_base_color_map.rs` - `factor: TyLinSrgbaF64` (pure alias).
- `internal/mesh/sample_material.rs` - decode `TySrgbaU8::from_array(texel)
  .to_f64().to_lin_srgba()` then `.componentwise_multiply(&map.factor)` (L281),
  field reads `.r/.g/.b/.a` (L285, L306-308); encode `TyLinSrgbaF64::new(..)
  .to_srgba().to_u8()` (L243, L262).
- `internal/pool_color.rs` - the 4-arm pool decode (Srgb / Srgba / LinearRgb /
  LinearRgba) -> `[u8;4]`.
- `internal/gltf/bake_atlas.rs` - `color_bytes_or` 4-arm decode; `linear.r/.g/.b`
  (L236); uses `TyFloatExt::to_unorm8` (scalar, STAYS).
- `internal/vmax/write_vmax.rs` - decode `TySrgbaU8::from_array(..).to_f64()
  .to_lin_srgba()`; luminance dot reads `.r/.g/.b` (L747, L846).
- `internal/voxj/voxj_value_pool_from_vox_value_pool.rs` - `decode_rgba` /
  `decode_rgb` via `to_lin_srgba`; `.r/.g/.b` (L112).
- `reduce_palette.rs` - `to_space` (L543-551): `TySrgbaU8::from_array` then per
  space `.to_f64().to_srgb().to_vector3()` (Srgb) /
  `.to_f64().to_lin_srgba().to_oklab().to_vector3()` (Oklab) / `.to_cielab()
  .to_vector3()` (Lab); `material_color` 4-arm decode (L174-191).

Cluster B frictions: `to_vector3` (no palette equiv - `TyColorToVector3` glue) is
the sharpest, the whole clustering pipeline consumes it. `componentwise_multiply`
-> `*` (by-value). Three byte-identical 4-arm pool decodes (`pool_color`,
`bake_atlas color_bytes_or`, `reduce_palette material_color`) can collapse to one
palette helper (optional simplification). No Hash / Ord / serde / hex here.

## Cluster C - vxl + treegrid (3 files)

- `vxl/implementation/palette_show.rs` - the 4-arm pool decode (mirrors
  `reduce_palette material_color`); `srgb_hex` (L536) is HAND-ROLLED from
  `[u8;4]`, never tyt `to_hex` - palette hex irrelevant here; `scalar_level`
  (L658) uses scalar `to_unorm8` (STAYS).
- `treegrid/color/tree_grid_value.rs` - four generic ctors `srgb` / `srgba` /
  `lin_rgb` / `lin_rgba`, each `T: Copy + Display + TyFloatExt + Into<f64>`,
  widening to f64 then `.to_u8()` / `.to_srgb()` / `.to_srgba()`; read-back via
  `.to_array()` only (no `.r/.g/.b` field access).
- `treegrid/color/tree_grid_json_value.rs` - JSON sibling of the above.

Cluster C frictions: method gap is dominant (`to_u8` -> `into_format`, `to_array`
-> `.into()`, `to_srgb` overloaded: DROP-ALPHA on `TySrgba<u8>` vs TRANSFER on
`TyLinSrgb<f64>` - rewrite each by hand, not find-replace). Generic bounds shift:
the ctors already widen to f64 before converting, so palette calls land on the
f64 alias and the `TyFloatExt` bound likely DROPS (net simpler); split the `use`
line since scalar `to_unorm8`/`TyFloatExt` stays for `scalar_level`.

## Cluster D - tyt-fbx + tyt-injection + ty-math-serde (5 files)

- `ty-math-serde/ty_srgba_serde.rs` - KEEP the DTO; the only body change is
  `From<TySrgba>` reading `c.red/.green/.blue/.alpha` instead of `.r/.g/.b/.a`
  (L18-21). Wire keys `r/g/b/a` come from the DTO field names, unchanged.
- `tyt-fbx/commands/create_point_cloud.rs` - `TySrgba::new(..)` (L429).
- `tyt-fbx/dependencies.rs` / `dependencies_impl.rs` - `&[Vec<TySrgba>]`
  signature-only (L31-35 / L48-56); compile unchanged once the alias re-exports.
- `tyt-injection/serialize_points_and_colors_json.rs` - `TySrgba::new` (test
  L43); `.copied()` needs `Copy` (palette `Srgba<f32>` is `Copy`); the pinned
  `r/g/b/a` JSON test (L48-52) stays green.

Cluster D frictions: one field rename total; everything else is `::new` or
signature. Wire fully insulated by keeping the DTO. Orphan rule safe both
directions (DTO local on one side of each `From`).
