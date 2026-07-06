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

_Pending. Record which adoptions landed and each larger primitive filed with its
target type._
