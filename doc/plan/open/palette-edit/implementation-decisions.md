# Implementation decisions

Code-level choices a reviewer of the Rust would want explained, recorded as
they land.

## S1. f64 language

- voxsmith checks the f32 range in `check_f32_range` and leaves the rounding
  to the glTF writer, which already narrows every float with `as f32`. The
  meshdoc keeps full `f64` values. Every material factor goes through the
  check, and only the unbounded `normalScale`, `emissiveStrength`,
  `alphaCutoff`, and `ior` can fail it. Custom float attributes go through it
  after the transfer. `COLOR_0` lies in `[0, 1]` and skips it.
- `CHROMA_FLOOR` stays at `1e-6`. With the `f32` narrowing gone, the rounded
  Oklab matrices leave at most about `1.7e-7` of chroma on a gray up to a
  linear `100`.
- `assert_close` keeps its `1e-5` relative tolerance because several tests
  compare against hand-typed reference values.
- Greedy merge class keys split an `f64` component's bits into two `u32`
  words.
- voxsurface's `corner_occlusion` and `mesh_occlusion` return `f64`, so the
  occlusion thirds reach the language and voxrender exact. Vertex positions
  stay `f32`.
