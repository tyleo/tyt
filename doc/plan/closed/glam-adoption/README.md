# Back ty-math's math types with `glam`

Status: **closed.** Landed as one commit (`22650a6`, 2026-07-24), rebased on
origin's latest. All nine steps (S1-S9) shipped: ty-math's math types are now
concrete `glam` aliases (`TyVector2/3/4`, `TyQuaternion`, `TyMatrix4x4`, plus the
F32/F64 and V3 I32/U32 families) with a small set of extension traits and four
glam-backed composites (`TyBounds`/`TyTransform`/`TyUniformTrs`/`TyPose`); every
consumer moved onto glam's own methods; the workspace is green (clippy clean, all
tests passing) with no external wire moved and `glam` named only inside ty-math.
The per-step keyboard record is in
[reference/implementation-decisions.md](reference/implementation-decisions.md).
A direct follow-up to the closed
[palette-adoption plan](../palette-adoption/README.md), which replaced ty-math's
hand-rolled color types with `type Ty... = palette::...` aliases and moved
consumers onto palette's own methods. This plan does the same for the GEOMETRIC
types, replacing hand-rolled vector/quaternion/matrix math with
[`glam`](https://docs.rs/glam) (a maintained, SIMD-capable linear-algebra crate).
ty-math keeps the `Ty...` names as the public vocabulary; `glam` stays an
implementation detail confined to ty-math.

## Goal in one paragraph

ty-math's math types already have glam's SHAPE (x/y/z fields, dot/cross,
column-major matrices, xyzw quaternions); they should have glam's CODE. Three
wins: (1) take glam's implementations where they are equal-or-better (SIMD,
`try_inverse`, a maintained slerp/euler/decompose surface, a tested
rotation-matrix-to-quaternion); (2) stop maintaining our own vector/quaternion/
matrix math and its tests; (3) `Vec*Vec` becomes the `*` operator and glam's
naming collapses a lot of hand-rolled boilerplate. The `Ty...` names stay what
consumers write - no consumer names `glam` - so the crates stay insulated exactly
as the palette plan insulated `palette`.

## The one structural difference from the palette plan

palette's types are generic over the component (`Srgba<T>`), so `type TySrgba<T>
= palette::Srgba<T>` worked. **glam's types are NOT generic** - `Vec3`, `DVec3`,
`IVec3`, `UVec3` are distinct concrete structs, no `Vec3<T>`. So the generic
`Ty*<T>` cannot survive as an alias. The owner has confirmed the generic `T` was
only a code-saving device (the `impl_ty_*_float!`/`_int!` macros already gate all
real behavior to concrete f32/f64/i32/u32), never used as a real parameter. So we
**drop the generic** and ship concrete aliases, with the bare name keeping today's
`T = f64` default. The entire consumer-side cost of that flip is ONE line
(`vxl/voxelize.rs:91 TyVector3<f64>`); see [consumer-census](reference/consumer-census.md).

## What "glam doesn't leak" means here

A `type` alias is transparent, so glam's method names (`length`, `normalize`,
`from_xyzw`), its `*`-as-Hadamard operator, and its `X/Y/Z` constants DO become
the vocabulary consumers write. That is accepted (same waiver as palette's field
renames), and welcomed - the owner prefers glam's names (see decisions).
"Doesn't leak" means the narrow, achievable thing: **no consumer crate names
`glam`.** ty-math re-exports every alias plus the extension traits that carry the
few methods glam lacks (`TyVector3Ext`, `TyQuaternionExt`, ...), so a consumer's
only math import is `ty_math::...`. That is the contract.

## Decisions

- **glam-only, one dependency (owner chose glam over glamx).** `glam = { version
  = "0.33", default-features = false, features = ["std", "float-types",
  "integer-types", "debug-glam-assert"] }`. glamx was evaluated (its `Pose3`/
  `MatExt`/`FloatExt` overlap ty-math thinly) and NOT adopted; its two useful
  ideas are lifted into ty-math with pure glam (see "Lift glam's algorithms").
  serde/`glam-assert` (all-builds) stay OFF. See [glam-facts](reference/glam-facts.md).
- **Adopt glam's fail-fast contract; enable `debug-glam-assert`.** Callers pass
  valid (unit) inputs; a violation panics loudly in debug/test at ZERO release
  cost, rather than ty-math defensively normalizing on every call. So glam's
  `from_axis_angle` (unit axis), `inverse` (unit quat), `normalize`,
  `is_normalized`, and `slerp` are used DIRECTLY - no guard/normalize wrappers.
  Call sites that supply a non-unit axis add an explicit `.normalize()` (the
  assert flags them during migration). This is faster in the long run and makes
  the migration self-checking. Do NOT re-add the old ty-math axis-normalize /
  zero-guard / non-unit-inverse defenses as silent overrides of the strict names.
- **Default to the strict glam methods; do NOT offer auto-fixing variants
  preemptively.** The strict methods are the architecture - they can be faster in
  the extreme (they inline to raw SIMD with no per-call branch/normalize) and keep
  one obvious meaning per name. Normalize explicitly at the ~5 call sites that need
  it. If an auto-normalizing / auto-fixing convenience ever proves genuinely
  recurring, it is a SEPARATELY, DISTINCTLY NAMED function with a clear
  prefix/suffix (mirroring glam's own `_or_zero` / `try_`, e.g.
  `from_axis_angle_normalized`), never a silent override of the strict name. That
  is a documented escape hatch, not a default.
- **Aliases (verified).** Concrete only: `TyVector3F64 = DVec3`, `...F32 = Vec3`,
  `...I32 = IVec3`, `...U32 = UVec3`; `TyQuaternionF64 = DQuat`; `TyMatrix4x4F64
  = DMat4`; bare `TyVector3/TyQuaternion/TyMatrix4x4` = the f64 form. Full table
  and per-method mapping in [glam-api-map](reference/glam-api-map.md).
- **Prefer glam's names over ty-math's legacy ones where they fit.** The DIRECT
  renames already do (`magnitude`->`length`, `normalized`->`normalize`, `new`->
  `from_xyzw`, `identity()`->`IDENTITY`, `UNIT_X`->`X`, `component(i)`->`[i]`).
  Extend it to the kept surface: the composites (`TyBounds`/`TyTransform`/
  `TyUniformTrs`/`TyPose`) expose an `IDENTITY` associated const (glam's
  convention), not an `identity()` fn. Keep a descriptive ty name only where glam
  has no equivalent.
- **Lift glam's better algorithms into the residue (pure glam, no glamx).**
  `from_basis_vectors`/`from_rotation_matrix` delegate to glam's tested
  `Quat::from_rotation_axes`, dropping ty-math's hand-rolled trace-branch;
  `rotate_extents_abs` becomes `DMat3::from_quat(q)` with per-column `.abs()` then
  `* extents` (glam vectors have `abs`), mirroring glamx's `MatExt::abs()` fold.
- **Residual methods ride EXTENSION TRAITS on the glam aliases (owner: use
  extension traits; repo style: one trait per file).** After the fail-fast
  decision the residue is small - only what glam genuinely lacks:
  `TyVector3Ext` (`triangle_normal`, `zup_to_yup`, `yup_to_zup`, `quantize`,
  `catmull_rom_position/tangent`), `TyQuaternionExt` (`to_euler_radians` [XYZEx
  convention wrapper], `from_rotation_matrix` [strips scale, delegates to glam],
  `from_basis_vectors`/`from_right_forward`/`from_right_up`, `rotate_extents_abs`,
  `canonicalized`). Re-exported so `use ty_math::TyVector3Ext` keeps `v.foo()`
  working. The trivial unused residue (`to_pure_quaternion`, `rotation_around`,
  `to_scale`, `from_x/y/z`, cosine `is_approximately_equal`, `rotate_towards`) are
  DROP candidates - glam covers or trivially inlines them; keep only on a found
  caller.
- **Composites stay hand-rolled ty-math structs.** glam has no AABB and no
  lossless TRS-with-quaternion-and-per-axis-scale (`Affine3` bakes scale+rotation
  into a 3x3 and cannot recover the fields; its `*` keeps the shear
  `TyTransform::compose` deliberately drops). So `TyBounds`, `TyTransform`,
  `TyUniformTrs`, AND `TyPose` stay structs whose fields become glam vectors/
  quaternions and whose bodies move onto glam ops. Concrete per precision (only
  `TyBounds` needs both f32 and f64; the rest are f64-only in practice).
- **Keep `TyFloatExt` verbatim** (glam is vector/matrix only; treegrid uses it as
  a generic bound). **Keep the serde DTO** (`TyVector3Serde {x,y,z}`); glam
  serializes as `[x,y,z]`, so the wire stays byte-identical.
- **Delete `array_conversions.rs`.** glam provides `new`/`from_array`/`to_array`/
  `from_slice`/`write_to_slice` natively on every vector and quaternion.

## Friction (eyes-open costs)

1. **Fail-fast shifts normalize to call sites.** glam `from_axis_angle` requires a
   UNIT axis and `inverse` assumes a unit quat; with `debug-glam-assert` on, a
   violation panics in tests. Real consumers pass caller-supplied axes
   (`from_vmax:645`, `to_vmax:1073`, `write_vmax:1445`, `hierarchy_show`,
   `tyt-vmax`); each becomes `DQuat::from_axis_angle(axis.normalize(), angle)`
   where the axis is not provably unit. The old zero-axis -> identity guard is
   gone; a site that can pass a zero axis handles it explicitly. This is the
   deliberate trade for dropping the per-call defensive normalize.
2. **Method-name renames are a repo-wide surface, decoupled from the alias flip.**
   `magnitude`->`length`, `magnitude_squared`->`length_squared`, `normalized`->
   `normalize`, `componentwise_multiply`->`*`, `componentwise_divide`->`/`,
   `component(i)`->`[i]`, `dot(&o)`->`dot(o)`, `cross(&o)`->`cross(o)`, quaternion
   `new`->`from_xyzw`, `identity()`->`IDENTITY`, `UNIT_X`->`X`. Rewrite by hand;
   `dot(&o)` vs `dot(o)` will not auto-fix.
3. **`scalar * Quat` is dropped.** glam has `Mul<f32> for Quat` but not `f32 *
   Quat`, and the orphan rule forbids re-adding it. Rewrite `s * q` -> `q * s`.
4. **Debug format churn.** `TyVector3 { x, y, z }` -> `DVec3(x, y, z)`. Any
   debug-string snapshot on a vector/quaternion re-baselines. (Hash/Eq: float
   vectors never had them; integer vectors keep them - only `TyVector3U32` is a
   map key, so safe.)
5. **`TyMatrix4x4` has no `.columns` array or `.get(row,col)`.** glam exposes
   `.col(i)` / `.x_axis..w_axis` and no element index. The only reader is
   ty-math-internal (`from_rotation_matrix`, now delegating to glam); no consumer
   calls `.get()` or `.columns`. `from_column_arrays` -> `from_cols_array_2d(&arr)`
   (add a leading `&`; the `[col][row]` layout matches).
6. **`round` on f32-SIMD** may round half-to-even vs ty half-away-from-zero. No
   consumer rounds an f32 vector (every `.round()` site is f64, scalar-backed and
   identical), so safe in practice; verify at the keyboard.

## Blast radius

ty-math math module (the 5 vector/quat/matrix families -> aliases + a small set of
extension traits; the 4 composites -> concrete glam-backed structs; delete
`array_conversions.rs`; keep `TyFloatExt`), plus ~30 voxsmith files, vxl
(hierarchy_show + voxelize), voxcore (vox_object + vox_main + vox_hierarchy_node),
tyt-fbx, tyt-injection, tyt-vmax, and `TyFloatExt`-only treegrid. See
[consumer-census](reference/consumer-census.md) for per-file sites. `ty-math-serde`'s
DTO is untouched (recompiles). No external wire changes: the `TyVector3` JSON is
pinned by the kept DTO; vmax/voxj/goxl/qb cross their boundaries as raw
`[f64;N]`/`[u32;N]` arrays and hex strings, and `.to_array()` still emits those.

## Commit strategy

The alias flip is ATOMIC, exactly as palette's was: the moment `TyVector3F64`
becomes `DVec3`, every `.magnitude()`, `.dot(&x)`, `componentwise_multiply`, and
composite-struct body breaks at once. There is no green intermediate. So: **one
clean commit, prepared across sessions** - work crate-by-crate in the working
tree, staged not committed, verifying sub-parts with `cargo check -p <crate>`, and
land one Conventional Commit when `cargo check --workspace` + clippy + tests are
green. Larger blast radius than palette; if the owner prefers, the ext-trait
scaffolding in ty-math can land first as an additive commit, but the alias flip +
consumer migration must still be one atomic commit. Bump ty-math `0.1.9 ->
0.1.10` (stay 0.1.x-patch so `^0.1` carets + `[patch.crates-io]` keep resolving,
mirroring palette's 0.1.8 -> 0.1.9).

## Not in scope

- `voxcore`'s `VoxValuePool` model and every voxel-codec wire format.
- Deleting the unused `TyPose`/`TyUniformTrs` (census shows zero consumers) - they
  migrate for API parity; a separate pass may remove them.
- Adopting glamx as a dependency, or glam's extras beyond parity (`try_inverse`,
  `project_point3`, SIMD `Vec3A`, the camera module) - available for free, but no
  consumer needs them yet; do not build features around them here.
- Replacing `TyFloatExt` with any external float trait (glam has none; it is a
  load-bearing generic bound in treegrid).
