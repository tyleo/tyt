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

- [ ] Replace the triplicated `translation([i32; 3]) -> TyTransformF64` with
      `TyTransform::from_translation` in `convert/qbcl/from_qb_file.rs:176`,
      `convert/qbcl/from_qbcl_file.rs:298`, and `convert/qbcl/from_qbt_file.rs:275`,
      and the `placed_at` test helper at `from_qbcl_file.rs:459`.
- [ ] Replace mvox's `quaternion_from_matrix` (`from_mvox_file.rs:387`) and
      `determinant` (`:379`) with `TyQuaternion::from_rotation_matrix`, extracting
      the columns from the frame matrix; keep the mirror-detection that negates a
      column and sets `scale.x = -1.0`. `from_rotation_matrix` now returns
      `Option`, so `.expect("the frame is a proper rotation")` at the call site
      (the frame is always a proper rotation, so the old identity fallback was dead
      code).
- [ ] Replace the three-component `color_floats` with `TySrgbaColor::to_vector3`
      (then `.to_array()` for the `[f64; 3]` pool) in
      `convert/qbcl/from_qb_file.rs:185`, `convert/qbcl/from_qbt_file.rs:284`, and
      `convert/qbcl/from_qbcl_file.rs:307`; delete the helpers.
- [ ] Replace the inline `round() as i32` world-position accumulation with
      `parent + position.round().to_i32()` in `convert/goxl/to_goxl_file.rs:153`,
      `convert/qbcl/to_qbcl_file.rs:298`, and mvox's `translation_of`
      (`to_mvox_file.rs:515`), threading positions as `TyVector3I32` where it stays
      typed end to end.
- [ ] Replace the float-to-unorm8 idiom with `TyFloatExt::to_unorm8` in
      `convert/gltf/from_gltf_bytes.rs:443` and the test `byte` at `:738`.
- [ ] Replace the duplicated triangle-normal winding test with
      `TyVector3::triangle_normal` in `convert/mesh/object_to_mesh_geometry.rs:280`
      and the test at `:440`.

Gate: `cargo test -p voxsmith` green, no golden change.

### B3: adopt in the vmax converter (own commit, last in Track B)

- [ ] Adopt `from_rotation_matrix` and, if landed as part of B1, `to_rgb`/`to_rgba`
      in `convert/vmax/from_vmax_file.rs`, and vectorize the `object_transform`
      offset and position math (`:610`) using `componentwise_multiply`, `Sub`, and
      `Add` on `TyVector3F64`.
- [ ] Consider `component_min_with`/`component_max_with` for `min_corner` (`:479`)
      and `object_bounds` (`:491`); these are borderline because of the i32/u32
      casts, so adopt only if it reads cleaner, and note the call in the log.

Gate: `cargo test -p voxsmith` green; vmax edits stand alone in the log.

## Track C: investigate the heavier logic under internal/

Investigation first. Read, catalog against the three patterns, adopt what fits, and
file the larger primitives as new items before building them. May span more than
one commit.

- [ ] Read `internal/mesh/triangle_bounds.rs`, `internal/mesh/voxelize_triangles.rs`
      (and `triangle_box_overlap`, `clamp_index`), and `internal/mesh/sample_material.rs`;
      record where the three patterns appear.
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
