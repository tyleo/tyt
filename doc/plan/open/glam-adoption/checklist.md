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

- [x] **S1. Add the `glam` dependency.** Workspace has no `[workspace.
      dependencies]` table (palette lives only in ty-math/Cargo.toml), so add to
      `ty-math/Cargo.toml`: `glam = { version = "0.33", default-features = false,
      features = ["std", "float-types", "integer-types", "debug-glam-assert"] }`.
      Bump ty-math `0.1.9 -> 0.1.10`. VERIFY: `glam` resolves to the 0.33 line and
      `use glam::{DVec2, DVec3, DVec4, Vec2, Vec3, Vec4, IVec3, UVec3, DQuat, Quat,
      DMat3, DMat4, Mat4, EulerRot};` all import (temporary scratch check).
      `cargo check -p ty-math` still green (dep unused yet). Record the resolved
      glam version in the decision log.
      DONE: resolved `glam v0.33.2`; the scratch probe additionally confirmed
      every api-map symbol for S2+ compiles against 0.33.2. See
      [decision log](reference/implementation-decisions.md#s1-glam-dependency).

- [x] **S2. Flip the math types to aliases + add ext traits + rewrite composites.**
      The atomic step; ty-math compiles on its own, consumers break until Phase 2.
      DONE: `cargo test -p ty-math` green (46 tests), clippy clean, `ty-math-serde`
      compiles. Owner override this session - KEEP every method (ported to glam),
      not just the api-map's EXT set: the "DROP trivia" below became ext methods,
      and `TyVector2Ext` (`to_vector3`) + `TyMatrix4x4Ext` (`get`) were added.
      `rotate_towards` -> `rotation_towards` (glam has a colliding inherent method).
      `is_normalized(tol)` deferred to S5. `to_euler_radians` matches the old
      Tait-Bryan formula to 1e-9 (test). A 4-agent adversarial audit found no
      semantic drift. See [decision log](reference/implementation-decisions.md#s2-ty-math-flip-aliases--ext-traits--composites).
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

- [x] **S3. voxsmith/convert (Cluster A, ~14 files).** `magnitude`->`length`;
      `componentwise_multiply(&o)`->`* o`; `to_f64/to_i32/to_u32`->`as_dvec3/
      as_ivec3/as_uvec3`; `from_column_arrays(a)`->`from_cols_array_2d(&a)`;
      `identity()`->`IDENTITY`; `.transform_point`->`.transform_point3`. At each
      `from_axis_angle` site pass a normalized axis (`axis.normalize()` unless
      provably unit; the assert flags misses). `from_rotation_matrix` -> ext.
      `zup_to_yup`/`yup_to_zup`/`triangle_normal` -> `TyVector3Ext`. Leave every
      foreign `[f64;3]`/`voxel.*`. Gate: `cargo test -p voxsmith` (runs after S4
      if the lib stays red).
      DONE: goxl + mvox + vmax + qbcl + gltf migrated; voxelize is alias-only (zero
      edits). No error in any `convert/` file via `cargo check -p voxsmith`
      (+ `--features gltf`); the rest of voxsmith stays red until S4.

- [x] **S4. voxsmith/internal + reduce_palette (Cluster B, ~15 files).** Struct
      field types are pure alias swaps; `.cross(&o)`->`.cross(o)`, `.dot(&o)`->
      `.dot(o)`, `component(i)`->`[i]`, `magnitude`->`length`, `.quantize(..)`
      (VECTOR) -> `TyVector3Ext::quantize`, `zup_to_yup` -> `TyVector3Ext`;
      `box_local.center/.extents` stay `TyBounds` fields. Gate: `cargo test -p
      voxsmith` green (default + `--features gltf`).
      DONE: `internal/mesh/` + `internal/gltf/*` + `internal/grid.rs` +
      `internal/voxj/*` + `internal/vmax/write_vmax.rs` + `reduce_palette.rs`, plus
      `convert/mesh/object_to_mesh_geometry.rs` (NOT in the census - found by
      compile). `assign_op_pattern` clippy now fires (glam has `AddAssign`): two
      `*x = *x + y` -> `+=` in reduce_palette. voxsmith green + clippy clean, both
      builds.

- [x] **S5. vxl + voxcore (Cluster C, 5 files).** Rewrite the ONE generic
      `vxl/voxelize.rs:91 TyVector3<f64>` -> `TyVector3F64`. `.compose`/
      `.transform_point`/`.to_euler_radians` (ext) on the `TyTransformF64`/
      `TyQuaternionF64` in `hierarchy_show.rs`. voxcore `vox_object.rs` public
      `TyVector3U32/I32` sigs + `.x/.y/.z` reads + `as u64` (glam fields survive);
      `vox_main.rs` `is_normalized(tol)` (glam's `is_normalized()` if the fixed
      threshold is acceptable, else a distinctly-named tol ext - confirm the
      caller's need), transform field reads; `vox_hierarchy_node.rs` `pub
      transform: TyTransformF64` field. Gate: `cargo test -p vxl -p voxcore` green.
      DONE: voxcore (S5 earlier) + vxl. `.compose(&x)` and `.transform_point(pt)`
      needed NO change (composite keeps `compose(&self, &Self)` and
      `transform_point(&self, DVec3)`). `voxelize.rs` main import `TyVector3` ->
      `TyVector3F64` (tests keep bare `TyVector3::new`). Euler `-0.0` signed-zero
      re-baseline: fold via `+ ZERO` in `format_vec3` so no `-0.00` renders (golden
      unchanged). vxl green + clippy clean (fixed an `assign_op_pattern`).

- [x] **S6. tyt-fbx + tyt-injection + tyt-vmax (Cluster D, ~10 files).** tyt-fbx
      `create_point_cloud.rs`: `.magnitude()`->`.length()`, `.cross(&v)`/`.dot(&v)`
      -> by value, `.x/.y/.z` reads AND the `min.x = v.x` write (glam fields
      writable). The public `serialize_points_and_colors_json` / `MeshWithUvs`
      sigs compile once bare `TyVector3 = DVec3`. tyt-injection: the
      `TyVector3Serde` bridge is unchanged; confirm the pinned `{x,y,z}` JSON test
      stays byte-identical. tyt-vmax `dependencies_impl.rs`: `from_axis_angle`
      (normalize the axis), `to_euler_radians().to_array()` (ext), leave the
      `[f64;N]` codec arrays. Gate: `cargo test -p tyt-fbx -p tyt-injection -p
      tyt-vmax` green.
      DONE: tyt-fbx `create_point_cloud.rs` by-value `dot`/`cross` + `magnitude`->
      `length`. tyt-vmax `local_transform` axis-angle guarded like the vmax
      converters (`ZERO_LENGTH_TOLERANCE` + normalize), `to_euler_radians` via ext.
      tyt-injection `{x,y,z}` JSON test still byte-identical (unchanged).

- [x] **S7. treegrid + ty-math-serde confirm.** treegrid only uses `TyFloatExt`
      as a generic bound - confirm it still compiles unchanged. Finish the
      ty-math-serde DTO check from S2 if deferred: `From<TyVector3>` reads
      `.x/.y/.z` on `DVec3`, `From<TyVector3Serde>` is `DVec3::new(..)`, both
      unchanged. Gate: `cargo test -p treegrid -p ty-math-serde` green.
      DONE: both compile + test unchanged under the whole-workspace build; no edits
      needed (confirmed by `cargo test --workspace`).

## Phase 3: sweep, verify, commit

- [x] **S8. Workspace green + re-baseline.** `cargo check --workspace`, `cargo fmt
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
      DONE: `cargo check --workspace` + `cargo fmt --all --check` + `cargo clippy
      --workspace --all-targets -- -D warnings` + `cargo test --workspace` all
      green, plus `-p voxsmith --features gltf` check/clippy/test (212 tests). Only
      test change: no golden/fixture file changed (git shows only `.rs` + these
      docs); the euler `-0.0` was absorbed in `format_vec3`, not a golden edit.
      `debug-glam-assert` on in tests -> zero normalization panics. No consumer
      names `glam` (only ty-math); no dangling old-API grep hit. q->M->q and XYZEx
      euler pinned by the S2 ty-math tests (green).

- [x] **S9. One clean commit.** Stage everything (code + these checklist ticks +
      the decision log). Present the staged diff for owner review; commit once,
      directly on main, with a Conventional Commits subject and the
      `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`
      trailer, only on explicit approval.
      DONE: rebased onto `origin/main` (10 upstream commits, fast-forward) via
      stash/pull/pop so the migration sits on top; one conflict in vxl
      `hierarchy_show.rs` (kept the `treeselect` import + added `TyQuaternionExt`).
      Workspace green again (check + fmt + clippy + test, plus voxsmith gltf);
      committed on main as `refactor(ty-math)!: back the math types with the glam
      crate`.

Gate (whole plan): workspace green, clippy clean, every external wire identical,
`glam` named only inside ty-math, ty-math no longer maintains its own
vector/quaternion/matrix math (arithmetic, dot/cross, length, arrays, the
float/int macros, `array_conversions`, the axis/matrix-to-quaternion algorithms) -
only the extension traits (the small tyt-specific residue), the four composite
structs, `TyFloatExt`, and the serde DTO remain hand-written.
