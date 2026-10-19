# Checklist

The design is in the [README](README.md). The research's section 7 holds each
benchmark's setup, threshold, and outcomes. Check steps off as they land. Log
code-level choices in
[implementation-decisions.md](implementation-decisions.md) and measurements in
[benchmarks.md](reference/benchmarks.md).

## Ground rules

- Each step is staged for review before the next starts.
- The contract page changes in the step that changes the behavior it
  describes, never ahead of it.
- Every golden stays within tolerance at every step, except where a step
  changes a rule and regenerates the goldens. That step lists what moved.
- Timing follows the research's common setup in section 7.
- A benchmark records its run in benchmarks.md before applying its threshold
  as written.
- Tests are inline `#[cfg(test)] mod tests` per file.

## Steps

- [x] **S1. Integer traversal in the reference.** `voxrender`'s walk becomes
      the integer DDA.
      1. A quantization module turns each placement's object-space ray into a
         fixed-point origin with 13 fraction bits and a 16-bit integer
         direction. Each axis is mirrored as needed so every direction
         component is non-negative. Primary directions come from a per-view
         base and per-pixel steps with guard bits
      2. The walk picks each step's axis by the sign of an integer error
         term. A tie goes to x, then y, then z. Entry from far outside the
         grid runs in i64
      3. A corner shadow ray starts at its lattice point inset by 2^-10. A
         point on a cell boundary belongs to the cell the ray moves into. The
         inset and the boundary rule replace `BIAS` and `INSET` in
         `render.rs`
      4. Hit distances for shading and for merging placements are computed
         in f64 from the integer state
      5. Tests: the precision-sim scenes match an exact rational walk, ties
         break in axis order, and the goldens regenerate with a list of what
         moved
      6. The contract's Surface and Lights sections state the integer walk.
         Its Transforms section says floats stay f64 until the quantization
- [x] **S2. Desktop harness and primary ids.** `voxrender-bench` renders
      headless through wgpu.
      1. `voxrender`'s golden and precision-sim scenes move behind a
         `fixtures` feature, which its tests and the bench share
      2. `GRID_RANGE_BITS` drops to 18, and the range check keeps each
         origin and its mirror within i32. The reference's grid rays and hit
         points hold i32, and its walk computes in i64
      3. A CLI runs one benchmark per subcommand and writes its report
      4. The integer DDA runs in WGSL in i32 alone over S1's quantized
         inputs and a dense cell buffer. It enters a grid as
         [entry32.py](reference/precision-sim/entry32.py) does. It writes a
         placement, cell, and face id per pixel
- [x] **S3. The CPU renderer gets its own crate.** A consumer injects the
      render tech through the repo's dependency pattern.
      1. `voxrender` keeps the contract types, the shared quantization, and
         the fixtures, and drops its `cpu` feature
      2. `voxrender` defines the `Render` capability trait over today's
         `render` inputs
      3. `voxrender-cpu` holds the reference renderer, the walk, and the
         goldens
      4. `voxsmith` renders through `D: Render`. Its `DependenciesImpl`
         implements `Render` with `voxrender-cpu` behind `impl`
      5. The contract and `voxrender`'s docs say where the reference lives
- [x] **S4. Shadow bits and B1 on this Mac.**
      1. The WGSL writes per-corner shadow bits. The entry's correction also
         finds a point light's end cell. A shadow's start in another
         placement's grid comes from f32
      2. B1's desktop half passes B1's line against the reference on the
         golden and precision-sim scenes. The report counts each scene's
         mismatches
      3. A float DDA on the same scenes records its flip count
- [x] **S5. Quest harness and B0.** `voxrender-bench-quest` runs as an
      OpenXR app on Quest 3 and reports what the device exposes.
      1. The repo README's Development section documents the Android setup:
         the SDK, NDK, and adb, the Rust Android target, the APK build tool,
         and the headset's developer mode
      2. The app opens an OpenXR session on Vulkan and builds wgpu from the
         session's instance and device. Swapchain images are wrapped as wgpu
         textures. One multiview pass clears both eyes
      3. B0's probe runs at startup and logs its report for adb to pull
      4. Timed runs pin the GPU at level 4, and VrApi confirms the level
      5. The report lands in benchmarks.md. B0's outcomes prune the later
         steps
- [x] **S6. B1 on Quest.** S2's and S4's kernels run in the headset. The
      run counts id mismatches and measures the int32 cost and the integer
      step against a float step. Decision 1 applies the 1.25x line.
      1. `voxrender-bench` becomes a library over a GPU device its caller
         opens. `voxrender-bench-desktop` takes the CLI and the headless
         device. `voxrender-wgpu` takes the walk. `voxrender` takes the ray
         and shadow-target setup that both renderers share
      2. The Quest app renders the reference on the headset at startup and
         logs B1's id and shadow counts
      3. Once the session runs, each pass times one kernel at the eye
         resolution: the integer and float walks at two depths, and a
         dependent chain of i32 or f32 adds and compares
      4. The report lands in benchmarks.md. Decision 1 applies the 1.25x
         line
- [ ] **S7. Bench scenes and the per-object structure.**
      1. `voxrender-bench` builds the research's five scenes from fixtures
         and generators. B1's ids run over them
      2. The 64-tree of research section 3.4 is built on the CPU from a
         dense grid: a dense root of 16^3 tiles, 4^3 inner nodes with u32
         pair masks and popcount-compacted children, uniform-material
         collapse, and 4^3 leaves with 0, 1, 2, 4, or 8-bit palettes. Tests:
         the tree gives back every cell of each scene's objects
      3. A WGSL walk over the tree runs through `HitTrace::with_walk`. It
         skips an empty node by an f32 guess of its exit cell, which wrapping
         multiply-adds and an exact sign check correct. B1's ids gain the
         tree walk, which must meet B1's line on every scene
- [ ] **S8. B2 on Quest.** The run measures ray rate and step cost with B2's
      setup. The rate decides which B3 arms get built.

## Later

The steps after S8 get their detail when S8 lands. They run in this order:

1. The stereo view in the reference: two off-axis frusta and a head pose
2. S6, the moving scene, joins the bench scenes. Its motion is a scripted
   function of time
3. B3, one step per arm the B2 rate allows, then the shootout
4. The reference rules B4 to B7 need, each before its benchmark: bloom at
   quarter resolution, the shadow memo's tier rules, `msaa4`, brick LOD, and
   transparency termination
5. B4 to B12
