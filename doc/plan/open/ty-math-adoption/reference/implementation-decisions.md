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

## Track C: heavier logic under internal/

_Pending. Record the catalog of the three patterns per file, which adoptions were
safe, and each larger primitive filed as a new item with its target type._
