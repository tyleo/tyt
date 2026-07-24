# Consumer census

Every consumer of a ty-math MATH type, from a workspace grep plus a per-cluster
read (2026-07-23). Line numbers are from that snapshot; confirm at the keyboard.
Color types (`TySrgba*`/`TyLinSrgb*`/`TyHexColor`) are excluded - they already
moved to palette. `voxcore`'s `VoxValuePool` and every voxel-codec wire is
untouched (raw arrays / scalars cross those boundaries; `.to_array()` still emits
`[T; N]`).

## Types actually used, and the ones that are dead

Consumers touch only `TyVector2F64`, `TyVector3` (F64/F32/I32/U32 + bare),
`TyVector4` (never by name - only inside the matrix), `TyQuaternionF64`,
`TyMatrix4x4F64`, `TyTransformF64`, `TyBoundsF64`/`TyBoundsF32`, and `TyFloatExt`.
**`TyPose*` and `TyUniformTrs*` have zero consumers anywhere** (migrate for API
parity, or a future pass may delete them). No consumer uses an f32 form of
Quaternion / Matrix4x4 / Transform / Pose / UniformTrs / Vector2 / Vector4.

Approx repo-wide reference tally (incl. ty-math + tests):
`TyVector3U32` 335, `TyVector3F64` 137, `TyTransformF64` 82, `TyVector3I32` 39,
`TyVector2F64` 23, `TyQuaternionF64` 23, `TyVector3F32` 21, `TyMatrix4x4F64` 7,
`TyBoundsF64` 6, `TyBoundsF32` 2.

## Crates in the blast radius

Depend on ty-math for math: **voxsmith** (heaviest, ~30 files), **vxl**,
**voxcore**, **tyt-fbx**, **tyt-injection**, **tyt-vmax**, and **treegrid**
(`TyFloatExt` bound only). Plus **ty-math-serde** (DTO). Every other crate
(all voxel-codecs, treeselect, the rest of tyt) has no ty-math dependency.

## The alias-flip breakage is ONE site

Making the aliases concrete (dropping the generic `T`) breaks exactly one
consumer line: `vxl/src/implementation/voxelize.rs:91`
`fn resolve_grid(extent: TyVector3<f64>)` -> rewrite to `TyVector3F64` (a
non-generic alias rejects `<f64>`). Every other `Ty*<...>` angle-bracket use is
inside ty-math itself. Bare `TyVector3` / `TyQuaternion` sites (tyt-fbx,
tyt-injection, voxsmith `voxelize_mesh`/`voxj` node builder, voxcore tests) keep
resolving because the bare alias becomes the f64 glam type. They still need the
method-body renames below (the larger surface), but the alias flip alone does not
break them.

## Cluster A - voxsmith/convert (~14 files)

- `gltf/from_gltf_bytes.rs` - `TyMatrix4x4F64::identity/from_column_arrays`,
  `.transform_point(..)`, `.yup_to_zup()`; `TyVector2F64::new`;
  `Vec<TyVector3F64>`; `.to_unorm8()`; many `TyVector3U32::new` (test).
  Field reads (test) `extent.x/.y/.z`, `transform.scale.z`.
- `gltf/object_to_glb_bytes.rs` - `TyBoundsF32::from_points(..).max()`,
  `TyVector3F32::from_array`, `.max()` corner then `max.x/.y` (test).
- `goxl/{from,to}_goxl_file.rs` - `TyTransformF64::{default,new}`,
  `TyVector3F64::new`, `TyQuaternionF64::identity`; `TyVector3I32/U32::new`;
  ops `parent + position.round().to_i32()`, `world.to_array()`.
- `mvox/from_mvox_file.rs` - `TyMatrix4x4F64::from_column_arrays`,
  `TyQuaternionF64::{from_rotation_matrix,identity}`, `TyTransformF64::{default,
  new}`, `TyVector3I32::from_array(..).to_f64()`, **`scale.x = -1.0` WRITE**,
  `.dot(&col.cross(&col))`, `TyVector3F64::{new,ONE}`.
  NON-SITE: `voxel.x/.y/.z/.color_index` is MagicaVoxel, not ty-math.
- `mvox/to_mvox_file.rs` - `position.round().to_i32().to_array()`, `bounds/2`,
  field reads on `TyVector3U32`.
- `qbcl/{from_qb,from_qbcl,from_qbt,to_qb,to_qbcl,to_qbt}_file.rs` -
  `TyTransformF64::{default,from_translation}`, `TyVector3I32/U32::new/from_array`,
  `bounds.to_array()`, `position.x/.y/.z` reads, `parent + position.round().
  to_i32()`.
- `vmax/{from,to}_vmax_file.rs` - `TyQuaternionF64::{from_axis_angle,identity}`,
  `TyTransformF64::{new,default}`, `TyVector3F64::{new,from_array}`,
  `.componentwise_multiply(&scale)`, `rotation.rotate(offset)`, quaternion field
  reads in tests. NON-SITE: `object.center/.scale` is `[f64;3]` from the codec.
- `voxelize/mesh.rs` - `TyBoundsF64::from_points`, `TyVector3F64::ZERO`,
  `fn extent -> TyVector3F64`.
- `voxelize/voxelize_mesh.rs` - **bare `TyVector3::splat(node_scale)`** (f64
  inferred), `TyVector3U32` fn params, `counts.x/.y/.z`.

Cluster A frictions: the `to_i32`/`to_f64`/`to_array`/`round` casts and the one
`componentwise_multiply` are the recurring edits; the `from_column_arrays` calls
need a leading `&`; leave every foreign `[f64;3]`/`voxel.*` alone.

## Cluster B - voxsmith/internal + reduce_palette (~15 files)

- `internal/mesh/{mesh_geometry,mesh_triangle,mesh_triangle_uvs,grid_space,
  sample_material,triangle_bounds,triangle_box_overlap,voxelize_triangles}.rs` -
  STRUCT FIELDS typed `Vec<TyVector3F32>` / `[TyVector3F64;3]` /
  `Option<[TyVector2F64;3]>` / `TyVector3F64`; `.cross(..)`, `.dot(&..)`,
  `.componentwise_multiply(&size)`, `TyVector3F64::{new,from_array}`,
  `TyVector2F64::new`, field reads `uv.x/.y`, `points[i].x/.y/.z`, `axis.x/.y/.z`.
- `internal/gltf/{material_document,object_to_gltf_document,bake_atlas}.rs` -
  `p.zup_to_yup() * scale`, `n.zup_to_yup()`, `TyVector3F32::{INFINITY,
  NEG_INFINITY}`, `.to_array()`, `.to_unorm8()`.
- `internal/grid.rs` - `TyVector3U32::new`, `origin + min.to_i32()`,
  `(p.to_i32() + offset).to_u32()`.
- `internal/vmax/write_vmax.rs` - `TyBoundsF64::from_min_size`,
  `box_local.center/.extents` reads + `-box_local.extents`,
  `.transform_point(..).to_array()`, `TyQuaternionF64::from_axis_angle`,
  `rotation.rotate(offset)`, heavy `transform.scale.x/.y/.z` reads.
- `internal/voxj/*.rs` - **bare `TyVector3::from_array`/`TyQuaternion::new`/
  `TyVector3::new`** (f64 inferred) in the node builder; `TyVector3U32::{new,
  from_array}`, `transform.rotation.to_array()` (quaternion), `.to_unorm8()`.
- `reduce_palette.rs` - STRUCT FIELD `coords: TyVector3F64`;
  `TyVector3F64::{new,default,INFINITY,NEG_INFINITY,ZERO}`, `.component(axis)`,
  `.quantize(low,high,BUCKETS).to_array()` (VECTOR quantize),
  `.magnitude()`/`.magnitude_squared()`, `*slot = *slot + error*weight`,
  `.to_unorm8()` (test).

Cluster B frictions: `magnitude`->`length`, `componentwise_multiply`->`*`,
`component(i)`->`[i]`, `quantize` (vector) stays an EXT; `zup_to_yup`/
`triangle_normal` stay EXT. The `resolve_*` files are test-only `TyVector3U32::new`.

## Cluster C - vxl + voxcore (5 files)

- `vxl/hierarchy_show.rs` - `TyTransformF64::{new}`, `.compose(&t)`,
  `.transform_point(v)`, `.rotation.to_euler_radians()`, `.position/.scale`
  reads, `TyVector3F64::ZERO`, `TyQuaternionF64::from_axis_angle` (test).
- `vxl/voxelize.rs` - **`TyVector3<f64>` explicit generic at :91** (the one
  breaking site), `TyVector3U32::new`, `counts.x/.y/.z`.
- `voxcore/vox_object.rs` - PUBLIC method sigs take/return `TyVector3U32`/
  `TyVector3I32` (consumed by vxl); private struct fields `bounds`/`origin`;
  `.x/.y/.z` reads, `as u64` casts.
- `voxcore/vox_main.rs` - `fn vector_is_finite(TyVector3F64)`,
  `node.transform.position/.scale/.rotation` reads, `rotation.is_normalized(tol)`;
  tests build bare `TyVector3`/`TyQuaternion`.
- `voxcore/vox_hierarchy_node.rs` - **`pub transform: TyTransformF64`** (field,
  crossed by vxl/voxsmith).

## Cluster D - tyt-fbx + tyt-injection + tyt-vmax + treegrid + serde

- `tyt-fbx/commands/create_point_cloud.rs` - bare `TyVector3`; `::new`, `*`, `-`,
  `.cross(&v)`, `.dot(&v)`, `.magnitude()`, `.x/.y/.z` reads AND writes
  (`min.x = v.x`).
- `tyt-fbx/dependencies.rs` / `dependencies_impl.rs` / `utilities/
  mesh_with_uvs.rs` - PUBLIC `trait Dependencies::serialize_points_and_colors_json
  (points: &[TyVector3], ...)` and `pub type MeshWithUvs = (Vec<TyVector3>, ..)`.
  Compile unchanged once bare `TyVector3` = `DVec3`.
- `tyt-injection/{mesh_with_uvs,parse_mesh_with_uvs_json,
  serialize_points_and_colors_json}.rs` - bare `TyVector3` in pub sigs;
  `TyVector3Serde`/`TySrgbaSerde` bridge via `.from()`; the pinned
  `serializes_to_stable_json` `{x,y,z}` test (guards the DTO).
- `tyt-vmax/implementation/dependencies_impl.rs` - `TyTransformF64::new`,
  `TyQuaternionF64::from_axis_angle`, `TyVector3F64::{new,from_array}`,
  `.position/.scale.to_array()`, `.rotation.to_euler_radians().to_array()`. All
  private fns. NON-SITE: `node.position/.rotation` here are `[f64;N]` codec arrays.
- `treegrid/color/{tree_grid_value,tree_grid_json_value}.rs` - `TyFloatExt` as a
  generic BOUND in public fns (color types otherwise). Untouched by the math flip
  except that `TyFloatExt` must stay a re-exported symbol.
- `ty-math-serde/ty_vector3_serde.rs` - the DTO; `From` bodies read `.x/.y/.z`
  (unchanged on `DVec3`).

## Traps (leave alone / watch)

- Foreign look-alikes: MagicaVoxel `voxel.x/.y/.z`, VMax `object.center/.scale`
  (`[f64;3]`), ObjectPlacement arrays, voxj/voxcore `Srgba*` enum VARIANTS. All
  NOT ty-math.
- Value-source methods with no type name on the line: `object.bounds()/.origin()`
  -> `TyVector3U32/I32`; `node.transform.position/.scale` -> `TyVector3F64`;
  `.rotation` -> `TyQuaternionF64`; `TyBoundsF32::from_points(..).max()`. Trace
  return types, not just literal `Ty*` tokens.
- Quaternion `.x/.y/.z/.w` reads (tests) at to_vmax and the voxj node builder -
  easy to mistake for vectors; glam keeps the fields so they survive.
- The one field WRITE beyond reads: `mvox/from_mvox_file.rs` `scale.x = -1.0`
  (glam fields are writable - fine).
