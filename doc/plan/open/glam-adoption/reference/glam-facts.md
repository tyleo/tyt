# glam crate facts (glam-only; glamx evaluated, not adopted)

Verified against docs.rs and the bitshifter/glam-rs + dimforge/glamx repos
(2026-07-23). Re-confirm at the keyboard before coding.

## glam (bitshifter/glam-rs)

- Latest `0.33.2` (2026-06-28). Edition 2021, MSRV 1.68.2, `MIT OR Apache-2.0`.
  Zero external deps by default. Compatible with the edition-2024 workspace.
- **Not generic over the scalar.** Every scalar family is its own concrete
  `#[repr(C)]` struct: `Vec3`/`DVec3`/`IVec3`/`UVec3`, `Quat`/`DQuat`,
  `Mat4`/`DMat4`, etc. There is no `Vec3<T>`. So ty-math cannot keep its generic
  `Ty*<T>` and goes to concrete aliases (owner: the `T` was a code-saving device).
- f64 families all exist under `float-types` (default): `DVec2/3/4`, `DQuat`,
  `DMat2/3/4`, `DAffine2/3`. No `DVec3A`/`DAffine3A` (f32-SIMD only; unused here).
- **Public fields** `x/y/z/w` on vectors and quaternions, so every consumer
  `.x/.y/.z/.w` read AND write survives. `Mat4`/`DMat4` expose `.x_axis..w_axis`
  (Vec4 columns) + `.col(i)`; there is NO `.columns` array and NO element index.
- Derives: float types `Clone,Copy,PartialEq` (NO `Eq`/`Hash`); integer types add
  `Eq`+`Hash`. `Debug`/`Display`/`Default` are hand-implemented (Default = `ZERO`
  for vecs, `IDENTITY` for Quat/Mat). Matches ty-math's Default.
- **serde emits a flat SEQUENCE** (`DVec3 -> [x,y,z]`, `DQuat -> [x,y,z,w]`), NOT
  `{x,y,z}`. So the ty-math-serde DTO stays and glam's `serde` feature stays OFF.

### The assert model (we opt IN, owner decision)

glam methods that require valid inputs carry `glam_assert!(...is_normalized())`
guards on `from_axis_angle` (unit axis), `inverse` (unit quat), `mul_vec3`,
`slerp`, `from_rotation_arc`, `from_rotation_axes`/`from_mat3`/`from_mat4`,
`normalize` (nonzero), `angle_between`. These are OFF unless a feature enables
them:

- `debug-glam-assert` - assertions fire in DEBUG builds only (tests included),
  ZERO release cost.
- `glam-assert` - assertions fire in ALL builds, release included.

**Decision: enable `debug-glam-assert`.** ty-math adopts glam's fail-fast
contract - callers pass valid (unit) inputs, and a violation panics loudly in
debug/test instead of silently producing a wrong result. This is faster in the
long run (no defensive normalize on every call) and makes the migration
self-checking: a call site that passes a non-unit axis to `from_axis_angle` trips
the assert in its test, pointing straight at the `.normalize()` it needs. (Switch
to `glam-assert` if release-time checks are ever wanted.)

Auto-fixing is opt-in and NAMED: the strict glam function keeps its plain name;
any auto-normalizing / auto-fixing convenience is a separately, distinctly-named
variant (a prefix/suffix, mirroring glam's own `normalize_or_zero` / `try_normalize`),
so the extra work is obvious at the call site. Prefer an explicit `.normalize()`
at the call site; add a named variant only where the pattern recurs.

## Dependency wiring (glam direct)

```toml
glam = { version = "0.33", default-features = false,
         features = ["std", "float-types", "integer-types", "debug-glam-assert"] }
```

- `float-types` gives `DVec*`/`DQuat`/`DMat4`; `integer-types` gives
  `IVec3`/`UVec3`. Drop `size-types` (unused). Do NOT enable `serde`.
- ty-math re-exports every `Ty*` alias + its extension traits, so no consumer
  ever names `glam` (mirrors how the palette plan confined `palette`).

## glamx: evaluated, not adopted (owner decision)

`glamx` (dimforge, v0.3.0) is a real glam superset that re-exports glam 0.33 and
adds `Pose3`/`DPose3` (rotation+translation, no scale), `Rot2`, `MatExt`
(incl. `abs()`, `try_inverse()`, `svd()`), a scalar `FloatExt`
(lerp/inverse_lerp/remap/fract_gl/step/saturate), and `Svd`/`SymmetricEigen`.
The owner chose **glam-only** (fewer deps, full control), but with two riders:

1. **Prefer glam's names** over ty-math's legacy names wherever they fit ty-math's
   paradigms. The DIRECT renames already do this (`magnitude`->`length`,
   `normalized`->`normalize`, `new`->`from_xyzw`, `UNIT_X`->`X`). Extend it to the
   kept surface: composites expose an `IDENTITY` associated const (glam's
   convention) rather than an `identity()` fn. Keep a descriptive ty-math name
   only where glam has no equivalent (`triangle_normal`, `zup_to_yup`, `quantize`).

2. **Lift glamx's better algorithms into ty-math using pure glam** (no glamx dep):
   - Quaternion-from-matrix and basis-vector construction delegate to glam's
     tested `Quat::from_rotation_axes` (glamx/glam's algorithm), dropping
     ty-math's hand-rolled trace-branch in `from_basis_vectors`.
   - `rotate_extents_abs` mirrors glamx's `MatExt::abs()` fold with pure glam:
     `let m = DMat3::from_quat(q); DMat3::from_cols(m.x_axis.abs(), m.y_axis.abs(),
     m.z_axis.abs()) * extents` (glam vectors have `.abs()`). Cleaner than the
     inlined scalar formula and reuses glam's `from_quat`.

`TyPose` therefore stays a hand-rolled glam-backed struct like the other
composites (NOT the glamx `DPose3` alias).
