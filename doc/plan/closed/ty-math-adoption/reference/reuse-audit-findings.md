# Broad ty-math reuse audit findings

_Part of the [ty-math Adoption Plan](../README.md)._

A codebase-wide audit (2026-07-05) for hand-rolled math the `ty-math` crate
could own, run as a multi-agent workflow: a `ty-math` surface census, twelve
subsystem finders spanning all of voxsmith `convert/` + `internal/` plus `vxl`
and `voxcore`, a dedup/rank synthesis, and an adversarial verify pass. It deduped
54 raw findings into 30 proposals, each verified: 14 confirmed, 9 real but
scope-narrowed, 7 rejected. This file is the durable record; the actionable items
are tracked as Track D in the [checklist](../checklist.md), and the decisions in
[implementation-decisions.md](implementation-decisions.md).

Sites are `path:line` from the audit; confirm at the keyboard, since edits shift
them. "VMAX" marks a file under `convert/vmax` or `internal/vmax`, which a second
branch also edits, so those land in their own trailing commits (Q2).

## Already landed

- `f2d4d5d` adopted `TyFloatExt::to_unorm8` (4 sites: `bake_atlas.rs:98`,
  `reduce_palette.rs` test hex helper, `voxj_value_pool_from_vox_value_pool.rs:97`,
  vxl `palette_show.rs:622`) and `TySrgbaColor::to_vector3` at
  `voxelize_mesh.rs:371`. Byte-identical, no golden moved.

## Group C: new ty-math methods (all approved to build)

Each in the crate's existing per-file float-macro form (or a generic/trait impl
where noted), with a doc comment and a unit test in that file's `tests` module.

- **C1. Integer `component_min_with` / `component_max_with`** on
  `impl<T: Copy + Ord> TyVector3<T>` (i32/u32). Cannot collide with the float
  macro (`f32`/`f64` are not `Ord`). Mirrors the existing float pair.
  - `voxcore/src/vox_object.rs:188-196` (`live_extent`, both, clean on
    `TyVector3U32`).
  - `convert/vmax/from_vmax_file.rs:474-483` (`min_corner`, min only, folds over
    `[i32;3]`, needs `from_array`/`to_array` wrapping). VMAX.
  - Verdict: needs-revision (sites corrected above; `object_bounds` is a
    subtract/+1/u32-cast fold, excluded).

- **C2. `TyVector3<u32>::to_i32(self) -> TyVector3<i32>`** and
  **`TyVector3<i32>::to_u32(self) -> TyVector3<u32>`** (componentwise `as`).
  Mirrors the landed `TyVector3<i32>::to_f64`.
  - `to_i32` (~5 sites): `internal/grid.rs:27-31`, `:36`, `:59-63`;
    `internal/voxj/voxj_decoded_object_from_vox_object.rs:58-62`;
    `convert/goxl/to_goxl_file.rs:201`; `convert/mvox/to_mvox_file.rs:462`.
  - `to_u32` (1 site): `internal/grid.rs:59-63` is the round-trip
    `(p.to_i32() + offset).to_u32()`.
  - Caveat: the i64-widened overflow guard at
    `vox_object_from_voxj_decoded_object.rs:69-74` has no `TyVector3<i64>` and
    stays hand-rolled.
  - Verdict: needs-revision (`to_i32` broadly reused; `to_u32` single-site).

- **C3. `TyVector3<u32>::to_f64(self) -> TyVector3<f64>`** (lossless widen,
  completes the family; `i32` already has it).
  - vxl `hierarchy_show.rs:929`, `:935` (x2); delete the `vec_u32_to_f64` helper
    (`:1177`).
  - Verdict: confirmed.

- **C4. `TyQuaternion::is_normalized(self, tolerance: T) -> bool`** =
  `(self.magnitude_squared() - 1).abs() <= tolerance`, on the existing
  `magnitude_squared`.
  - `voxcore/src/vox_main.rs:602-611` (call inverts: `!q.is_normalized(tol)`).
  - `voxel-codecs/voxj-codec/src/internal/voxj_validation/check_transforms.rs:27-36`
    (components are loose; build `TyQuaternion::new(..)` first).
  - Verdict: confirmed.

- **C5. Axis-rotation helpers `zup_to_yup` / `yup_to_zup`** on
  `impl<T: Copy + Neg<Output = T>> TyVector3<T>` (generic, covers f32 and f64):
  `zup_to_yup = (x, z, -y)` (a +90 rotation about X) and its inverse
  `yup_to_zup = (x, -z, y)`. Landed as sign-aware named helpers, NOT the six
  GLSL swizzles first sketched here: the glTF conversions are +/-90 rotations
  about X (a permutation PLUS a sign flip), so a pure permutation like `.xzy() =
  (x, z, y)` never produces the `-y`/`-z` and would need a trailing negate at the
  call site, no cleaner than the explicit `new(x, z, -y)`. There is no
  pure-permutation site in the tree, so the swizzles had no clean consumer; the
  owner chose the domain-named rotation instead.
  - `convert/gltf/from_gltf_bytes.rs:457` (import, `world.yup_to_zup()`).
  - `internal/gltf/object_to_gltf_document.rs:27-28` (export, `p.zup_to_yup() *
    scale` and `n.zup_to_yup()`; the `* scale` stays the existing `Mul<T>`).
  - `internal/gltf/material_document.rs:77-78` (export, same pair).
  - Verdict: confirmed, with a real consumer at all three sites.

- **C6. `TyBounds::from_min_size(min, size) -> Self`** (extents = size * 0.5,
  center = min + extents). (`from_min_max` was a lone one-off; dropped.)
  - `internal/vmax/write_vmax.rs` `content_box` (~`:1121-1154`), `object_box_local`
    (~`:590-602`). Corners arrive mixed i32/u32/f64, so a `to_f64` widen precedes;
    identical to the current `as f64`/`* 0.5`. VMAX.
  - Verdict: needs-revision (`from_min_size` holds; `from_min_max` dropped).

- **C7. `componentwise_divide(&self, other: &Self) -> Self`** on
  `impl<T: Div + Copy> TyVector3<T>` (mirror of `componentwise_multiply`).
  - `internal/mesh/grid_space.rs:36-42` (`to_grid`); retype the private `size`
    field to `TyVector3F64`. Lone site, but approved.
  - Verdict: needs-revision (1 real divide site).

- **C8. `TyRgbaColorF64::to_linear_rgba(self) -> TyLinearRgbaColorF64`** (float
  sRGB decode reusing the existing private `srgb_to_linear`; alpha passthrough).
  Distinct from the u8 `TySrgbaColor::to_linear_rgba`, which quantizes to 8-bit
  first, so that one is NOT numerically equal to this f64 [0,1] decode.
  - `internal/voxj/voxj_value_pool_from_vox_value_pool.rs:104-132`
    (`decode_rgb`/`decode_rgba`). Lone site, but approved.
  - Verdict: needs-revision.

## Group A: adopt existing ty-math, non-vmax (no new API)

Behavior-preserving; each is a verbatim duplicate of a method `ty-math` already
has.

- **A1. `TyVector3F64` `Sub`/`dot`/`cross`** in
  `internal/mesh/triangle_box_overlap.rs:55-71`: delete the private `sub`/`dot`/
  `cross` free fns, route the SAT test through `from_array`. Confirmed.
- **A2. Retype `CellAccum`** in `internal/mesh/sample_material.rs` onto
  `TyVector4F64`/`TyVector3F64` with `Add` + `Div<T>` + `to_array`/`from_array`:
  fields `:147-166`, `+=` `:190-207`, `/ n` `:243-267`, the `[c.r,c.g,c.b,c.a]`
  at `:277-283`. Emissive keeps its literal `1.0` alpha (divide still
  vectorizes). Private local accumulator, no wire/model touched. Confirmed.
- **A3. `TyVector3I32::to_f64` + `from_array`**: vxl `hierarchy_show.rs:1172`
  (delete `vec_i32_to_f64`; call `:928` needs only `.to_f64()`) and mvox
  `from_mvox_file.rs:348` (`from_array([i32;3]).to_f64()`). Confirmed.
- **A4. `TySrgbaColor::from_hex` + `From<TySrgbaColor> for [u8;4]`**: vxl
  `fill_color.rs:39` (call), delete `parse_rgba_hex` (`:49-65`). `from_hex`
  tolerates an optional leading `#` (a superset of the pre-stripped input).
  Confirmed.
- **A5. `TyVector3F32::INFINITY` / `NEG_INFINITY` consts**: the AABB-fold seeds at
  `internal/gltf/object_to_gltf_document.rs:43-44` and
  `internal/gltf/material_document.rs:83-84`. Confirmed.
- **A6. `to_array` / `from_array` / `From<[T;N]>` packing** (6 voxj sites):
  `voxj_hierarchy_node_from_vox_hierarchy_node.rs:27-38`;
  `vox_object_from_voxj_decoded_object.rs:41,51`;
  `voxj_decoded_object_from_vox_object.rs:18-19`; `write_voxj.rs:81-82`;
  `convert/mesh/object_to_mesh_geometry.rs:32`. PARTIAL:
  `vox_hierarchy_node_from_voxj_hierarchy_node.rs:36,75` (position only;
  rotation/scale keep their validation + normalization). Drop the `grid.rs:19`
  `splat(0)` case (not an array conversion). Needs-revision.
- **A7. `TyBounds::from_points` + `size()` / `max()`**:
  `convert/voxelize/mesh.rs:41-48` (`extent`, `size()` == `max - min`) and the
  test at `convert/gltf/object_to_glb_bytes.rs:87-88` (`max()`). Confirmed.
  CAUTION: do NOT adopt in `internal/mesh/triangle_bounds.rs` -- reconstructing
  min/max through separately-halved center +/- extents is not bit-exact and can
  nudge a voxel-cell boundary. Keep its direct `(min, max)` fold unless `TyBounds`
  gains a min/max-preserving constructor.
- **A8. `TyVector3F64::ZERO` const**: vxl `hierarchy_show.rs:936`. Lone production
  one-off; cosmetic. Needs-revision.

## Group B: adopt existing ty-math, vmax-only (trailing commits)

- **B1. Delete `write_vmax`'s `vector()` helper -> `to_array`**:
  `internal/vmax/write_vmax.rs:1054`, `:1231`, `:1233` (helper `:1311`). Confirmed.
- **B2. `TyTransformF64::transform_point`**: `write_vmax.rs:1096` (call); delete
  the hand-rolled TRS transform at `:1180-1193`. Confirmed.
- **B3. Round-to-nearest chain** (`to_f64`/`Sub`/`Add`/`round`/`to_i32`/
  `from_array`): `from_vmax_file.rs:529-535` (`pivot_origin`), `:569-573`
  (`authored_box` box_min). The offset at `:165-169` is a pure `TyVector3I32` Sub,
  no rounding. Confirmed.
- **B4. `min_corner` integer min fold**: `from_vmax_file.rs:474-483`, pairs with
  C1.
- **B5. `content_box` / `object_box_local`**: adopt C6 `from_min_size`.
- **B6. `extend_bounds`**: `write_vmax.rs:1158-1178`, float `component_min_with`/
  `component_max_with` or `TyBounds::encapsulate`. Single fold site.
  Needs-revision.

## Rejected (do not build)

- `TyQuaternion::normalized` adoption for the voxj hierarchy quaternion
  (`vox_hierarchy_node_from_voxj_hierarchy_node.rs:63-83`): `normalized()` is
  `self * (1.0 / magnitude())` (multiply-by-reciprocal), the hand-rolled code is
  per-component `x / magnitude` (true division); these round differently in
  IEEE-754 and could shift voxj/VoxState goldens. Keep the explicit divide. (Its
  `magnitude_squared` half is covered by C4.)
- A `barycentric` associated method: the existing scalar `Mul<T>` + `Add` express
  `p0*w0 + p1*w1 + p2*w2` bit-identically, so adopt operators if anything, not a
  new method.
- `l1_norm` (sum of |components|): one site (`triangle_box_overlap.rs:50`), raw
  array code.
- `(a - b).magnitude()` for the `edge` distance (`sample_material.rs:338-341`):
  one 2-line site.
- `transform_aabb_conservative` on `TyTransform`: one site AND not bit-identical
  (hand-rolled uses 3x `rotate()` + abs-of-columns vs the matrix-row form;
  diverges on negative scale).
- The `f32 -> f64` widen family on `TyVector3<f32>`/`TyVector2<f32>`/
  `TyMatrix4x4<f32>`: three lone one-offs that widen raw gltf-crate arrays, not
  ty-math types.
- `TyVector3F64::to_u32` (round + saturating `as u32`): one site
  (`authored_box:575-577`); the sibling integer-extent sites want an i32->u32
  variant instead.
- The 4-component `/255` test helper at `palette_show.rs:874`: test-only, one
  site.
