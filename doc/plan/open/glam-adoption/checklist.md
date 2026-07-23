# glam adoption checklist

Ordered migration for the [plan](README.md): replace ty-math's hand-rolled math
types with concrete `glam` aliases + a small set of extension traits and move
every consumer onto glam's own API. Line numbers are from the census snapshot
([reference/consumer-census.md](reference/consumer-census.md)); confirm at the
keyboard. The alias flip is atomic (see README "Commit strategy"), so the
workspace goes red at S2 and returns to green only at S8 - stage across sessions,
commit once.

## Ground rules

- Prefer glam's behavior AND its names; do NOT preserve byte-exactness.
  Re-baseline internal tests/goldens that legitimately shift (Debug format,
  near-gimbal euler). Stop only if an EXTERNAL wire would move - none should (S8).
- **Fail-fast: use glam's strict methods directly** (`from_axis_angle`,
  `inverse`, `normalize`, `is_normalized`, `slerp`). Do NOT re-add ty-math's old
  defensive normalize/guard as a silent override. Normalize explicitly at the
  call site where an input is not provably unit. `debug-glam-assert` (enabled in
  S1) makes a violation panic in tests - that is the signal to add `.normalize()`.
  Only if an auto-fixing pattern genuinely recurs, add a DISTINCTLY NAMED variant
  (e.g. `from_axis_angle_normalized`), never a silent one.
- Only ty-math names `glam`. Consumers import `ty_math::...` only; if a consumer
  would need `glam::`, ty-math is missing a re-export (an alias or an ext method) -
  add it.
- Do NOT enable glam's `serde` or `glam-assert` (all-builds) features. Keep the
  serde DTO (`TyVector3Serde {x,y,z}`); the JSON wire stays byte-identical.
- Repo style: edition 2024, consolidated nested `use`, one public item per file
  in snake_case, ONE extension trait per file named for the trait, doc comments on
  public items, 80-col ASCII comments, no em dashes. Composites get an `IDENTITY`
  associated const (glam naming), not an `identity()` fn. `cargo fmt --all` +
  `cargo clippy --workspace --all-targets -- -D warnings` + `cargo test` before
  staging; the pre-commit hook enforces fmt + clippy.
- Verify sub-parts with `cargo check -p ty-math` (S1-S2), then per consumer crate
  (S3-S7), then `cargo check --workspace` (S8).

## Phase 1: ty-math foundation

- [ ] **S1. Add the `glam` dependency.** Workspace has no `[workspace.
      dependencies]` table (palette lives only in ty-math/Cargo.toml), so add to
      `ty-math/Cargo.toml`: `glam = { version = "0.33", default-features = false,
      features = ["std", "float-types", "integer-types", "debug-glam-assert"] }`.
      Bump ty-math `0.1.9 -> 0.1.10`. VERIFY: `glam` resolves to the 0.33 line and
      `use glam::{DVec2, DVec3, DVec4, Vec2, Vec3, Vec4, IVec3, UVec3, DQuat, Quat,
      DMat3, DMat4, Mat4, EulerRot};` all import (temporary scratch check).
      `cargo check -p ty-math` still green (dep unused yet). Record the resolved
      glam version in the decision log.

- [ ] **S2. Flip the math types to aliases + add ext traits + rewrite composites.**
      The atomic step; ty-math compiles on its own, consumers break until Phase 2.
      - Replace the vector/quat/matrix base + alias files with concrete aliases
        ([api-map](reference/glam-api-map.md) "Type aliases"): `TyVector2/3/4`
        (bare = `DVec*`) and `*F32/*F64` (+ V3 `*I32/*U32`); `TyQuaternion` (bare
        = `DQuat`) + `*F32/*F64`; `TyMatrix4x4` (bare = `DMat4`) + `*F32/*F64`.
        Keep the doc comments. Delete the old generic structs and their
        `impl_ty_*_float!`/`_int!` macros.
      - Delete `array_conversions.rs` and its re-export (glam has the array/slice
        conversions natively).
      - Add the extension traits, one trait per file, re-exported from lib.rs
        (api-map EXT rows only - the residue is small after fail-fast):
        - `TyVector3Ext` (on `DVec3`/`Vec3`, and int families where it
          type-checks): `triangle_normal`, `zup_to_yup`, `yup_to_zup`, `quantize`,
          `catmull_rom_position/tangent`. DROP the unused trivia
          (`to_pure_quaternion`, `rotation_around`, `to_scale`, `from_x/y/z`,
          cosine `is_approximately_equal`, `is_normalized_approximately_equal`,
          `rotate_towards`) unless a caller is found; glam covers or inlines them.
        - `TyQuaternionExt` (on `DQuat`/`Quat`): `to_euler_radians`
          (`to_euler(EulerRot::XYZEx)` wrapped so `EulerRot` never leaks),
          `from_rotation_matrix` (normalizes cols = strips scale, then delegates to
          `Quat::from_rotation_axes`; keep `None`-on-degenerate only if `from_mvox`
          consumes it, else assert), `from_basis_vectors`/`from_right_forward`/
          `from_right_up` (delegate to `Quat::from_rotation_axes` - glam's
          algorithm, drop ty's trace-branch), `rotate_extents_abs`
          (`DMat3::from_quat(q)` + per-column `.abs()` then `* extents`),
          `canonicalized`. Do NOT wrap `from_axis_angle`/`inverse`/`is_normalized` -
          use glam's directly.
      - Rewrite the composites as concrete glam-backed structs with an `IDENTITY`
        const (api-map "Composites"): `TyBounds` (f32 + f64, `Vec3`/`DVec3` fields,
        bodies onto glam `min`/`max`/`+`/`-`/`*`); `TyTransform` (f64,
        `DVec3`+`DQuat`+`DVec3`, keep the lossy `compose`); `TyUniformTrs` (f64);
        `TyPose` (f64, hand-rolled - NOT glamx `DPose3` - `calculate_relative_pose`
        via glam `inverse` + `*`). Keep `TyFloatExt` verbatim.
      - lib.rs: update the `mod`/`pub use` list (drop `array_conversions`, add the
        ext-trait modules); no `glam` symbol is re-exported by name.
      - Port ty-math's own unit tests onto the new API (`length`/`normalize`/`*`/
        `from_xyzw`/`IDENTITY`/`as_*`); DELETE tests for behaviors now owned by
        glam (the macro's array round-trips) or dropped (`from_axis_angle`
        zero-guard, `from_x/y/z`, the non-unit `inverse`).
      - Gate: `cargo test -p ty-math` green. `cargo check -p ty-math-serde` after
        confirming the DTO body compiles (may defer to S7).

## Phase 2: migrate consumers (workspace red until S8)

- [ ] **S3. voxsmith/convert (Cluster A, ~14 files).** `magnitude`->`length`;
      `componentwise_multiply(&o)`->`* o`; `to_f64/to_i32/to_u32`->`as_dvec3/
      as_ivec3/as_uvec3`; `from_column_arrays(a)`->`from_cols_array_2d(&a)`;
      `identity()`->`IDENTITY`; `.transform_point`->`.transform_point3`. At each
      `from_axis_angle` site pass a normalized axis (`axis.normalize()` unless
      provably unit; the assert flags misses). `from_rotation_matrix` -> ext.
      `zup_to_yup`/`yup_to_zup`/`triangle_normal` -> `TyVector3Ext`. Leave every
      foreign `[f64;3]`/`voxel.*`. Gate: `cargo test -p voxsmith` (runs after S4
      if the lib stays red).

- [ ] **S4. voxsmith/internal + reduce_palette (Cluster B, ~15 files).** Struct
      field types are pure alias swaps; `.cross(&o)`->`.cross(o)`, `.dot(&o)`->
      `.dot(o)`, `component(i)`->`[i]`, `magnitude`->`length`, `.quantize(..)`
      (VECTOR) -> `TyVector3Ext::quantize`, `zup_to_yup` -> `TyVector3Ext`;
      `box_local.center/.extents` stay `TyBounds` fields. Gate: `cargo test -p
      voxsmith` green (default + `--features gltf`).

- [ ] **S5. vxl + voxcore (Cluster C, 5 files).** Rewrite the ONE generic
      `vxl/voxelize.rs:91 TyVector3<f64>` -> `TyVector3F64`. `.compose`/
      `.transform_point`/`.to_euler_radians` (ext) on the `TyTransformF64`/
      `TyQuaternionF64` in `hierarchy_show.rs`. voxcore `vox_object.rs` public
      `TyVector3U32/I32` sigs + `.x/.y/.z` reads + `as u64` (glam fields survive);
      `vox_main.rs` `is_normalized(tol)` (glam's `is_normalized()` if the fixed
      threshold is acceptable, else a distinctly-named tol ext - confirm the
      caller's need), transform field reads; `vox_hierarchy_node.rs` `pub
      transform: TyTransformF64` field. Gate: `cargo test -p vxl -p voxcore` green.

- [ ] **S6. tyt-fbx + tyt-injection + tyt-vmax (Cluster D, ~10 files).** tyt-fbx
      `create_point_cloud.rs`: `.magnitude()`->`.length()`, `.cross(&v)`/`.dot(&v)`
      -> by value, `.x/.y/.z` reads AND the `min.x = v.x` write (glam fields
      writable). The public `serialize_points_and_colors_json` / `MeshWithUvs`
      sigs compile once bare `TyVector3 = DVec3`. tyt-injection: the
      `TyVector3Serde` bridge is unchanged; confirm the pinned `{x,y,z}` JSON test
      stays byte-identical. tyt-vmax `dependencies_impl.rs`: `from_axis_angle`
      (normalize the axis), `to_euler_radians().to_array()` (ext), leave the
      `[f64;N]` codec arrays. Gate: `cargo test -p tyt-fbx -p tyt-injection -p
      tyt-vmax` green.

- [ ] **S7. treegrid + ty-math-serde confirm.** treegrid only uses `TyFloatExt`
      as a generic bound - confirm it still compiles unchanged. Finish the
      ty-math-serde DTO check from S2 if deferred: `From<TyVector3>` reads
      `.x/.y/.z` on `DVec3`, `From<TyVector3Serde>` is `DVec3::new(..)`, both
      unchanged. Gate: `cargo test -p treegrid -p ty-math-serde` green.

## Phase 3: sweep, verify, commit

- [ ] **S8. Workspace green + re-baseline.** `cargo check --workspace`, `cargo fmt
      --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test
      --workspace`. Confirm the only test changes are intended (deleted
      macro/dropped-method tests, Debug-format re-baselines, any near-gimbal euler
      re-baseline). Confirm the `debug-glam-assert` build has NO axis/quat
      normalization panics left (every `from_axis_angle`/`inverse` input is unit).
      Confirm NO external wire moved: the `TyVector3` JSON, and vmax/voxj/goxl/qb
      array/hex bytes, all identical (no golden/fixture file changed). Grep that no
      consumer names `glam` and no dangling reference to a removed method/type/
      `array_conversions` remains. Confirm the quaternion q->matrix->q round-trip
      and the `to_euler_radians` XYZEx convention with a focused test.

- [ ] **S9. One clean commit.** Stage everything (code + these checklist ticks +
      the decision log). Present the staged diff for owner review; commit once,
      directly on main, with a Conventional Commits subject and the
      `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`
      trailer, only on explicit approval.

Gate (whole plan): workspace green, clippy clean, every external wire identical,
`glam` named only inside ty-math, ty-math no longer maintains its own
vector/quaternion/matrix math (arithmetic, dot/cross, length, arrays, the
float/int macros, `array_conversions`, the axis/matrix-to-quaternion algorithms) -
only the extension traits (the small tyt-specific residue), the four composite
structs, `TyFloatExt`, and the serde DTO remain hand-written.
