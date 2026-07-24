# Implementation decisions

Running log of non-obvious code-level decisions made while executing the
[checklist](../checklist.md). Created at S1; appended each session. Facts here are
what was verified at the keyboard, superseding the reference docs where they
differ.

## S1: glam dependency

- **Resolved version: `glam v0.33.2`.** `version = "0.33"` locked to `0.33.2`
  (crates.io latest, 2026-06-28) and written to the workspace `Cargo.lock`. Zero
  transitive deps added (glam pulls nothing with these features).
- **Features:** `["std", "float-types", "integer-types", "debug-glam-assert"]`,
  `default-features = false`. `serde` and `glam-assert` (all-builds) stay OFF, per
  plan. `size-types` dropped (unused).
- **Version bump:** ty-math `0.1.9 -> 0.1.10` (stays 0.1.x-patch so `^0.1` carets
  + workspace `[patch.crates-io]` keep resolving; mirrors palette 0.1.8 -> 0.1.9).
- **`cargo check -p ty-math` green with the dep unused** (no unused-crate lint on
  by default, so no gate noise before S2 references it).
- **API drift check (compile-only scratch, then deleted).** Every glam symbol the
  [api-map](glam-api-map.md) relies on for S2+ was confirmed present with the
  expected shape in 0.33.2:
  - Types: `DVec2/3/4`, `Vec2/3/4`, `IVec3`, `UVec3`, `DQuat`, `Quat`, `DMat3`,
    `DMat4`, `Mat4`, `EulerRot`.
  - Vector: `new`/`splat`/`from_array`/`to_array`/`from_slice`/`write_to_slice`,
    `v[i]`, `dot(o)`/`cross(o)` by value, `*` and `/` Hadamard, `length`/
    `length_squared`/`normalize`/`min`/`max`/`abs`/`round`/`lerp`, `ZERO`/`ONE`/
    `INFINITY`/`NEG_INFINITY`/`X`/`Y`/`Z`/`NEG_X`, `as_ivec3`/`as_uvec3`/
    `as_dvec3`, `perp_dot` (V2), `extend`/`truncate`.
  - Quaternion: `IDENTITY`/`from_xyzw`/`length`/`length_squared`/`normalize`/
    `conjugate`/`dot`/`xyz`, `from_axis_angle`/`inverse`/`is_normalized`/`slerp`/
    `from_rotation_arc`, `q * Vec3` / `q * q` / `q * f64` (no `f64 * q`),
    `to_euler(EulerRot::XYZEx)`, `from_rotation_axes`, `Neg`.
  - Matrix: `from_cols_array_2d(&arr)` (leading `&` required) / `to_cols_array_2d`
    / `IDENTITY` / `from_quat` / `x_axis` / `col(i)` / `from_cols` /
    `transform_point3` / `transform_vector3` / `determinant` / `inverse` /
    `transpose`; `DMat3::from_quat` + per-column `.abs()` fold for
    `rotate_extents_abs`.

## S2: ty-math flip (aliases + ext traits + composites)

Landed; `cargo test -p ty-math` green (46 tests), clippy clean, `ty-math-serde`
compiles unchanged.

- **Owner override (mid-session): keep every method, do not drop.** The plan's
  DROP list (0-caller trivia) was going to be deleted; the owner asked instead to
  KEEP all methods, ported to their best glam-compatible form. Types may still be
  dropped. So the "DROP" trivia became extension methods (see below). Renames
  forced by a glam name-collision are accepted.
- **`rotate_towards` -> `rotation_towards` (forced rename).** glam has an
  INHERENT `DVec3::rotate_towards(self, rhs, max_angle) -> Vec3` (rotates a vector
  toward another by a capped angle) that shadows any same-named trait method in
  call syntax. The old ty `rotate_towards` returned the shortest-arc *rotation*
  quaternion, so it is kept as `TyVector3Ext::rotation_towards` (= glam
  `from_rotation_arc`). This was the only inherent-name collision.
- **Extension traits shipped (one per file, all re-exported):**
  - `TyVector2Ext` (on `DVec2`/`Vec2`, assoc `Vector3`): `to_vector3` (= glam
    `extend(0.0)`, kept as a named convenience per owner).
  - `TyVector3Ext` (on `DVec3`/`Vec3`, assoc `Scalar` + `Quaternion`):
    `triangle_normal`, `zup_to_yup`, `yup_to_zup`, `quantize`,
    `catmull_rom_position/tangent`, plus the kept-per-owner
    `from_x/from_y/from_z`, `to_scale`, `to_pure_quaternion`, `rotation_around`
    (normalizes the axis), `rotation_towards`, `is_approximately_equal` (cosine),
    `is_normalized_approximately_equal`.
  - `TyQuaternionExt` (on `TyQuaternionF64`/`DQuat` only - census confirms zero
    f32-quaternion consumers): `to_euler_radians`, `from_rotation_matrix`
    (Option), `from_basis_vectors`, `from_right_forward`, `from_right_up`,
    `rotate_extents_abs`, `canonicalized`, plus kept-per-owner `rotate_around_axis`
    (normalizes the axis) and `is_approximately_equal` (abs-dot).
  - `TyMatrix4x4Ext` (on `TyMatrix4x4F64`/`DMat4` only): `get(row, col)` =
    `self.col(col)[row]` (kept per owner; glam has no element accessor).
- **`from_rotation_matrix` keeps `Option` (None on degenerate).** The one consumer
  (`voxsmith .../from_mvox_file.rs:378`) does `.expect(...)`, so the Option is
  consumed; kept rather than asserted. Delegates to glam `from_rotation_axes` after
  normalizing the three columns.
- **`to_euler_radians` uses glam `to_euler(EulerRot::XYZEx)` and MATCHES the old
  Tait-Bryan formula.** A test (`to_euler_radians_matches_the_tait_bryan_reference`)
  compares glam's output to the old atan2/asin formula on four arbitrary rotations
  to 1e-9 and passes, and the single-axis reads (roll=x/pitch=y/yaw=z) hold. So the
  vmax euler wire that reads these angles will not move (final confirm at S6/S8).
- **`rotate_extents_abs` lifted to `DMat3::from_quat(q)` + per-column `.abs()`.**
  Verified equal to the old row-form abs fold by test (quarter-turn swaps x/y
  extents; identity is a no-op).
- **Composites are concrete glam-backed structs with an `IDENTITY` const + manual
  `Default = IDENTITY`.** `TyBounds` is an `impl_ty_bounds!` macro pair (f32 + f64);
  `TyTransform`/`TyPose`/`TyUniformTrs` are single f64 structs (bare name = struct,
  `*F64` = alias). Manual `Default` is required (scale must be `ONE`/`1.0`, which a
  derived `Default` would zero); pose could derive but is kept manual for symmetry.
- **Types dropped (owner: types may go).** Deleted `array_conversions.rs` (glam has
  the array/slice conversions) and the three unused f32 composite aliases
  `TyTransformF32`/`TyPoseF32`/`TyUniformTrsF32` (0 refs; the api-map designs these
  three as f64-only). `TyBoundsF32` is kept (2 refs). All f32/i32/u32 vector,
  quaternion, and matrix aliases are kept (cheap, parity).
- **`TyColorToVector3` narrowed to `f64` (non-generic).** The generic `TyVector3<T>`
  return cannot survive concrete aliases; the sole consumer (`reduce_palette`) uses
  only `TySrgbF64`/`TyOklabColorF64`/`TyCielabColorF64`, so the trait is now
  non-generic returning `TyVector3F64`. The unused f32 color->vector path is gone.
- **Deferred to S5:** quaternion `is_normalized(tolerance)` (custom-tolerance
  variant). glam's inherent `is_normalized()` has a fixed tolerance and the name
  collides; `vox_main` passes a tolerance, so a distinctly-named ext
  (`is_normalized_within`) will be added when S5 confirms that caller's need.
- **f32 `round`:** no ty-math code rounds an f32 vector, so the half-to-even vs
  half-away-from-zero delta does not arise here (re-confirm at consumer sites).
- **Debug format:** composite structs now print as `TyBoundsF64`/`TyBoundsF32`/
  `TyTransform`/... and vectors as glam `DVec3(..)`; no ty-math golden pins these.

## S3-S7: consumer migration

### S3: voxsmith/convert (in progress)

- **goxl converter done** (`from_goxl_file.rs` + `to_goxl_file.rs`). Three edits,
  all DIRECT api-map renames:
  - `TyQuaternionF64::identity()` -> `TyQuaternionF64::IDENTITY` (test helper);
    glam `DQuat` has no `identity()` fn, only the `IDENTITY` const.
  - `to_i32()` -> `as_ivec3()` at two sites:
    `node.transform.position.round()` (DVec3 -> IVec3) and
    `object.voxel_position(..)` (UVec3 -> IVec3).
  - Return/field types confirmed at the keyboard: `VoxObject::voxel_position ->
    Option<TyVector3U32>`, `VoxHierarchyNode.transform.position: DVec3`.
    `round().as_ivec3()` equals the old `round().to_i32()` (round yields an
    integral f64, so the trunc-toward-zero cast is exact).
  - No other math site in either file; the `id.to_u32()` calls are branded_id
    `U32Id`, not a ty-math vector cast, left alone.
- **Blocker: voxsmith is not compile-verifiable until voxcore (S5) lands.**
  voxsmith depends on voxcore, which is RED (3 S5 errors: `component_min_with`/
  `component_max_with` -> `min`/`max` in `vox_object.rs`, and the deferred
  `is_normalized(tol)` in `vox_main.rs`). `cargo check -p voxsmith` stops at the
  voxcore dependency before it reaches voxsmith's own files, so `cargo check -p
  voxsmith` cannot gate an S3/S4 chunk. Those chunks are verified by inspection
  against the api-map until voxcore compiles. RECOMMEND migrating voxcore (the
  leaf dependency) before/early in the consumer phase so every downstream chunk
  becomes compile-verifiable; the atomic single-commit strategy is unaffected by
  chunk order.
- Verified: voxcore migrated next (S5), so `cargo check -p voxsmith` now clears
  the voxcore dep and reports 0 errors in goxl.
- **mvox converter done** (`from_mvox_file.rs` + `to_mvox_file.rs`), 0 errors via
  `cargo check -p voxsmith`. Edits: `to_f64` -> `as_dvec3`;
  `from_column_arrays([..])` -> `from_cols_array_2d(&[..])`; `from_rotation_matrix`
  now via `TyQuaternionExt` (returns Option, `.expect`ed as before);
  `.dot(&col.cross(&col))` -> by value; two `to_i32` -> `as_ivec3`; test
  `identity()` -> `IDENTITY`.
- **vmax converter done** (`from_vmax_file.rs` logic + tests; `to_vmax_file.rs` is
  a thin wrapper over `write_vmax` (S4), so its sites are test-only). Edits:
  `component_min_with` -> `min`; `to_f64`/`to_i32` -> `as_dvec3`/`as_ivec3`;
  `componentwise_multiply(&scale)` -> `* scale`; `rotation.rotate(v)` ->
  `rotation * v`; test `identity()` -> `IDENTITY`.
- **Fail-fast axis guard at `axis_angle`.** VMax stores an unrotated object as
  `rotation: [0, 0, 0, 0]` (the serde default; a test feeds it), so a zero axis
  reaches `from_axis_angle`. Guard: `axis.length() < 1e-12 -> IDENTITY`, else
  `from_axis_angle(axis.normalize(), angle)`. Reproduces the old ctor's
  degenerate-axis identity; zero angle needs no guard (glam yields IDENTITY).
  Explicit per-site handling, not a silent wrapper of the strict method.
  `to_vmax:1073` keeps its literal unit axis `(0, 0, 1)`.
- **qbcl converters done** (`from_qb`/`from_qbt`/`from_qbcl` + `to_qbcl`), 4 edits,
  all DIRECT vector casts:
  - The three `from_*` `translation()` helpers:
    `TyVector3I32::from_array(position).to_f64()` -> `.as_dvec3()` (IVec3 -> DVec3).
  - `to_qbcl_file.rs:297` `parent + position.round().to_i32()` -> `.as_ivec3()`
    (DVec3 -> IVec3, the summed-world hierarchy fold). `to_qb`/`to_qbt` have NO
    world/`to_i32` logic despite the census listing all three - only `to_qbcl`
    builds the summed scene tree; the other two carry a single object each.
  - Left alone (not vector casts): every `id.to_u32()`/`object_id.to_u32()` is a
    branded `U32Id`, not a ty-math cast; `bounds.to_array()` (`object.bounds()` ->
    `TyVector3U32`) and `world.to_array()` (`TyVector3I32`) are glam-native;
    `TyVector3U32/I32::new`/`from_array` and `TyTransformF64::from_translation`/
    `default` all resolve on the aliases with no change.
  - Verified: `cargo check -p voxsmith` reports no error in any `qbcl/` file (the
    remaining 27 errors are all S4 `internal/`/`reduce_palette`, still pending).
- **gltf converter done** (`from_gltf_bytes.rs`), 4 edits, behind `--features gltf`:
  - `TyMatrix4x4F64::from_column_arrays(m)` -> `from_cols_array_2d(&m)` (leading `&`;
    the node's `[[f64;4];4]` column-major matrix, `[col][row]` layout matches).
  - `TyMatrix4x4F64::identity()` -> `IDENTITY`. This is the `::`-associated-fn form;
    the `.identity()` site grep missed it and the compiler flagged it. (Watch for
    `::identity()` in later clusters, not just `.identity()`.)
  - `world.transform_point(v)` -> `transform_point3(v)` (DMat4 point transform, w=1).
  - `world.yup_to_zup()` moved to the ext trait, so `use ty_math::TyVector3Ext` was
    added to the import block.
  - The other 5 `convert/gltf` files (object_to_*, material_atlas) hold only
    test-only `TyVector3U32::new` (glam-native) - no edits.
- **voxelize alias-only, zero edits.** `mesh.rs`/`voxelize_mesh.rs` use only
  `TyBoundsF64::from_points`, `TyVector3F64::{ZERO,new}`, bare `TyVector3::splat`,
  `TyVector3U32` params, and `.x/.y/.z` reads - all glam-native or kept-method.
- **Cluster A / S3 complete.** No error in any `convert/` file via `cargo check -p
  voxsmith` and `cargo check -p voxsmith --features gltf`; the remaining voxsmith
  errors are all S4 (`internal/`, `reduce_palette`).

### S5: voxcore (vxl pending)

- voxcore green, 81 tests pass, clippy clean.
- Added the S2-deferred `TyQuaternionExt::is_normalized_within(tol)` =
  `(length_squared - 1).abs() <= tol`. glam's fixed `is_normalized` (~2e-4) is
  too loose for the caller's 1e-6 (`vox_main` validate).
- Mechanical: `component_min_with`/`component_max_with` -> `min`/`max` (UVec3
  live_extent); test `TyQuaternion::new` -> `from_xyzw`.
- The vxl half of S5 is still pending.

### S4: voxsmith/internal + reduce_palette (in progress)

- **`internal/mesh/` geometry cluster done** (4 files), all DIRECT renames:
  - `grid_space.rs`: `.componentwise_divide(&self.size)` -> `/ self.size`;
    `offset.componentwise_multiply(&self.size)` -> `offset * self.size`
    (glam `*`/`/` are Hadamard; `*` binds tighter than `+`, so `self.min + offset *
    self.size` groups correctly).
  - `triangle_bounds.rs`: `component_min_with(&p)`/`component_max_with(&p)` ->
    `.min(p)`/`.max(p)` (glam pairwise component min/max, matches).
  - `triangle_box_overlap.rs`: by-value `cross`/`dot` - `axis.cross(edge)` ->
    `cross(*edge)` (edge is `&TyVector3F64` from `for edge in &edges`, so deref, not
    just drop `&`); `edges[0].cross(&edges[1])` -> `cross(edges[1])`;
    `axis.dot(&v[i])` -> `dot(v[i])`.
  - `sample_material.rs`: barycentric `v0.dot(&v0)` ... -> by-value `dot`.
  - The other `internal/mesh/*` files are struct-field-only alias swaps (no edit).
  - `internal/mesh/` is `_mesh`-gated; verified clean via `cargo check -p voxsmith
    --features gltf` (no error anywhere under `internal/mesh/`).
- **Watch (from the S4 grep):** the by-value `cross`/`dot` trap has two arg shapes -
  `.cross(&x)` (drop the `&`) AND `.cross(loop_ref)` where the binding is already a
  `&Vec` (add a `*`). A `.cross(&` grep misses the second; read the binding type.
- **`internal/gltf/` document pair done** (`object_to_gltf_document.rs` +
  `material_document.rs`, near-identical), all DIRECT:
  - `component_min_with(&point)`/`component_max_with(&point)` -> `.min(point)`/
    `.max(point)` (the AABB fold over baked positions).
  - `p.zup_to_yup()` / `n.zup_to_yup()` are ext methods, so each file's
    `use ty_math::TyVector3F32` became `{TyVector3Ext, TyVector3F32}`. Confirms
    `TyVector3Ext` covers the f32 `Vec3` form (`p: TyVector3F32`), not just f64.
  - `p.zup_to_yup() * scale` keeps glam `Vec3 * f32` (scalar mul) unchanged.
  - gltf-gated; clean via `cargo check -p voxsmith --features gltf` (no error
    anywhere under `internal/gltf/`, so `bake_atlas.rs` etc. need no edit either).
- **`internal/grid.rs` + `internal/voxj/*` done** (default build), all DIRECT:
  - `grid.rs`: `min.to_i32()` -> `min.as_ivec3()` (x2, `origin + min` and `-min`;
    unary `-` binds looser than the method call, so `-min.as_ivec3()` negates the
    IVec3); `(p.to_i32() + offset).to_u32()` -> `(p.as_ivec3() + offset).as_uvec3()`.
  - `voxj_decoded_object_from_vox_object.rs:73`: `min.to_i32()` -> `.as_ivec3()`.
    Its `:37`/`:63` `.to_u32()` are branded material ids and `:55`
    `(position - min).to_array()` is UVec3-native - all left alone.
  - `vox_hierarchy_node_from_voxj_hierarchy_node.rs:75`: `TyQuaternion::new(x,y,z,w)`
    -> `from_xyzw` (glam ctor). The surrounding code divides each component by the
    magnitude, so the quat is explicitly unit - no debug-glam-assert risk. The `let
    magnitude = (..).sqrt()` above is a SCALAR, not a vector `.magnitude()`.
  - All other voxj `.to_u32()` (write_voxj, voxj_palette, voxj_hierarchy_node) are
    branded `U32Id` -> `usize` index casts, left alone.
  - Clean via `cargo check -p voxsmith` (no error under `grid.rs`/`voxj/`).
- **`internal/vmax/write_vmax.rs` done.** `to_f64`->`as_dvec3` at the three
  `from_min_size` sites; `component_min_with`/`max_with`->`min`/`max`;
  `transform.rotation.rotate(v)`->`* v` (DQuat*DVec3, 4 sites); `from_axis_angle`
  guarded (see the const note). `transform.transform_point(pt)` needed NO change:
  it is the composite `TyTransform::transform_point(&self, DVec3)`, a kept method,
  not glam `DMat4::transform_point3`.
- **`reduce_palette.rs` done.** `component(axis)`->`[axis]` (glam `Index<usize>`);
  `.quantize(..)` via `TyVector3Ext` (import added); `magnitude`/`magnitude_squared`
  ->`length`/`length_squared`; `component_min_with`/`max_with`->`min`/`max`; two
  `*x = *x + y`->`+=` (clippy `assign_op_pattern`, now that glam has `AddAssign`).
- **New shared const `ty_math::ZERO_LENGTH_TOLERANCE = 1e-12` (owner request).**
  Three vmax sites decode a stored `[x,y,z,angle]` and guard a degenerate axis
  before glam's strict `from_axis_angle`; the magic `1e-12` is now this named,
  re-exported ty-math const (owner: recommended tolerances live in the math crate,
  per-topic, for consistency). File `zero_length_tolerance.rs` (no `ty_` prefix, to
  match the un-prefixed const name). Sites: `convert/vmax/from_vmax_file.rs`,
  `internal/vmax/write_vmax.rs`, `tyt-vmax/dependencies_impl.rs`.

### S5 (vxl half), S6, S7

- **vxl done.** `hierarchy_show.rs`: `object.origin()/.bounds()/live_extent().to_f64()`
  -> `as_dvec3`; `to_euler_radians` via `TyQuaternionExt` (import). `.compose(&x)`
  and `.transform_point(pt)` unchanged (composite keeps `&Self` / by-value point).
  `voxelize.rs:91` generic `TyVector3<f64>`->`TyVector3F64` (main import swapped;
  tests keep bare `TyVector3::new`). Test `from_axis_angle` at :1245 keeps its
  LITERAL unit axis `(0,0,1)` - no normalize. Euler signed-zero: `format_vec3`
  folds `-0.0`->`0.0` via `+ TyVector3F64::ZERO` so no `-0.00` renders and the
  hierarchy golden is unchanged; also fixed an `assign_op_pattern`
  (`rotation = rotation * k`->`*=`).
- **tyt-fbx done.** `create_point_cloud.rs`: by-value `dot`/`cross` (14 sites),
  `.magnitude()`->`.length()`. `min.x = v.x` writes survive (glam fields writable).
- **tyt-vmax done.** `local_transform` axis-angle guarded with
  `ZERO_LENGTH_TOLERANCE` + `normalize` (matches the converters); `.compose(&local)`
  unchanged; `to_euler_radians().to_array()` via ext.
- **tyt-injection / treegrid / ty-math-serde: no edits.** Compile + test unchanged
  under the whole-workspace build; the pinned `{x,y,z}` JSON test still passes.
- **Census gap found: `convert/mesh/object_to_mesh_geometry.rs`** was NOT in the
  consumer-census but has `TyVector3F32::triangle_normal(..)` (assoc fn on the ext
  trait - needs `TyVector3Ext` in scope, both the module and test imports) and
  `.dot(&normal)`/`.dot(&stored)` by-value. Found by compiling `--features gltf`,
  not by grep. Lesson: drive the tail to green by compiler, not census.

## S8: sweep + verify (done; S9 = commit, pending owner approval)

- Green across the board: `cargo check --workspace`; `cargo fmt --all --check`;
  `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`
  (0 failed); plus `-p voxsmith --features gltf` for check/clippy/test (212 tests).
- **No external wire moved.** `git status` shows only `.rs` + the two plan docs
  changed - no golden/fixture/`.json`/`.png` touched. The one behavioral shift
  (euler `-0.0`) was absorbed in the `format_vec3` display, not by editing a golden.
- **`debug-glam-assert` is on in tests and nothing panicked**, so every
  `from_axis_angle`/`normalize`/`inverse` input is unit/valid at run time.
- **`glam` is named only inside ty-math** (grep clean; a stray consumer comment that
  said "glam" was reworded). No dangling `componentwise`/`component_*_with`/
  `magnitude`/`to_f64`/`from_column_arrays`/`array_conversions` anywhere.
- q->M->q and the XYZEx euler convention stay pinned by the S2 ty-math tests (green).

## S9: one clean commit

- **Rebased onto upstream first.** Local `main` was 10 commits behind
  `origin/main` (treegrid/vxl/vmax refactors). Stashed the staged WIP, fast-forward
  pulled, popped the stash back on top so the migration lands above everything.
- **One conflict, in `vxl/src/implementation/hierarchy_show.rs`.** The upstream
  `treeselect` refactor added `use treeselect::TreeSelection;` while the migration
  added `TyQuaternionExt` to the `ty_math` import (for `.to_euler_radians()`). Kept
  both. Every other file three-way merged clean.
- Re-verified green post-rebase: `cargo check --workspace`; `cargo fmt --all
  --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test
  --workspace` (0 failed); plus `-p voxsmith --features gltf`. Staged set is still
  the same 70 files (no golden/wire file; `glam` named only inside ty-math).
- Committed once, directly on `main`, on explicit owner approval:
  `refactor(ty-math)!: back the math types with the glam crate`.
