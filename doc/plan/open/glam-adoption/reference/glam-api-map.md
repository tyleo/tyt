# glam API map

The verified mapping from ty-math's hand-rolled math surface to glam 0.33.
Source-verified against docs.rs and the glam repo; re-confirm at the keyboard.
This is the factual backbone for the [checklist](../checklist.md); the
[README](../README.md) records the decisions and [glam-facts](glam-facts.md) the
crate facts.

Legend: **DIRECT** = glam has it (maybe renamed / an operator / a `X` const), the
ty method is deleted and call sites move onto glam. **EXT** = no glam equivalent,
stays a ty-math extension trait on the glam alias (one trait per file, per
CLAUDE.md), so callers keep `v.foo()` with a `use ty_math::TyVectorExt`. **DROP** =
no analogue and no consumer (glam covers it or it inlines trivially).

Contract (owner): ty-math adopts glam's fail-fast model with `debug-glam-assert`
ON. Callers pass valid (unit) inputs; a violation panics in debug/test. So the old
ty-math defensive normalize/guard wrappers are NOT recreated - glam's
`from_axis_angle`/`inverse`/`normalize`/`is_normalized`/`slerp` are used directly.
Prefer glam's names for the kept surface where they fit. Any auto-normalizing /
auto-fixing convenience is a DISTINCTLY NAMED variant (prefix/suffix, like glam's
`_or_zero` / `try_`), never a silent override of the strict base.

## Type aliases (the re-exports)

Concrete only - the generic `T` is gone. The bare name keeps today's `T = f64`
default.

```
pub type TyVector2   = glam::DVec2;  pub type TyVector2F64 = glam::DVec2;  TyVector2F32 = Vec2;
pub type TyVector3   = glam::DVec3;  pub type TyVector3F64 = glam::DVec3;  TyVector3F32 = Vec3;
                                     pub type TyVector3I32 = glam::IVec3;  TyVector3U32 = UVec3;
pub type TyVector4   = glam::DVec4;  pub type TyVector4F64 = glam::DVec4;  TyVector4F32 = Vec4;
pub type TyQuaternion= glam::DQuat;  pub type TyQuaternionF64 = DQuat;     TyQuaternionF32 = Quat;
pub type TyMatrix4x4 = glam::DMat4;  pub type TyMatrix4x4F64  = DMat4;     TyMatrix4x4F32  = Mat4;
// TyPose / TyBounds / TyTransform / TyUniformTrs: hand-rolled ty-math structs
// (no glam analogue) holding glam fields - see "Composites".
```

## Vectors (DIRECT unless noted)

| ty-math | glam | note |
|---|---|---|
| `new`, `splat`, `from_array`, `to_array`, `from_slice`, `write_to_slice` | same names | glam has all five natively -> `array_conversions.rs` is DELETED |
| `From<[T;N]>` / `From<Self> for [T;N]` | same | glam adds tuple `From` too |
| `component(i)` (panics) | `v[i]` (`Index<usize>`, panics) | reads by Copy |
| `dot(&o)` / `cross(&o)` | `dot(o)` / `cross(o)` | by value; `cross` on V3/IVec3/UVec3 |
| `componentwise_multiply(&o)` | `a * o` | glam `Vec*Vec` is Hadamard |
| `componentwise_divide(&o)` | `a / o` | glam `Vec/Vec` is per-component |
| `magnitude` / `magnitude_squared` | `length` / `length_squared` | float only; `length_squared` on ints too |
| `normalized` | `normalize` | zero -> non-finite; `debug-glam-assert` catches it in tests |
| `component_min_with` / `component_max_with` | `min` / `max` | pairwise component min/max (NOT `min_element`); on ints too |
| `abs`, `round`, `lerp` | `abs`, `round`, `lerp` | round: f64 scalar-backed matches half-away-from-zero; f32-SIMD may differ (no consumer rounds an f32 vec - safe, verify) |
| `to_f64` / `to_i32` / `to_u32` | `as_dvec3` / `as_ivec3` / `as_uvec3` | `as` per element: trunc-toward-zero, wrap on int->int (matches) |
| `ZERO` `ONE` `INFINITY` `NEG_INFINITY` | same | int families lack the float ones |
| `UNIT_X/Y/Z` / `UNIT_NEG_X/Y/Z` | `X/Y/Z` / `NEG_X/Y/Z` | renamed; UVec3 lacks `NEG_*` |
| V2 `cross` (perp-dot) | `perp_dot` | renamed |
| V2 `to_vector3` (z=0) | `v.extend(0.0)` | renamed |
| V4 `from_vector3(v3,w)` / `truncate` | `v3.extend(w)` / `truncate` | extend lives on Vec3 |
| `triangle_normal(a,b,c)` (unnormalized) | - | **EXT** (glam idiom adds `.normalize()`; keep unnormalized) |
| `zup_to_yup` / `yup_to_zup` | - | **EXT** |
| `quantize(low,high,buckets)` | - | **EXT** |
| `catmull_rom_position/tangent` | - | **EXT** (glam has no spline on vectors) |
| `rotate_towards(target)->Quat` | `Quat::from_rotation_arc(from,to)` | **DROP** (glam's shortest-arc, unit inputs, assert) - inline at any caller |
| `to_pure_quaternion`, `rotation_around`, `to_scale`, `from_x/y/z`, `is_approximately_equal` (cosine), `is_normalized_approximately_equal` | trivial / `Quat::from_axis_angle` / `v.x` / `Vec3::X*s` | **DROP** candidates (all unused by consumers; inline on a found caller) |

Active consumer-used EXT vector methods: `triangle_normal`, `zup_to_yup`,
`yup_to_zup`, `quantize`. The rest have no current caller.

## Quaternion

| ty-math | glam | class | note |
|---|---|---|---|
| `new(x,y,z,w)` | `from_xyzw` | DIRECT | rename; 2 consumer sites |
| `to_array`/`from_array`/`from_slice`/`write_to_slice` | same | DIRECT | `[x,y,z,w]` |
| `identity()` | `IDENTITY` (const) | DIRECT | fn -> const at every call site |
| `magnitude`/`magnitude_squared`/`normalized` | `length`/`length_squared`/`normalize` | DIRECT | |
| `conjugate`, `dot`, `vector_part` | `conjugate`, `dot`, `xyz` | DIRECT | |
| `rotate(v)` | `q * v` (`Mul<Vec3>`) | DIRECT | same active rotation, order matches |
| `Mul` (Hamilton), `Neg`, `Add`, `Sub`, `Mul<f32>` | same | DIRECT | product byte-identical, rhs applied first |
| `slerp_towards(to,t)` | `slerp` | DIRECT | glam does the same shortest-arc flip + nlerp fallback |
| `to_matrix4x4()` | `DMat4::from_quat(q)` | DIRECT | verify column parity with a q->M->q round-trip |
| `Default` = identity | `default()` = IDENTITY | DIRECT | |
| `from_axis_angle(axis,angle)` | `from_axis_angle` (unit axis) | DIRECT | **fail-fast:** glam needs a unit axis; `debug-glam-assert` catches a non-unit one. Call sites `axis.normalize()` where not provably unit. Drop the old ty normalize+zero-guard; if a normalizing ctor recurs, add a distinctly-named `from_axis_angle_normalized`, not a silent one |
| `inverse()` | `inverse()` (unit) | DIRECT | glam = conjugate assuming unit; relative-pose/trs hold that invariant. Drop the non-unit ext |
| `is_normalized(tol)` | `is_normalized()` (fixed ~1e-6) | DIRECT | prefer glam's; add a thin ext ONLY if a caller needs a custom tolerance (vox_main passes one - confirm) |
| `to_euler_radians()` | `DVec3::from(q.to_euler(EulerRot::XYZEx))` | EXT | wrap the tuple; `XYZEx` is the exact `Rz*Ry*Rx`, roll=x/pitch=y/yaw=z. Keep `TyQuaternionExt::to_euler_radians` so `EulerRot` never leaks |
| `from_basis_vectors(r,u,f)` | `Quat::from_rotation_axes(r,u,f)` | EXT-thin | **lift glam's algorithm** (r->x_axis,u->y_axis,f->z_axis); drop ty's trace-branch. Assumes orthonormal (assert) |
| `from_rotation_matrix(M)->Option` | glam `from_mat4` + our scale-strip | EXT | reads 3 upper-left cols, normalizes each (SEMANTIC - strips scale, not defensive), builds via `Quat::from_rotation_axes`. Keep `None` on degenerate only if `from_mvox` consumes it; else assert |
| `from_right_forward` / `from_right_up` | - | EXT | derive the third basis vector via cross (unit inputs), then `from_rotation_axes` |
| `rotate_extents_abs(extents)` | - | EXT | `let m = DMat3::from_quat(q); DMat3::from_cols(m.x_axis.abs(), m.y_axis.abs(), m.z_axis.abs()) * extents` (lifts glamx's `MatExt::abs` fold, pure glam) |
| `canonicalized()` (w>=0) | - | EXT | `if q.w >= 0 { q } else { -q }` |
| `rotate_around_axis(axis,angle)` | - | DROP | inline `DQuat::from_axis_angle(axis, angle) * self` at any caller |
| `Mul<Quat> for f32` (scalar*q) | - | DROP | orphan rule forbids re-adding; rewrite `s * q` -> `q * s` |

## Matrix (TyMatrix4x4 -> DMat4/Mat4, DIRECT)

| ty-math | glam | note |
|---|---|---|
| `new([TyVector4;4])` | `from_cols(x,y,z,w)` | 4 args, not one array |
| `from_column_arrays([[T;4];4])` | `from_cols_array_2d(&arr)` | same `[col][row]` col-major; glam takes `&` (add a leading `&`) |
| `to_column_arrays()` | `to_cols_array_2d()` | same layout |
| `identity()` | `IDENTITY` (const) | |
| `from_quaternion(q)` | `from_quat(q)` | same 3x3 build |
| `transform_point(v)` / `transform_vector(v)` | `transform_point3` / `transform_vector3` | match (w=1 / w=0) |
| `Mul<Vec4>` / `Mul<Self>` | `Mat4 * Vec4` / `Mat4 * Mat4` | col-major, rhs applied first |
| `Default` = identity | `default()` = IDENTITY | |
| `columns[i]` (field) | `.col(i)` / `.x_axis..w_axis` | glam has no `.columns` array; ty-internal reads rewrite to `.x_axis.truncate()` etc. |
| `get(row,col)` | `m.col(col)[row]` | no consumer calls `.get()`; drop it (or a `TyMatrix4x4Ext::get` if wanted) |
| - | `determinant`, `inverse`, `try_inverse`, `transpose` | pure gain from glam |

## Composites (hand-rolled ty-math structs, glam-backed; `IDENTITY` const, not `identity()` fn)

glam has no AABB and no lossless TRS-with-quaternion-and-per-axis-scale; so these
stay ty-math structs whose fields become glam types and whose bodies move onto
glam ops. Concrete per precision (drop the generic). Give each an `IDENTITY` /
`ZERO` associated const to match glam's naming (const-constructible since
`DVec3::ZERO`/`DQuat::IDENTITY`/`DVec3::ONE` are consts).

- **TyBounds** {center, extents}: needs BOTH f32 and f64 (census). Two concrete
  structs (or the float-macro) over `Vec3`/`DVec3`. `from_points` fold uses
  `a.min(b)`/`a.max(b)`; `scale`/`size`/`min`/`max`/`from_min_size`/`encapsulate`
  use glam `+ - *`. `#[derive(Default)]` works.
- **TyTransform** {position, rotation, scale: DVec3} (T*R*S): f64-only in practice.
  `transform_point` = `position + rotation * (scale * point)`; `compose` keeps the
  DELIBERATELY lossy per-axis `scale * child.scale` (Hadamard) - do NOT switch to
  `Affine3` (its `*` keeps the shear ty drops, and it cannot recover the
  quaternion+per-axis-scale fields). `IDENTITY` const = 0/ID/ONE.
- **TyUniformTrs** {translation, rotation, scale: f64}: unused by consumers. Same
  shape; scalar scale stays a plain `f64`.
- **TyPose** {position, rotation}: unused by consumers. Hand-rolled struct like the
  others (NOT glamx `DPose3`). `calculate_relative_pose` = `let inv =
  self.rotation.inverse(); Self::new(inv * (target.position - self.position), inv *
  target.rotation)` (glam `DQuat * DVec3` rotates, `DQuat * DQuat` composes, glam
  `inverse` under the unit invariant); `transform_aabb_conservative` uses
  `rotate_extents_abs`; `with_uniform_scale` builds `TyUniformTrs`. `IDENTITY`
  const.

## TyFloatExt - KEEP verbatim

glam is vector/matrix only; no scalar float trait. `quantize`,
`is_approximately_equal`, `wrap01`, `lerp`, `to_unorm8` all stay. `TyFloatExt` is
also load-bearing as a generic bound in treegrid's public fns, so it must survive
as a named, re-exported symbol. (The vector `.lerp` maps to glam's vector `lerp`;
the SCALAR trait stays.)

## Serde - KEEP the DTO

glam serializes a vector as a flat sequence `[x,y,z]`, not `{x,y,z}`. The
`serializes_to_stable_json` test pins the object form. So keep `TyVector3Serde
{x,y,z: f64}` and its two `From` impls; glam `serde` feature stays OFF. Both
`From` bodies are unchanged: `From<TyVector3>` reads `v.x/v.y/v.z` (public on
`DVec3`), `From<TyVector3Serde>` is `DVec3::new(v.x,v.y,v.z)` (const fn, exists).
Orphan rule is safe (the DTO is local on one side of each `From`). `TySrgbaSerde`
is untouched (palette color, not glam).

## Re-exports ty-math must add

```
// the Ty* aliases already flow out via the per-file `pub use ...::*;`
pub use ty_vector3_ext::*;   // + TyVector2Ext / TyVector4Ext / TyQuaternionExt (+ TyMatrix4x4Ext if kept)
// no glam symbol is re-exported by name; the to_euler_radians wrapper hides
// EulerRot. Consumers import ty_math::* and never name glam.
```

## Behavior deltas to re-baseline (owner relaxed byte-exactness, mirror palette)

- **Debug format:** ty `TyVector3 { x, y, z }` -> glam `DVec3(x, y, z)`. Any
  debug-string snapshot on a vector/quaternion re-baselines.
- **Hash/Eq:** float vectors/quats lose nothing (ty floats had none); integer
  vectors keep `Eq`+`Hash` (`IVec3`/`UVec3`). Census: only `TyVector3U32` is a map
  key -> fine. Confirm no float-vector HashMap key.
- **round on f32-SIMD** may be half-to-even vs ty half-away-from-zero. No consumer
  rounds an f32 vector (all `.round()` sites are f64) - safe; note it.
- **near-gimbal euler** may differ slightly (glam's handling vs ty's asin-clamp);
  re-baseline any exact euler assertion.
