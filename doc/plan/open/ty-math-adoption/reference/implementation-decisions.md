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

_Pending. Record the final method names against the Q3 recommendations and any
signature that differed from the plan (for example whether the vector cast landed
as `to_i32`, `as_i32`, or a combined `round_to_i32`)._

## Track C: heavier logic under internal/

_Pending. Record the catalog of the three patterns per file, which adoptions were
safe, and each larger primitive filed as a new item with its target type._
