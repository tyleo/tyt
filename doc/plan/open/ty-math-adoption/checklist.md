# ty-math Adoption Checklist

Tracks the three tracks from the [README](README.md): harmless cleanups with no
new API, a `ty-math` extension with adoption, and an investigation of the heavier
`internal/` logic. Line numbers are from the audit at the time of writing; confirm
them at the keyboard, since edits shift them. Every item is unchecked because no
work has started.

Log non-obvious code-level choices in
[reference/implementation-decisions.md](reference/implementation-decisions.md) as
they land.

## Ground rules

- Each track is a standalone branch off `main`. Within a track, land non-vmax work
  first and put every edit under `convert/vmax` or `internal/vmax` in its own
  trailing commit (Q2), because a second branch is editing vmax.
- Run `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`
  before every commit; the pre-commit hook enforces both.
- Follow the repo style: Rust edition 2024, consolidated nested `use`, one public
  type per file named in snake_case, doc comments on public items, comments
  wrapped to 80 columns and ASCII-only.
- Tracks A and B change no serialized bytes, so the workspace stays green with no
  golden churn. If a converter test's output changes, something is wrong; stop.

## Track A: harmless cleanups, no new API

Behavior-preserving. Each edit must be numerically identical to what it replaces.
No `ty-math` change.

### A1: non-vmax converters

- [x] Replace the four-component `color_floats` with `TySrgbaColor::from_array` and
      `to_rgba`/`to_array` in `convert/goxl/from_goxl_file.rs:172` (called `:84`)
      and `convert/mvox/from_mvox_file.rs:196` (called `:106`); delete both
      helpers.
- [x] Replace the test hex parsers with `TySrgbaColor::from_hex` in
      `convert/gltf/from_gltf_bytes.rs:839` (`rgb`),
      `convert/goxl/from_goxl_file.rs:289` (`srgba`),
      `convert/mvox/from_mvox_file.rs:565` (`srgba`), and
      `convert/qbcl/from_qbcl_file.rs:401` (`srgb`, three-component: use
      `from_hex(..).to_rgba().to_vector3().to_array()`).
- [x] Replace `TyVector3::new(node_scale, node_scale, node_scale)` with
      `splat(node_scale)` in `convert/voxelize/voxelize_mesh.rs:95`.
- [x] Replace `TyVector3F64::default()` with `ZERO` in `convert/voxelize/mesh.rs:46`
      (the zero-size-box branch), and the mvox scale literal
      `TyVector3F64::new(1.0, 1.0, 1.0)` with `ONE` where it is not later mutated.
- [x] Replace the `[bounds.x, bounds.y, bounds.z]` rebuilds with
      `bounds.to_array()` in `convert/qbcl/to_qb_file.rs:71`,
      `convert/qbcl/to_qbcl_file.rs:175` and `:382`, and
      `convert/qbcl/to_qbt_file.rs:169`.

Gate: `cargo test -p voxsmith` green, no golden change.

### A2: vmax converter (own commit, last in Track A)

- [x] Replace `color_floats` with `TySrgbaColor::from_array(*color).to_rgba()` in
      `convert/vmax/from_vmax_file.rs:374` (called `:266`) and `vec3` with
      `TyVector3F64::from_array` at `:594` (called `:623`, `:632`, `:633`); delete
      both helpers.
- [x] Replace the test `color_floats(hex)` with
      `TySrgbaColor::from_hex(..).to_rgba().to_array()` in
      `convert/vmax/to_vmax_file.rs:534`.

Gate: `cargo test -p voxsmith` green; the vmax edits stand alone in the log.

## Track B: extend ty-math, then adopt

Land each method with a unit test in the crate's existing float-macro form, then
adopt. Confirm final method names against Q3 and record them in the decision log.

### B1: ty-math additions

- [x] `TyVector3::round` (`ty_vector3.rs` float macro) and a `to_i32` truncating
      cast to `TyVector3<i32>`, so `round().to_i32()` rounds; unit test both.
- [x] `TyTransform::from_translation` (`ty_transform.rs` float macro): identity
      rotation, unit scale; unit test.
- [x] `TyBounds::from_points` and `size` (`ty_bounds.rs` float macro), the min/max
      fold and the full extent; unit test both, including the empty-iterator
      `None`.
- [x] `TyVector3::triangle_normal(a, b, c)` (`ty_vector3.rs`, generic over the
      cross bound); unit test against a known winding.
- [x] `TyQuaternion::from_rotation_matrix -> Option<Self>` (`ty_quaternion.rs`
      float macro), wrapping `from_basis_vectors` on the normalized matrix columns
      and returning `None` for a degenerate matrix (revises the plan's identity
      fallback); unit test that it inverts `TyMatrix4x4::from_quaternion` and that a
      degenerate matrix is `None`.
- [x] `TyFloatExt::to_unorm8` (`ty_float_ext.rs`), `(x.clamp(0, 1) * 255).round()
      as u8`; unit test the clamp and rounding.
- [x] `TySrgbaColor::to_vector3 -> TyVector3<f64>` (`ty_srgba_color.rs`), the
      alpha-dropping companion to `to_rgba`, named `to_vector3` to mirror
      `TyRgbaColor::to_vector3` (see the color-type follow-up in the README); unit
      test.

Gate: `cargo test -p ty-math` green with the new tests, and `cargo check` the
workspace so the patched consumers pick up the new methods.

### B2: adopt in the non-vmax converters

- [x] Replace the triplicated `translation([i32; 3]) -> TyTransformF64` with
      `TyTransform::from_translation` in `convert/qbcl/from_qb_file.rs:176`,
      `convert/qbcl/from_qbcl_file.rs:298`, and `convert/qbcl/from_qbt_file.rs:275`,
      and the `placed_at` test helper at `from_qbcl_file.rs:459`.
- [x] Replace mvox's `quaternion_from_matrix` (`from_mvox_file.rs:387`) and
      `determinant` (`:379`) with `TyQuaternion::from_rotation_matrix`, extracting
      the columns from the frame matrix; keep the mirror-detection that negates a
      column and sets `scale.x = -1.0`. `from_rotation_matrix` now returns
      `Option`, so `.expect("the frame is a proper rotation")` at the call site
      (the frame is always a proper rotation, so the old identity fallback was dead
      code). `ty-math` has no determinant, so the mirror check now uses the scalar
      triple product of the columns via `dot`/`cross` (exact for the
      signed-permutation frame).
- [ ] **Moved to the [ty-color-model plan](../ty-color-model/README.md).** The
      three-component qbcl `color_floats` in `convert/qbcl/from_qb_file.rs:185`,
      `from_qbt_file.rs:284`, and `from_qbcl_file.rs:307` resolve there with the new
      color types (their `[u8; 3]` sources needed a synthetic alpha, the RGB-vs-RGBA
      smell that plan owns), not here.
- [x] Replace the inline `round() as i32` world-position accumulation with
      `parent + position.round().to_i32()` in `convert/goxl/to_goxl_file.rs:153`,
      `convert/qbcl/to_qbcl_file.rs:298`, and mvox's `translation_of`
      (`to_mvox_file.rs:515`), threading positions as `TyVector3I32` where it stays
      typed end to end.
- [x] Replace the float-to-unorm8 idiom with `TyFloatExt::to_unorm8` in
      `convert/gltf/from_gltf_bytes.rs:443` and the test `byte` at `:738`.
- [x] Replace the duplicated triangle-normal winding test with
      `TyVector3::triangle_normal` in `convert/mesh/object_to_mesh_geometry.rs:280`
      and the test at `:440`.

Gate: `cargo test -p voxsmith` green, no golden change.

### B3: adopt in the vmax converter (own commit, last in Track B)

- [x] Vectorize the `object_transform` offset and position math (`:595`) using
      `to_f64`, `componentwise_multiply`, `Sub`, and `Add` on `TyVector3F64`.
      `from_rotation_matrix` does not apply (vmax stores an axis-angle rotation, not
      a matrix), and `to_rgba` was already adopted for the color pool in Track A2,
      so neither is a new change here.
- [x] Considered `component_min_with`/`component_max_with` for `min_corner` (`:474`)
      and `object_bounds` (`:487`): **not adopted.** Both methods are float-only
      (they live in the `impl_ty_vector3_float` macro), but the two functions work
      on `[i32; 3]` / `[u32; 3]`, so routing through them would add i32/u32 casts
      rather than remove any. Left as-is.

Gate: `cargo test -p voxsmith` green; vmax edits stand alone in the log.

## Track C: investigate the heavier logic under internal/

Investigation first. Read, catalog against the three patterns, adopt what fits, and
file the larger primitives as new items before building them. May span more than
one commit.

- [x] Read `internal/mesh/triangle_bounds.rs`, `internal/mesh/voxelize_triangles.rs`
      (and `triangle_box_overlap`, `clamp_index`), and `internal/mesh/sample_material.rs`;
      record where the three patterns appear. Cataloged in the decision log: pattern
      map per file plus the `ty-math` surface gates (items 2/3 are safe drop-ins;
      the `to_grid`, SAT overlap, `CellAccum` color sums, and barycentric blends
      need a `ty-math` addition first, to be filed in item 4).
- [ ] Adopt `TyBounds::from_points` in `triangle_bounds.rs` so it returns a
      `TyBounds` (or keeps the tuple but folds through the constructor), and adopt
      `TyVector3::triangle_normal` in the rasterizer where the geometric normal is
      computed. Assert the results match the prior code.
- [ ] Inspect `internal/cell_color.rs:10` (`cell_color -> [u8; 4]`): file whether it
      should return `TySrgbaColor`, and the ripple into the qbcl/goxl `to_*_file.rs`
      destructures, as its own item with the consumer list before changing the
      signature.
- [ ] File any larger geometry primitive the rasterizer wants (a triangle-box SAT
      overlap, point-in-triangle, or barycentric interpolation on `TyVector3`) as a
      new checklist item under Track C, with the current location and what it
      computes; decide per item whether it belongs on an existing type or a new
      one.
- [ ] Read `internal/vmax/write_vmax.rs` in its own commit: catalog the
      reverse-direction color emit (clamp-and-round to byte, a candidate for
      `TyRgbaColor::to_srgba`) and any nearest-palette color-distance metric (a
      candidate for a distance on `to_oklab`/`to_cielab`); adopt the safe ones and
      file the rest.

Gate: each landed change is covered and asserted against the prior behavior; each
filed item names its location and its target `ty-math` type.

## Track D: broad ty-math reuse audit (opened 2026-07-05)

A codebase-wide audit (all voxsmith `convert/` + `internal/`, `vxl`, `voxcore`,
`voxj-codec`) deduped 54 findings into 30 verified proposals. Full record,
including exact sites, verdicts, and the rejected list, is in
[reference/reuse-audit-findings.md](reference/reuse-audit-findings.md). Owner
decisions (see the [decision log](reference/implementation-decisions.md#track-d-broad-reuse-audit)):
build every approved new method; keep the 3-component qbcl `color_floats` helpers
deferred; **stage and pause per chunk** (do not auto-commit). Each `[ ]` below is
one reviewable, staged chunk. vmax edits (Track D3) trail in their own commits.

Shipped in `f2d4d5d`: `TyFloatExt::to_unorm8` (4 sites) and
`TySrgbaColor::to_vector3` at `voxelize_mesh.rs:371`.

### D1: new ty-math methods, land with adoption (non-vmax first)

Each method lands in its file's float-macro or generic form with a doc comment and
a unit test, then adopts at the non-vmax sites; vmax adoption trails in D3.

- [x] **C3** `TyVector3<u32>::to_f64`; adopted in vxl `hierarchy_show.rs`
      (`build`, `runtime_min`, `runtime_size`), deleting `vec_u32_to_f64` and
      dropping `TyVector3U32` from the module import. The sibling `vec_i32_to_f64`
      stays for A3.
- [x] **C2** `TyVector3<u32>::to_i32` and `TyVector3<i32>::to_u32` (concrete
      `impl` blocks); adopted at the vector-native sites `internal/grid.rs` (both
      casts, threading `copy_voxels`'s offset as `TyVector3I32`) and
      `convert/mvox/to_mvox_file.rs` (`(bounds / 2).to_i32()`). The
      `convert/goxl/to_goxl_file.rs:201` and
      `voxj_decoded_object_from_vox_object.rs` sites lift an input `[i32;3]` /
      `[u32;3]` array, so they defer to A6 (packing) where that array becomes a
      vector.
- [x] **C1** integer `component_min_with`/`component_max_with` via a concrete
      `impl_ty_vector3_int!` macro over `i32`/`u32` (a generic `impl<T: Ord>`
      fails E0592 against the float methods); adopted in voxcore
      `vox_object.rs` (`live_extent`). vmax `min_corner` trails in D3.
- [x] **C4** `TyQuaternion::is_normalized(self, tolerance)`; adopted in voxcore
      `vox_main.rs` (`!rotation.is_normalized(ROTATION_TOLERANCE)`). The voxj-codec
      `check_transforms.rs` site is DEFERRED: voxj-codec has no `ty-math`
      dependency, so adopting there means adding a cross-layer dep to a lean codec
      crate -- an owner decision, pending.
- [x] **C5** `zup_to_yup` / `yup_to_zup` axis rotations (`(x,z,-y)` / `(x,-z,y)`,
      permutation + sign) on `impl<T: Copy + Neg> TyVector3<T>`; adopted at the
      three glTF sites (`from_gltf_bytes` import, `object_to_gltf_document` and
      `material_document` export, the scale folding into the existing `Mul<T>`).
      Superseded the pure-swizzle idea: the sites are +/-90 rotations about X, so a
      bare permutation never fit.
- [x] **C7** `componentwise_divide` on `impl<T: Copy + Div> TyVector3<T>`; adopted
      in `internal/mesh/grid_space.rs`: retyped the private `size` to
      `TyVector3F64`, so `to_grid` is `(point -
      min).componentwise_divide(&size).to_array()` and `cell_center` folds through
      the existing `componentwise_multiply`.
- [ ] **C8 (moved)** -> [ty-color-model plan](../ty-color-model/README.md) step S9.
      The `to_linear_rgba` sRGB decode re-lands there as `TySrgba<f64>::to_lin_srgba`
      and adopts at the voxj `decode_rgb`/`decode_rgba` against the new color types;
      C8 was reverted from this plan.
- [x] **C6** `TyBounds::from_min_size(min, size)`; added with a test. Adoption is
      vmax-only (`write_vmax` content/object box), so it trails in D3.

### D2: adopt existing ty-math, non-vmax (no new API)

- [x] **A1** routed `triangle_box_overlap.rs` through `TyVector3F64`
      `Sub`/`dot`/`cross`; deleted the three private free fns. Kept the
      `[f64; 3]` public signature (no ripple into `to_grid`).
- [ ] **A2 (moved)** -> [ty-color-model plan](../ty-color-model/README.md). The
      `sample_material.rs` `CellAccum` retype accumulates color values and shares
      `sample_material` with the color-type migration, so it lands there (step S6)
      to avoid touching the file twice.
- [x] **A3** `TyVector3I32::to_f64` + `from_array`: vxl `hierarchy_show.rs`
      (deleted `vec_i32_to_f64`, `object.origin().to_f64()`) and mvox
      `from_mvox_file.rs` (`TyVector3I32::from_array(frame.translation).to_f64()`).
- [ ] **A4 (moved)** -> [ty-color-model plan](../ty-color-model/README.md). vxl
      `fill_color.rs:39`'s `parse_rgba_hex` -> `from_hex` adopts the sRGB color type,
      so it lands there (step S6) once `from_hex` is on `TySrgba<u8>`.
- [x] **A5** `TyVector3F32::INFINITY`/`NEG_INFINITY` consts at the two glTF AABB
      seeds (`object_to_gltf_document.rs`, `material_document.rs`).
- [x] **A6** `to_array`/`from_array`/`From<[T;N]>` packing at the 6 voxj pack/
      unpack sites (position-only partial at
      `vox_hierarchy_node_from_voxj_hierarchy_node.rs`). Also carried the C2 casts
      deferred here: `to_goxl_file.rs` (`world + position.to_i32()`, threading
      `world` as `TyVector3I32`) and `voxj_decoded_object_from_vox_object.rs`
      (`origin + min.to_i32()`, `min`/`size` kept as `TyVector3U32`).
- [x] **A7** `TyBounds::from_points` + `size()`/`max()` at
      `convert/voxelize/mesh.rs` (`extent`, `bounds.size()`) and the
      `object_to_glb_bytes.rs` test (`bounds.max()`). Left `triangle_bounds.rs`
      untouched (bit-risk: separately-halved center +/- extents can shift a cell
      boundary).
- [x] **A8** (optional, cosmetic) `TyVector3F64::ZERO` at vxl
      `hierarchy_show.rs` (the zero-size runtime grid).

### D3: adopt in vmax (own trailing commits)

- [x] **B1** deleted `write_vmax`'s `vector()` helper for `to_array()` at the
      three `VMax{Object,Group}` pack sites (`write_vmax.rs:1209,1386,1388`); the
      `TyVector3F64` import stays (six other sites use it). Audit line numbers had
      shifted; confirmed at the keyboard.
- [x] **B2** adopted `TyTransformF64::transform_point` in `subtree_box_local`
      (`write_vmax.rs:1251`), deleting the local hand-rolled TRS free fn (was
      `:1336`); the call wraps `child_center` through
      `TyVector3F64::from_array`/`to_array` at the `[f64;3]` boundary.
      `transform_half` was left as-is (per-column basis rotate, not a point
      transform).
- [x] **B3** the round-to-nearest chains now fold through
      `to_f64`/`Sub`/`Add`/`round`/`to_i32`/`from_array`/`to_array` in
      `pivot_origin` (`from_vmax_file.rs:545`) and `authored_box`'s `box_min`
      (`:585`). The `authored_box` `size` (`round().max(0.0) as u32`) stays as-is:
      it is the rejected `TyVector3F64::to_u32` case (no float-vector `to_u32`).
- [x] **B4** `min_corner` (`from_vmax_file.rs:490`) folds through C1's integer
      `component_min_with` on `TyVector3I32`, wrapping the `[i32;3]` positions with
      `from_array` and the result with `to_array`. `object_bounds` stays as-is
      (a subtract/+1/u32-cast fold, excluded by the C1 note).
- [x] **B5** `content_box` (`write_vmax.rs:599`) and both branches of
      `object_box_local` (`:1276`) build a `TyBoundsF64::from_min_size`, reading
      back `.center`/`.extents` (and `-extents` for content's min offset). The
      per-component `x / 2.0` becomes `from_min_size`'s `x * 0.5`, an exact
      power-of-two scaling, so it is bit-identical.
- [x] **B6** `extend_bounds` (`write_vmax.rs:1289`) vectorizes onto `TyVector3F64`:
      `lo = center - half`, `hi = center + half`, and the running `(min, max)`
      folds through float `component_min_with`/`component_max_with`, keeping the
      `([f64;3], [f64;3])` tuple representation. `TyBounds::encapsulate` was NOT
      used: its center/extents round-trip is not bit-exact against the raw min/max
      the caller reads back.

Gate: Tracks D1/D2 change no serialized bytes; `cargo test -p` the touched crates
must stay green with no golden churn. Each new method ships a unit test in its
`ty-math` file. vmax (D3) edits stand alone in the log.
