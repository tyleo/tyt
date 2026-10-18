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
- [ ] **S2. Desktop harness and B1 on this Mac.** `voxrender-bench` renders
      headless through wgpu.
      1. A CLI runs one benchmark per subcommand and writes its report
      2. `GRID_RANGE_BITS` drops to 18, and the range check keeps each
         origin and its mirror within i32
      3. The integer DDA runs in WGSL in i32 alone over S1's quantized
         inputs and a dense cell buffer. It enters a grid as
         [entry32.py](reference/precision-sim/entry32.py) does. It writes a
         placement, cell, and face id per pixel, plus the per-corner shadow
         bits
      4. B1's desktop half passes with zero id mismatches against the
         reference on the golden and precision-sim scenes
      5. A float DDA on the same scenes records its flip count
- [ ] **S3. Quest harness and B0.** `voxrender-bench` runs as an OpenXR app on
      Quest 3 and reports what the device exposes.
      1. The repo README's Development section documents the Android setup:
         the SDK, NDK, and adb, the Rust Android target, the APK build tool,
         and the headset's developer mode
      2. The app opens an OpenXR session on Vulkan and builds wgpu from the
         session's instance and device. Swapchain images are wrapped as wgpu
         textures. One multiview pass clears both eyes
      3. B0's probe runs at startup and logs its report for adb to pull
      4. The report lands in benchmarks.md. B0's outcomes prune the later
         steps
- [ ] **S4. B1 on Quest.** S2's kernels run in the headset. The run counts
      id mismatches and measures the int32 cost and the integer step against
      a float step. Decision 1 applies the 1.25x line.
- [ ] **S5. Bench scenes and the per-object structure.**
      1. `voxrender-bench` builds the research's five scenes from fixtures
         and generators
      2. The 64-tree of research section 3.4 is built on the CPU from a
         dense grid
      3. Tests: a walk over the tree finds the reference's first hits on
         every scene
- [ ] **S6. B2 on Quest.** The run measures ray rate and step cost with B2's
      setup. The rate decides which B3 arms get built.

## Later

The steps after S6 get their detail when S6 lands. They run in this order:

1. The stereo view in the reference: two off-axis frusta and a head pose
2. B3, one step per arm the B2 rate allows, then the shootout
3. The reference rules B4 to B7 need, each before its benchmark: bloom at
   quarter resolution, the shadow memo's tier rules, `msaa4`, brick LOD, and
   transparency termination
4. B4 to B12
