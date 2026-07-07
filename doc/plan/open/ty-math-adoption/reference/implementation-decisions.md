# Implementation decisions

_Part of the [ty-math Adoption Plan](../README.md)._

Code-level decisions made while executing the [checklist](../checklist.md),
recorded as they land. The plan-level decisions and their rationale live in the
[README](README.md#decisions); this log is for the finer choices a reviewer of the
Rust would want explained, for example the confirmed name and signature of each new
`ty-math` method, whether a borderline vmax adoption was taken, and what the Track C
investigation found in each `internal/` file.

No work has landed yet. Add a section under the relevant track as its first chunk
lands.

## Track A: harmless cleanups

### A1: non-vmax converters (landed)

- The four-component `color_floats([u8; 4]) -> [f64; 4]` helpers became
  `TySrgbaColor::from_array(color).to_rgba().to_array()` inline, and both helpers
  (goxl, mvox) were deleted. `to_rgba` is the straight `/255.0` normalize with no
  transfer function, so the bytes are identical.
- The test hex parsers use `TySrgbaColor::from_hex(hex).expect("a valid hex
  color")` then `.to_rgba().to_array()` (four-component `srgba`, `rgb`) or
  `.to_rgba().to_vector3().to_array()` (three-component qbcl `srgb`, which drops
  alpha). `from_hex` strips the leading `#` itself, matching the old manual strip.
- The mvox `TyVector3F64::new(1.0, 1.0, 1.0)` at the main-code frame decode is
  `let mut scale` and gets `scale.x = -1.0` on a mirror, so it was left; only the
  non-mutated `placed_at` test-helper scale became `ONE`. The goxl and qbcl
  `placed_at` test helpers hold the same literal but sit outside this checklist
  item's named scope, so they were not touched.
- `object.bounds()` returns a `TyVector3`, so `[bounds.x, bounds.y, bounds.z]`
  folded to `bounds.to_array()` in the four qbcl `to_*` matrix rebuilds.
- `TyVector3::splat` and the `ONE`/`ZERO` constants are exact replacements for the
  repeated-argument `new` and `default` forms.

### A2: vmax converter (landed)

- vmax's `color_floats(&[u8; 4]) -> [f64; 4]` in `from_vmax_file.rs` became
  `TySrgbaColor::from_array(*color).to_rgba().to_array()`. The checklist wrote it
  as `.to_rgba()` alone, but the `Srgba` pool's `values` is `Vec<[f64; 4]>`, so the
  trailing `.to_array()` is required to type-check and is byte-identical.
- The `vec3([f64; 3]) -> TyVector3F64` helper became `TyVector3F64::from_array` at
  its three call sites (`object_transform` scale, `group_transform` position and
  scale); both helpers deleted.
- The `to_vmax_file.rs` test `color_floats(hex)` kept its name and signature; only
  the body switched to `TySrgbaColor::from_hex(hex).expect(..).to_rgba().to_array()`,
  which handles the `#RRGGBB` / `#RRGGBBAA` opaque-alpha default the same way.

## Track B: ty-math additions and adoption

### B1: ty-math additions (landed)

All seven landed with the Q3 names, each in its file's float-macro form (except
`triangle_normal`, which is generic, and `to_unorm8`, a trait method) with a doc
comment and a unit test in that file's `tests` module.

- `TyVector3::round(self) -> Self` and `TyVector3::to_i32(self) -> TyVector3<i32>`,
  both in the float macro. The cast landed as `to_i32` (not `as_i32` or a combined
  `round_to_i32`); `round` is half-away-from-zero, `to_i32` truncates toward zero,
  and `v.round().to_i32()` composes to nearest-integer rounding.
- `TyVector3::triangle_normal(a, b, c) -> Self`, an associated function in the
  generic `impl<T: Add + Copy + Mul + Sub>` block beside `cross`, computing
  `(b - a).cross(&(c - a))`.
- `TyTransform::from_translation(position) -> Self`, written as
  `Self { position, ..Self::default() }` so it inherits Default's identity rotation
  and unit scale.
- `TyBounds::from_points(impl IntoIterator<Item = TyVector3>) -> Option<Self>` and
  `TyBounds::size(&self) -> TyVector3` (`extents * 2`). `from_points` folds the
  first point as the seed and returns `None` on an empty iterator.
- `TyQuaternion::from_rotation_matrix(TyMatrix4x4) -> Option<Self>`. It truncates
  the first three columns to `TyVector3` and returns `None` when any column is
  shorter than `1e-12` (the degenerate guard); otherwise it feeds the normalized
  columns to `from_basis_vectors`, wrapped in `Some`. This revises the Q3
  "identity fallback": a collapsed matrix is not meaningfully the identity, so the
  honest result is `None` (and it parallels the `from_points` `Option`). Callers
  that know the matrix is a proper rotation `.expect(..)`; the mvox adoption does.
  The single "normalize" is on the columns (to strip scale); an orthonormal input
  already yields a unit quaternion, so the output is not re-normalized.
- `TyFloatExt::to_unorm8(self) -> u8`, `(self.clamp(0.0, 1.0) * 255.0).round() as
  u8`, added to the trait and both macro impls; `ty_float_ext.rs` gained its first
  `tests` module.
- `TySrgbaColor::to_vector3(self) -> TyVector3<f64>`, `self.to_rgba().to_vector3()`.
  Named `to_vector3` (not the Q3 `to_rgb`) because the crate has no 3-component RGB
  type, so the honest return is a vector and this mirrors
  `TyRgbaColor::to_vector3`. The RGB-vs-RGBA type asymmetry is deferred to a
  documented follow-up in the [README](../README.md#follow-up-the-rgb-color-type-model).

### B2: adopt in the non-vmax converters (landed)

- The triplicated qbcl `translation([i32; 3])` helper now bodies to
  `TyTransformF64::from_translation(TyVector3I32::from_array(position).to_f64())`,
  and the qbcl `placed_at` test helper to `from_translation`; this orphaned
  `TyQuaternionF64` in the three qbcl `from_*` imports (and the `from_qbcl` test
  import), which were trimmed. The `i32`-vector to `f64`-vector step motivated a new
  `TyVector3<i32>::to_f64()` in `ty-math` (a dedicated `impl TyVector3<i32>` block),
  the lossless counterpart to B1's `to_i32`, with its own unit test.
- mvox's `quaternion_from_matrix` is removed in favor of
  `TyQuaternion::from_rotation_matrix`, fed a `TyMatrix4x4F64` built column-major
  from the mirror-stripped 3x3. `from_basis_vectors` on those columns is the same
  branch-for-branch algorithm `quaternion_from_matrix` used, so the quaternion is
  identical for the proper (signed-permutation) frame. The call `.expect`s, since
  the frame is always proper after the mirror split. `ty-math` has no determinant,
  so the local `determinant` now returns the scalar triple product of the columns
  (`c0.dot(&c1.cross(&c2))`) via `ty-math`; for a signed-permutation matrix this is
  exactly +/-1, so the `< 0.0` mirror test is unchanged.
- The `round() as i32` world-position accumulation in goxl/qbcl `emit_node` now
  threads `parent`/`world` as `TyVector3I32` (`parent + position.round().to_i32()`),
  with `.to_array()` at the `emit_object`/`emit_object_node`/`synthesize_matrix`
  boundaries, where downstream u32 grid math keeps arrays. mvox's `translation_of`
  leaf helper returns `position.round().to_i32().to_array()`.
- gltf's byte-encode idiom (main and the test `byte` helper) uses
  `TyFloatExt::to_unorm8`; the mesh winding test (main and test) uses
  `TyVector3::triangle_normal`, matching the prior `(b - a).cross(&(c - a))` edge
  order exactly.
- **Deferred:** the three-component qbcl `color_floats([u8; 3])` helpers, because
  `TySrgbaColor::to_vector3` would need a synthetic alpha for a 3-byte source. Left
  in place pending the color-model follow-up.

### B3: adopt in the vmax converter (landed)

- `object_transform` is vectorized: `box_min`/`origin` (`[i32; 3]`) go through
  `TyVector3I32::from_array(..).to_f64()`, and the offset and world position become
  `(box_min - center - origin).componentwise_multiply(&scale)` and
  `object.position + center + rotation.rotate(offset)` on `TyVector3F64`. Same
  arithmetic, so the vmax round-trips are unchanged. This is the first consumer of
  the `to_f64` added for B2.
- Nothing else in the checklist applied: `from_rotation_matrix` is for a matrix,
  but vmax stores an axis-angle rotation (`axis_angle` -> `from_axis_angle`); the
  color pool already routes through `to_rgba` (Track A2); and
  `component_min_with`/`component_max_with` are float-only while `min_corner`
  (`[i32; 3]`) and `object_bounds` (`[u32; 3]`) are integer, so adopting them would
  add casts, not remove them. Left as-is.

## Track C: heavier logic under internal/

### Item 1: catalog the three patterns in internal/mesh (landed)

The three patterns from the [README](../README.md): (1) duplicated color scaling,
the 8-bit<->float normalize and its reverse-direction accumulate/average; (2)
array-shaped vector math, `[f64; 3]`/`[f64; 4]` where a `TyVector3`/color op would
serve; (3) hoistable primitives, reusable geometry or color operations `ty-math`
could own. Read `triangle_bounds.rs`, `triangle_box_overlap.rs`,
`voxelize_triangles.rs`, `sample_material.rs`, and the supporting `grid_space.rs`
(which holds `to_grid`, `cell_index`, `cell_center`, `clamp_index`).

**triangle_bounds.rs**

- Pattern 3, a direct duplicate of a landed Track B primitive. The whole function
  is `TyBounds::from_points`: it seeds on the first point and folds the rest with
  `component_min_with`/`component_max_with`, exactly `from_points`'s body, then
  returns a `(min, max)` tuple instead of a `TyBounds`. `TyBounds<f64>` already
  exposes `min()`/`max()`, so a tuple-shaped caller keeps working after the switch.
  No new `ty-math` needed. Adopt in item 2.
- No pattern 1; no stray array math (already `TyVector3F64`).

**triangle_box_overlap.rs**

- Pattern 2: the local `sub`/`dot`/`cross` free functions on `[f64; 3]`, plus
  `center: [f64; 3]`, `triangle: &[[f64; 3]; 3]`, and `axes: [[f64; 3]; 3]`. All
  three ops exist on `TyVector3F64` (`dot`, `cross`, `Sub`). The SAT projection in
  `separates` also needs the L1 norm `axis[0].abs() + axis[1].abs() +
  axis[2].abs()` (the box radius) and the min/max over the three vertex
  projections; `TyVector3` has `abs()` but no component-sum accessor, so the radius
  stays `.x + .y + .z`.
- Pattern 3: `triangle_box_overlap` itself is the triangle-box SAT overlap
  primitive the README names as a filing candidate (item 4); `separates` is its
  SAT-internal helper.
- Adoption gate: the `[f64; 3]` arrays flow in from `voxelize_triangles`' `grid:
  [[f64; 3]; 3]`, which is `GridSpace::to_grid`'s `[f64; 3]` return. Routing the
  overlap test through `TyVector3F64` ripples into `to_grid`'s return type, so this
  is a file-first primitive (item 4), not a drop-in.

**voxelize_triangles.rs**

- Pattern 2: `grid` is `[[f64; 3]; 3]` (from `to_grid`); `center = [x + 0.5, y +
  0.5, z + 0.5]`; `cell_range` folds per-axis min/max/floor over the three
  `[f64; 3]` corners. Array-shaped, but the arrays are grid coordinates tied to
  `to_grid`'s `[f64; 3]` return and the `[usize; 3]` cell-index math, so a clean
  `TyVector3` adoption waits on `to_grid` (see grid_space below).
- `fill_enclosed` is a pure integer-index flood fill: no vector math, no `ty-math`
  fit.
- `clamp_index` (defined in grid_space) is a float-floor-to-clamped-`usize` index
  helper: integer-index specific, a weak `ty-math` fit; not a candidate.

**sample_material.rs**

- Pattern 1: `CellAccum.base_color: [f64; 4]` and `emissive: [f64; 3]` accumulate
  component-wise (`accum.base_color[0] += color[0]` ...) and average (`/ n`) as raw
  arrays; `apply` repacks the mean into `TyLinearRgbaColorF64::new(.. / n, ..)`, and
  `base_color_linear`/`emissive` unpack a `TyLinearRgbaColorF64` back to
  `[f64; 4]`/`[f64; 3]`. This is array-shaped color math a linear color carrying
  `Add` + scalar `Mul`/`Div` (and `to_array`/`from_array`) would absorb. Gap:
  `TyLinearRgbaColorF64` today has only `new`, `componentwise_multiply`,
  `to_srgba`, `to_oklab`, `to_cielab` (accessors `.r/.g/.b/.a`), so adopting needs
  a `ty-math` extension first, not a drop-in. File in item 4.
- Pattern 2: `sample_steps`/`edge` compute the distance between two `[f64; 3]` grid
  points by hand (`dx, dy, dz`, `sqrt`), which `TyVector3F64::magnitude` of a
  difference would serve, but the inputs are `[f64; 3]` from `to_grid` again.
  `bary_point`/`bary_uv` do explicit component barycentric blends on
  `TyVector3F64`/`TyVector2F64`. `barycentric` (the point-onto-plane projection)
  already uses idiomatic `TyVector3F64` dot ops.
- Pattern 3: the barycentric blend (`bary_point`, `bary_uv`) and the
  point-in-triangle projection (`barycentric` + `clamp_barycentric`) are the
  barycentric-interpolation / point-in-triangle primitives the README names as
  filing candidates (item 4).

**grid_space.rs** (supporting; holds the array-shaped map both consumers build on)

- Pattern 2: `GridSpace.size: [f64; 3]`, `counts: [usize; 3]`; `to_grid` is the
  per-axis `(point - min) / size` returning `[f64; 3]`; `cell_center` rebuilds a
  `TyVector3F64` per axis. `to_grid` is the natural `(point -
  min).componentwise_divide(&size)`, but `ty-math` has `componentwise_multiply`
  only, no `componentwise_divide` (its `Div<T>` is scalar). A clean vectorization
  of `to_grid` (which would in turn let the rasterizer and overlap test go
  vector-native) waits on a `componentwise_divide` addition. File in item 4.
- `voxel_size` (the zero-extent guard) is scalar; no `ty-math` fit.

**ty-math surface gates (what item 1 establishes for the later items)**

- Present, safe to adopt now (items 2/3): `TyBounds::from_points`/`min`/`max`/
  `size`, `TyVector3::triangle_normal`, `magnitude`, `dot`/`cross`/`Sub`.
- Missing, an addition must land before adoption (item 4 or follow-ups): a
  `TyVector3::componentwise_divide` (for `to_grid`); `Add` + scalar `Mul`/`Div` +
  `to_array`/`from_array` on `TyLinearRgbaColor` (for the `CellAccum` color sums); a
  barycentric-blend primitive on `TyVector3`; a triangle-box SAT overlap primitive.
  These are named here and formally filed as checklist items in item 4.

### Items 2-5

Subsumed by the broad reuse audit (Track D below), which read the same
`internal/` logic as part of a codebase-wide pass and filed the concrete items:
Track C item 2's `from_points` in `triangle_bounds` is now a documented do-NOT
(bit-risk), its `triangle_normal`-in-rasterizer maps to the
`triangle_box_overlap` `Sub`/`dot`/`cross` adoption (D2 A1); item 4's larger
primitives were evaluated (barycentric rejected in favor of `Mul`+`Add`, the SAT
overlap kept as an adopt-existing, a swizzle set added); item 5's `write_vmax` is
Track D3. Item 3 (`cell_color` signature) was not surfaced as high-value and stays
an open question.

## Track D: broad reuse audit

A codebase-wide audit (2026-07-05), run as a multi-agent workflow (a `ty-math`
census, twelve subsystem finders over all voxsmith `convert/` + `internal/` plus
`vxl` and `voxcore`, a dedup/rank synthesis, an adversarial verify pass), deduped
54 raw findings into 30 verified proposals: 14 confirmed, 9 real but
scope-narrowed, 7 rejected. The full record with exact sites and per-proposal
verdicts is in
[reference/reuse-audit-findings.md](reuse-audit-findings.md); the actionable
items are Track D in the [checklist](../checklist.md).

Owner decisions:

- **Build every approved new method** (Group C, all eight). Extend `ty-math`
  freely, per Q1.
- **Swizzles as GLSL-style accessors**, not domain-named methods: the six
  permutations `xyz`/`xzy`/`yxz`/`yzx`/`zxy`/`zyx` on `impl<T: Copy>
  TyVector3<T>`, generic so they cover every component type. The Z-up <-> Y-up
  glTF conversion is a rotation (a swizzle plus one sign flip), so the swizzle is
  the reusable primitive and the sign stays local at the call site.
- **The 3-component qbcl `color_floats` helpers stay deferred.** Routing `[u8;3]`
  through `TySrgbaColor::to_vector3` needs a synthetic throwaway alpha, the exact
  RGB-vs-RGBA smell the [color-model
  follow-up](../README.md#follow-up-the-rgb-color-type-model) owns; the clean
  `voxelize_mesh` site (color already carries alpha) was adopted in `f2d4d5d`, the
  three qbcl `[u8;3]` sites were not.
- **Stage and pause per chunk**, not auto-commit: each Track D `[ ]` is staged and
  presented for review before it lands.

Verified corrections worth remembering:

- **Do NOT adopt `TyBounds::from_points` inside `triangle_bounds.rs`** (reverses
  the Track C item-2 sketch). It returns the direct `(min, max)` that sizes voxel
  cells; `from_points` reconstructs min/max through separately-halved center +/-
  extents, which is not bit-exact and can nudge a boundary sample into a
  neighboring cell. `size()` never reconstructs min/max, so the `mesh.rs` extent
  adoption (D2 A7) is safe; only the direct-corners use in `triangle_bounds` is
  not.
- **`TyQuaternion::normalized` is not a drop-in** for the voxj hierarchy
  quaternion normalize: `normalized()` is `self * (1.0 / magnitude())`
  (multiply-by-reciprocal), the hand-rolled code is per-component `x / magnitude`
  (true division); they round differently and could move goldens. Its
  `magnitude_squared` half is still adopted through the new `is_normalized` (C4).
- **A `barycentric` method is unwarranted**: the existing scalar `Mul<T>` + `Add`
  already express `p0*w0 + p1*w1 + p2*w2` bit-identically.

Rejected proposals (not built): the quaternion `normalized` adoption, a
`barycentric` method, `l1_norm`, an `(a-b).magnitude` edge distance,
`transform_aabb_conservative` on `TyTransform` (not bit-identical),
the `f32 -> f64` widen family (raw gltf arrays, not ty-math types), a
`TyVector3F64::to_u32`, and a test-only `/255` helper. Rationale per item is in
the findings file.

### C1: integer component min/max (landed)

The audit proposed a generic `impl<T: Copy + Ord> TyVector3<T>` for the integer
`component_min_with`/`component_max_with`, on the theory it could not collide with
the float methods since `f32`/`f64` are not `Ord`. That is wrong for inherent
methods: the compiler rejects it with E0592 (duplicate definitions), because
coherence cannot rule out an upstream `Ord` impl for `f64`, so the generic and the
concrete float methods are treated as a potential duplicate. The fix keeps the
same method names but defines them on the concrete integer types through a new
`impl_ty_vector3_int!` macro (invoked for `i32` and `u32`), mirroring the existing
`impl_ty_vector3_float!`. Float `min`/`max` semantics (NaN-ignoring) are untouched;
the integer methods use `Ord::min`/`Ord::max`, bit-identical to the hand-rolled
fold. Adopted in voxcore `live_extent`, replacing the six per-axis
`min.x.min(p.x)` lines with `min.component_min_with(&p)` /
`max.component_max_with(&p)`; the `max - min + 1` size still follows.

### C2: integer casts to_i32 / to_u32 (landed)

`TyVector3<u32>::to_i32` and `TyVector3<i32>::to_u32`, componentwise `as` casts
(wrapping, matching the hand-rolled code). Unlike the symmetric min/max, casts are
directional, so they live on the concrete `impl TyVector3<i32>` (`to_u32`, beside
`to_f64`) and a new `impl TyVector3<u32>` (`to_i32`), not the shared
`impl_ty_vector3_int!` macro; the float `to_i32` (truncating f64/f32 -> i32) is a
different concrete type, so no collision. Adopted only at the sites whose source is
already a `TyVector3`: `internal/grid.rs` vectorizes end to end
(`origin + min.to_i32()`, `-min.to_i32()`, and `copy_voxels`'s offset threaded as
`TyVector3I32` so the body is `(p.to_i32() + offset).to_u32()`), and
`convert/mvox/to_mvox_file.rs` pivots via `(bounds / 2).to_i32().to_array()`. The
`to_goxl_file.rs:201` and `voxj_decoded_object_from_vox_object.rs:58-62` sites lift
an input `[i32;3]`/`[u32;3]` array through `from_array`, which is A6 (packing)
plumbing (voxj even destructures a vector to `[u32;3]` at `:18` just to rebuild it),
so both defer to A6 rather than round-trip through `from_array` here. All casts are
byte-identical; no golden moved.

### C3: u32 to_f64 (landed)

`TyVector3<u32>::to_f64`, the lossless widen completing the family (`i32` already
had it), added to the same `impl TyVector3<u32>` block beside `to_i32`. Adopted in
vxl `hierarchy_show.rs` object_rows (`object.bounds().to_f64()`, `min.to_f64()`,
`size.to_f64()`), deleting the `vec_u32_to_f64` helper. That was the module's only
by-name use of `TyVector3U32` (the rest are in the test module's own import), so it
came off the line-13 import. The i32 sibling `vec_i32_to_f64` stays until A3
removes it (its call at `:928` and a second mvox site). `u32 -> f64` is exact, so
the hierarchy display output is unchanged.

### C4: quaternion is_normalized (landed, one site)

`TyQuaternion::is_normalized(self, tolerance) -> bool`, `(self.magnitude_squared()
- 1.0).abs() <= tolerance`, in the float macro beside `normalized`. Adopted in
voxcore `vox_main`'s node-transform check: the four-term `length_squared` fold and
the `(length_squared - 1.0).abs() > ROTATION_TOLERANCE` gate become
`!rotation.is_normalized(ROTATION_TOLERANCE)`. `rotation` is already a
`TyQuaternionF64` and `magnitude_squared` sums `x,y,z,w` in the same order, so the
error is raised on exactly the same inputs; behavior-preserving.

The second audit site, voxj-codec `check_transforms.rs`, is DEFERRED. voxj-codec's
dependencies are base64/flate2/serde_json/voxj -- no `ty-math` -- and it validates
the raw voxj serde transform (`rotation: [f64; 4]`). Adopting `is_normalized` there
means constructing a `TyQuaternion` from the array and adding `ty-math` as a
dependency to a deliberately lean codec crate. That is a layering decision for the
owner, not a mechanical adoption, so the hand-rolled `length_squared` check stays
until that call is made.

### C5: glTF axis rotation, not swizzles (landed)

The audit and an early owner preference framed C5 as the six GLSL swizzle
accessors (`.xyz`/`.xzy`/...). Reading the actual call sites reversed that: every
glTF site is a +/-90 rotation about X, a permutation PLUS one sign flip
(`(x, z, -y)` on export, `(x, -z, y)` on import), so a pure permutation never
produces the negation and would need a trailing negate, no cleaner than the
explicit `new(x, z, -y)`. No pure-permutation site exists in the tree, so the
swizzles had no clean consumer. The owner chose sign-aware, domain-named helpers
instead: `TyVector3::zup_to_yup` (`(x, z, -y)`) and `yup_to_zup` (`(x, -z, y)`),
generic over `T: Copy + Neg<Output = T>` so both f32 and f64 grids use them.
Adopted at all three glTF sites: `from_gltf_bytes` (`world.yup_to_zup()` after the
`transform_point`), and the `object_to_gltf_document` / `material_document` export
closures (`p.zup_to_yup() * scale` for positions, `n.zup_to_yup()` for the unit
normals, the `* scale` staying the existing `Mul<T>`). Pure component reorder plus
negate, so the baked glTF bytes are unchanged.

### C7: componentwise_divide (landed)

`componentwise_divide` on `impl<T: Copy + Div<Output = T>> TyVector3<T>`, the
companion to `componentwise_multiply` in its own minimal-bound block (Div only, so
it stays off the Add/Mul/Sub geometry block). Adopted in `internal/mesh/grid_space`
by retyping the private `size` field from `[f64; 3]` to `TyVector3F64` (a private
field, nothing serialized): `to_grid` becomes `(point -
min).componentwise_divide(&size).to_array()`, and `cell_center` builds an
`(x+0.5, y+0.5, z+0.5)` offset and returns `min +
offset.componentwise_multiply(&size)` (the existing multiply), replacing the
per-axis `self.size[i]` indexing the retype forced anyway. Every operation is the
same per-component subtract/divide/multiply/add in the same order, so the
rasterizer and sampler are bit-identical.

### C6: from_min_size (added, adoption trails in D3)

`TyBounds::from_min_size(min, size) -> Self` in the `impl_ty_bounds_float!` macro
(f32/f64), beside `from_points`: `extents = size * 0.5`, `center = min + extents`.
Float-only, matching the sibling constructors that all use `* 0.5` / `* 2.0`. The
unit test mirrors the `from_points` case (same min `(-1, -2, 2)`, size
`(4, 6, 3)`), so the two constructors are shown to agree, and asserts `center`,
`min`, `max`, and `size`. No adoption this chunk: both consumers are vmax
(`write_vmax` `content_box` at `:599` and `object_box_local` at `:1276`, which
compute `half = bounds / 2`, `center = box_min + half`, and return
`(center, -half, half)`), so the min/size widen and swap land in D3 B5. The
`from_min_max` sibling the audit floated was dropped as a lone one-off.

### A1: triangle_box_overlap through TyVector3F64 (landed)

`internal/mesh/triangle_box_overlap.rs` now converts its inputs to `TyVector3F64`
via `from_array` and runs the separating-axis test on the vector `Sub`, `dot`, and
`cross`, deleting the local `sub`/`dot`/`cross` free functions. The three ops are
component-for-component identical to the deleted helpers (same operand order and
`(a + b) + c` associativity in `dot`), so the overlap decision is bit-identical and
no voxelization golden moved. Notes:

- The public signature stays `triangle_box_overlap(center: [f64; 3], half: f64,
  triangle: &[[f64; 3]; 3])`. The `[f64; 3]` corners flow in from
  `voxelize_triangles`' `to_grid` return, so widening the boundary to `TyVector3F64`
  would ripple into `to_grid`; the conversion stays inside the function.
- The triangle normal is kept as the explicit `edges[0].cross(&edges[1])`, NOT
  `TyVector3::triangle_normal`. `triangle_normal(v0, v1, v2)` is `(v1 - v0) x (v2 -
  v0)`, but the code crosses `(v1 - v0) x (v2 - v1)`; the two are the same plane
  normal mathematically but computed from different operands, so only the original
  edge cross is bit-exact.
- `separates`'s box radius stays the explicit `axis.x.abs() + axis.y.abs() +
  axis.z.abs()` sum: `TyVector3` has `abs()` but no component-sum accessor, so the
  scalar sum is the byte-identical form.
- Verification gotcha: `internal/mesh` is behind `#[cfg(feature = "_mesh")]`, which
  only the `gltf` feature enables and voxsmith's defaults do not, so the file and
  its tests compile only under `--features gltf`. The gates were rerun as
  `cargo {check,clippy,test} -p voxsmith --features gltf` (176 tests green,
  including the five `triangle_box_overlap` cases).

### A3: i32 grid vectors through to_f64 / from_array (landed)

The two `i32 as f64` widen sites now use the existing `TyVector3I32` casts, both
lossless and byte-identical to the per-component `as f64`:

- vxl `implementation/hierarchy_show.rs`: `object_rows` reads
  `object.origin().to_f64()` and the private `vec_i32_to_f64` helper is deleted.
  That was the module's last by-name use of `TyVector3I32` outside the test module,
  so it comes off the line-13 import (mirrors the C3 `vec_u32_to_f64` removal). vxl
  tests stay green (152).
- mvox `convert/mvox/from_mvox_file.rs`: `transform_from_frame`'s position is
  `TyVector3I32::from_array(frame.translation).to_f64()` (`translation` is
  `[i32; 3]`), replacing the three-line `TyVector3F64::new(.. as f64 ..)`;
  `TyVector3I32` joins the import. mvox is a default feature and its frame
  round-trips (117 tests) stay green with no golden change. mvox is not vmax, so
  this is non-vmax D2 work.

### A5: glTF AABB seeds through the INFINITY consts (landed)

The two mesh AABB folds seed their `min`/`max` with `TyVector3F32::INFINITY` /
`TyVector3F32::NEG_INFINITY` instead of `TyVector3F32::splat(f32::INFINITY)` /
`splat(f32::NEG_INFINITY)`, in `internal/gltf/object_to_gltf_document.rs` and
`material_document.rs`. The consts already existed in the `impl_ty_vector3_float!`
macro (no new API) and expand to `Self { x: INFINITY, y: INFINITY, z: INFINITY }`,
identical to the `splat` form, so the accessor `min`/`max` bytes are unchanged. Both
files are behind the `gltf` feature, so the gates ran as `--features gltf`
(176 tests green, including the glTF export goldens that carry the AABB).

### A6: voxj/goxl array packing + deferred C2 casts (landed)

Seven sites, all behavior-preserving. The plain `to_array`/`from_array` swaps are
byte-identical (the macro packs/unpacks `x, y, z[, w]` in order); the two
vectorizations are the notable ones. mvox/vmax untouched; goxl and voxj are default
features and `object_to_mesh_geometry` is behind `gltf`, so the gates ran
`--features gltf` (176 tests green, no golden change).

- `voxj_hierarchy_node_from_vox_hierarchy_node.rs`: the `VoxjTransform` pack is
  `transform.position.to_array()` / `rotation.to_array()` / `scale.to_array()`
  (the quaternion `to_array` is `[x, y, z, w]`, matching the old field list).
- `vox_hierarchy_node_from_voxj_hierarchy_node.rs`: **position only.** The finite
  check iterates `transform.position` directly and the build is
  `TyVector3::from_array(transform.position)`; rotation keeps its magnitude
  normalize and scale keeps its non-zero validation, so both stay destructured.
- `vox_object_from_voxj_decoded_object.rs`: `TyVector3U32::from_array(bounds)` and
  `TyVector3I32::from_array(origin)`; the `size_x/y/z` destructure stays for the
  error messages.
- `voxj_decoded_object_from_vox_object.rs`: fully vectorized. `live_extent` now
  keeps its `(min, size)` as `TyVector3U32` (the empty case is
  `unwrap_or((new(0,0,0), new(0,0,0)))`), so `positions.push((position -
  min).to_array())` (u32 vector subtract, same wrap/panic as the per-axis form),
  `bounds: size.to_array()`, and the deferred C2 cast `origin: (origin +
  min.to_i32()).to_array()`.
- `write_voxj.rs`: the `VoxjEditObject` is `bounds.to_array()` / `origin.to_array()`.
- `convert/mesh/object_to_mesh_geometry.rs`: `object.bounds().to_array()` collapses
  the read-then-repack.
- `convert/goxl/to_goxl_file.rs`: the deferred C2 cast. `emit_object`'s `world`
  parameter becomes `TyVector3I32` (already the type in `emit_node`, so its call
  drops the `.to_array()`; the orphan-object call passes
  `TyVector3I32::new(0, 0, 0)`), and the body is `let world_position = (world +
  position.to_i32()).to_array()`.
- Coverage of the non-zero cast paths: the voxj `never_discards_edit_state_with_
  margin` round-trip uses `origin: [-1, 0, 0]` (negative), exercising the site-4
  `origin + min.to_i32()` add and the site-5 emit; the goxl block-position fixtures
  (`[-16, 16, -32]`) exercise the site-7 `world + position.to_i32()` add. Both
  vector adds are the same `TyVector3I32 + TyVector3U32::to_i32()` shape.

### A7: TyBounds in mesh extent + the glb test (landed)

- `convert/voxelize/mesh.rs` `extent()` folds the triangle points through
  `TyBoundsF64::from_points` and returns `bounds.size()` instead of calling
  `triangle_bounds` and subtracting `max - min`; `triangle_bounds` drops from the
  import (still used elsewhere in `internal/mesh`, not touched). `size()` is
  `extents * 2.0 = ((max - min) * 0.5) * 2.0`, which is bit-identical to `max - min`
  for the normal-range mesh extents here (halving then doubling is exact away from
  the subnormal floor), so the grid-resolution figure is unchanged. Covered by the
  tolerance-asserting `extent_converts_gltf_y_up_to_voxel_json_z_up` test.
- `convert/gltf/object_to_glb_bytes.rs` test `z_up_bar_becomes_y_up_and_scales`
  builds `TyBoundsF32::from_points` over the exported positions and reads
  `bounds.max()` for the two axis maxima, replacing the two `fold(f32::MIN,
  f32::max)` lines. This is a test-only assertion with a `1e-6` tolerance, so the
  `center + extents` corner reconstruction (not bit-exact in general) is fine, and
  for the bar's clean integer coordinates it is exact anyway. This is why A7 stops
  at `size()`/`max()` and does NOT touch `triangle_bounds.rs`, whose direct
  `(min, max)` corners size voxel cells and must stay bit-exact.

### A8: TyVector3F64::ZERO in hierarchy_show (landed)

vxl `hierarchy_show.rs` `object_rows`: the empty-object runtime size becomes
`TyVector3F64::ZERO` instead of `TyVector3F64::new(0.0, 0.0, 0.0)`. Cosmetic and
bit-identical (the const is `{ x: 0.0, y: 0.0, z: 0.0 }`); vxl tests stay green
(152). This closes Track D2: A1/A3/A5/A6/A7/A8 landed, A2/A4 moved to the
[ty-color-model plan](../ty-color-model/README.md). Remaining work is Track D3
(vmax, trailing commits).

### D3 B1: write_vmax vector() helper -> to_array (landed)

First Track D3 (vmax) chunk, in its own trailing commit per Q2. The private
`vector(TyVector3F64) -> [f64; 3]` helper (body `[v.x, v.y, v.z]`) was deleted for
`TyVector3F64::to_array()`, which `ty_array_conversions!(TyVector3, 3, x, y, z)`
generates as the same `[self.x, self.y, self.z]`, so the packed `[f64; 3]` is
byte-identical. Adopted at the three pack sites: `VMaxObject.scale` (`:1209`) and
`VMaxGroup.position`/`.scale` (`:1386`/`:1388`), all reading
`node.transform.{position,scale}` off a `TyTransformF64` whose `position`/`scale`
fields are `TyVector3F64`. The `TyVector3F64` import stays: six other sites in the
file use it (`from_axis_angle`, the three per-axis scale rotates, the offset
build). The audit line numbers had shifted (the helper was at `:1466`, not the
`:1311` the audit recorded), reconfirmed at the keyboard. voxsmith stays green
(117 tests), no golden moved.

### D3 B2: transform_point method in subtree_box_local (landed)

The local free fn `transform_point(&TyTransformF64, [f64; 3]) -> [f64; 3]` (scale
componentwise, `rotation.rotate`, then add `position`) was deleted for the
`TyTransformF64::transform_point` method, which computes `position +
rotation.rotate(scale.componentwise_multiply(&point))` -- the same TRS in the same
order. Byte-identical: `componentwise_multiply` is `scale.{x,y,z} * point.{x,y,z}`
where the free fn wrote `point[i] * scale.{}`, and IEEE-754 multiply is
commutative bit-for-bit, so the scaled vector matches; the `rotate` input and the
per-component `position + rotated` add are otherwise identical. The lone call in
`subtree_box_local` (`:1251`) wraps its `[f64; 3]` `child_center` through
`TyVector3F64::from_array(..)` and reads the result back with `.to_array()`, both
byte-identical packing, so the memoized subtree box is unchanged. `transform_half`
(the sibling AABB half-extent fold) was left as-is: it is a per-column rotate of
the scaled basis vectors, not a `transform_point`, so no method fits. voxsmith
stays green (117 tests), no golden moved.

### D3 B3: round-to-nearest chains in from_vmax_file (landed)

Two per-component `(..).round() as i32` chains now fold through the vector
methods, both byte-identical:

- `pivot_origin` (`:545`) becomes
  `(TyVector3I32::from_array(box_min).to_f64() -
  TyVector3F64::from_array(center)).round().to_i32().to_array()`. The `Sub` keeps
  the `box_min - center` order (non-commutative, preserved); `to_f64` is the exact
  `as f64` widen, `round` is per-component `f64::round` (half away from zero),
  `to_i32` truncates an already-integer value.
- `authored_box`'s `box_min` (`:585`) becomes
  `(TyVector3F64::from_array(object.center) +
  TyVector3F64::from_array(min)).round().to_i32().to_array()`.

The sibling `size` in `authored_box` (`(max - min).round().max(0.0) as u32`) is
LEFT as the per-component array form: it needs a round-then-saturating-`as u32`,
i.e. the `TyVector3F64::to_u32` the audit explicitly rejected (there is no
float-vector `to_u32`; the integer `to_u32` is on `TyVector3<i32>`, and routing
through it would add a clamp-and-cast, not remove one). `min` stays a `[f64; 3]`
local (used by both the vectorized `box_min` and the array `size`); `from_array`
copies it, so the later `min[i]` indexing is unaffected. The pure-`Sub` offset at
`:165` was already a `TyVector3I32` subtract with no rounding, so it is not in
B3's scope. voxsmith stays green (117 tests), no golden moved.
