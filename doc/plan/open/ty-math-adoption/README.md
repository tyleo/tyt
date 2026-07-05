# ty-math Adoption in the voxsmith Converters Plan

Status: **open.** This plan captures a math-consolidation pass over the voxsmith
converters, driven by an audit of `projects/utilities/voxsmith/src/convert`
against the `ty-math` crate. The converters already use `ty-math` well at the
`voxcore` boundary, but the same hand-rolled color and vector arithmetic recurs
across the format readers and writers, and a few reusable primitives live in
codec files that `ty-math` should own. The work splits into three tracks:
behavior-preserving cleanups that need no new API, a `ty-math` extension that
absorbs the recurring patterns and adopts them, and an investigation of the
heavier logic under `internal/` for the same opportunities. The executable steps
live in [checklist.md](checklist.md), with a per-session resume prompt in
[continue-ty-math-adoption.md](continue-ty-math-adoption.md) and a running log of
code-level choices in
[reference/implementation-decisions.md](reference/implementation-decisions.md).

`ty-math` is the shared math foundation and is in this workspace, so it is meant
to grow: per-crate custom math migrates into it rather than the reverse. Every
consumer (`voxcore`, `voxsmith`, `vxl`) takes it through the root
`[patch.crates-io]`, so a new method reaches all of them in one step with no
external coordination and no version bump. The one dependency this plan does not
extend is `branded-id`.

## The audit, in one paragraph

The transform, grid, position, and color values the converters hand to `voxcore`
(`TyVector3U32`, `TyTransformF64`, `TyQuaternionF64`, `TySrgbaColor`) are already
typed and idiomatic, and `MeshGeometry` stores `Vec<TyVector3F32>`. The gaps are
three: an 8-bit-to-float sRGB normalize (`color_floats`, `c.map(|b| b as f64 /
255.0)`) is copied into six format readers instead of using `TySrgbaColor::to_rgba`;
`ty-math` is missing a handful of small operations (a vector round, a
translation-only transform, an AABB-of-points constructor, a triangle normal, a
matrix-to-quaternion, a float-to-unorm8) so position and geometry math falls back
to raw arrays; and one function, mvox's `quaternion_from_matrix`, is a verbatim
reimplementation of the algorithm `ty-math` already ships as
`TyQuaternion::from_basis_vectors`. The heaviest conversion logic
(`write_vmax`, `cell_color`, `voxelize_triangles`, `sample_material`,
`triangle_bounds`) lives in `voxsmith/src/internal`, not `convert/`, so the
richest primitives are just outside the audited directory; Track C covers them.

## The tracks

**Track A: harmless cleanups, no new API.** Behavior-preserving refactors that
route existing hand-rolled math through methods `ty-math` already has. The
four-component `color_floats` copies collapse to `TySrgbaColor::to_rgba`; the test
hex parsers collapse to `TySrgbaColor::from_hex`; `TyVector3::new(s, s, s)` becomes
`splat(s)`; `new(1, 1, 1)` and `default()` become `ONE` and `ZERO`; the
`[v.x, v.y, v.z]` rebuilds become `to_array()`; and vmax's `vec3` becomes
`from_array`. Each edit is numerically identical to what it replaces. Nothing here
touches `ty-math` itself.

**Track B: extend `ty-math`, then adopt.** Add the small operations the audit
found missing, each behind the crate's existing float-macro pattern, and switch
the converters onto them. The additions are a vector `round` plus an `i32` cast, a
`TyTransform::from_translation`, a `TyBounds::from_points` with a `size` accessor,
a `TyVector3::triangle_normal`, a `TyQuaternion::from_rotation_matrix` wrapping the
existing `from_basis_vectors` with a normalize and identity guard, a
`TyFloatExt::to_unorm8`, and a `TySrgbaColor::to_rgb` for the alpha-dropping
callers. Adoption removes the triplicated qbcl `translation` helper, the mvox
`quaternion_from_matrix` and `determinant`, the duplicated triangle-normal
winding test, and the remaining array-shaped position math.

**Track C: investigate the heavier logic under `internal/`.** The audit was scoped
to `convert/`, but the reusable geometry and the reverse-direction color emit live
in `internal/mesh` and `internal/vmax`. Read `voxelize_triangles`,
`sample_material`, `triangle_bounds`, `cell_color`, and `write_vmax`; catalog the
same three patterns (duplicated color scaling, array-shaped vector math, hoistable
primitives); adopt the Track B additions where they already fit (for example
`TyBounds::from_points` in `triangle_bounds`, `triangle_normal` in the
rasterizer); and file anything larger (a triangle-box overlap primitive, a
`cell_color` returning `TySrgbaColor`, a color-distance metric on
`to_oklab`/`to_cielab`) as its own checklist item. This track is investigation
first and may land as more than one commit.

## vmax runs through all three tracks

Every track touches vmax somewhere: Track A cleans `convert/vmax/from_vmax_file.rs`
and its tests, Track B adopts `from_rotation_matrix` and the vectorized transform
there, and Track C reads `internal/vmax/write_vmax.rs`. Another branch is
concurrently editing vmax, so the ground rule across the whole plan is that any
edit under `convert/vmax` or `internal/vmax` lands in its own commit, sequenced
last within its track, so it can be held, reordered, or rebased against the other
branch without dragging the non-vmax work with it.

## Not in scope

- `branded-id` is not extended; it stays an external dependency.
- The wire formats and the voxcore model do not change. Every edit is
  consumer-side arithmetic; no serialized bytes move.
- No new `ty-math` types beyond methods on the existing ones, unless Track C's
  investigation makes an explicit case for one (for example a triangle type) and
  files it as its own item first.

## Crates in the blast radius

- **`ty-math`** (Track B, Track C where adopted): new methods on `TyVector3`,
  `TyTransform`, `TyBounds`, `TyQuaternion`, `TyFloatExt`, and `TySrgbaColor`, each
  in the existing per-file float-macro form with a doc comment and a unit test.
- **`voxsmith`** (all tracks): `convert/{qbcl,goxl,mvox,vmax,gltf,voxelize,mesh}`
  in Tracks A and B; `internal/{mesh,vmax,cell_color}` in Track C.
- **`voxcore`, `vxl`**: consume `ty-math` through the workspace patch; no code
  change unless Track C changes a shared signature (for example `cell_color`),
  which would be filed and decided first.

## What is settled

- `ty-math` is the in-workspace math foundation and is meant to absorb this
  arithmetic; extending it is the goal, not a last resort. The prior voxj plan's
  note against extending it does not bind here.
- All `ty-math` consumers resolve through `[patch.crates-io]` at the workspace
  root, so a new method reaches `voxcore`, `voxsmith`, and `vxl` together with no
  version bump.
- The audit's per-file findings, with line numbers, are the source for the
  checklist; they are recorded there rather than repeated here.
- vmax edits are isolated by commit because a second branch is in the same files.

## Decisions

### Q1. How far to extend `ty-math`

**Decision: extend it freely for the recurring patterns.** The additions in Track
B are small, general, and each removes real duplication or unblocks typed code.
They go in the crate's existing float-macro pattern with doc comments and tests.
Every consumer is patched to the local path, so no version bump is needed.
`branded-id` is the sole exception and is left untouched.

### Q2. Sequencing against the concurrent vmax branch

**Decision: isolate every vmax edit in its own commit, last within its track.**
Both `convert/vmax` and `internal/vmax` are being edited on another branch. Keeping
our vmax changes in dedicated, trailing commits lets them be dropped, reordered, or
rebased independently, and keeps the non-vmax cleanups landable regardless of the
other branch's state. Track A and Track B each end with a vmax-only commit; Track C
puts `write_vmax` in its own.

### Q3. Names and placement of the new `ty-math` methods

**Decision: recommended names below, refined in the decision log as they land.**
The shapes are settled; the exact identifiers are an implementation detail to
confirm at the keyboard and record in
[reference/implementation-decisions.md](reference/implementation-decisions.md).

- `TyVector3<f32|f64>::round(self) -> Self` and a `to_i32(self) -> TyVector3<i32>`
  truncating cast, composed as `pos.round().to_i32()`.
- `TyTransform<f32|f64>::from_translation(position) -> Self` (identity rotation,
  unit scale).
- `TyBounds<f32|f64>::from_points(impl IntoIterator<Item = TyVector3>) -> Option<Self>`
  and `size(&self) -> TyVector3` (the full extent, `extents * 2`).
- `TyVector3::triangle_normal(a, b, c) -> Self`, the unnormalized `(b - a) x
  (c - a)`.
- `TyQuaternion<f32|f64>::from_rotation_matrix(TyMatrix4x4) -> Self`, wrapping
  `from_basis_vectors` on the upper-left columns with a normalize and an identity
  fallback.
- `TyFloatExt::to_unorm8(self) -> u8`, the `(x.clamp(0, 1) * 255).round() as u8`
  idiom, beside the existing `quantize`.
- `TySrgbaColor::to_rgb(self) -> TyVector3<f64>`, the alpha-dropping companion to
  `to_rgba`, for the three-component sRGB pools.

## Execution shape

1. Track A: the behavior-preserving cleanups, non-vmax first, vmax in a trailing
   commit. No `ty-math` change.
2. Track B: land each `ty-math` method with its test, then adopt across the
   converters, non-vmax first and vmax trailing.
3. Track C: read the `internal/` logic, adopt the Track B additions where they fit,
   and file the larger primitives as new items; `write_vmax` in its own commit.
   This track may span more than one commit.

## Test and fixture strategy

Tracks A and B are refactors of covered code, so the existing converter and command
tests are the primary gate: they must stay green with no golden churn, since no
serialized bytes change. Each new `ty-math` method ships a focused unit test in its
own file's `tests` module, matching the crate's convention (a round-trip or a
known-value assertion). Track C adds coverage only where it changes behavior, for
example a `from_points` adoption in `triangle_bounds` asserted against the prior
tuple result, or a signature change to `cell_color` exercised through the readers
that consume it. No wire fixtures are rebuilt.
