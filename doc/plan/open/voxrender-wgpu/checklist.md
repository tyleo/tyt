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
- [x] **S7. Bench scenes and the per-object structure.**
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
- [x] **S8. B2 on Quest.** The run measures ray rate and step cost with B2's
      setup. The rate decides which B3 arms get built.
      1. A fixed-N-step loop reads a storage buffer, a 3D texture, a 2D
         array, and u32-pair masks. Each pass times one layout at one N over
         both eyes. The report regresses ms against N per layout. The desktop
         runs the loops through `b2-times`
      2. The tree walk traces S1 and S5 at 1680x1760x2. Both eyes trace the
         scene's view. The report gives the ray rate and iterations per ray
      3. Hardware ray query traces the same scenes over a TLAS of
         placements. Each object's BLAS holds one AABB per non-empty brick,
         at 4^3 and at 16^3. A candidate marches its brick to the first hit.
         B1's ids check the hits against the reference
      4. One worn run times all three. The report lands in benchmarks.md
- [ ] **S9. A faster march on Quest.** B2's first rates come from
      first-draft code. S9 raises them as far as it can before R picks the
      B3 arms.
      - [x] **S9.1. Research.** Industry research finds the levers that cut
            each ray's cost or the rays a frame needs. The research ranks
            the levers into an ordered list of experiments. The report lands
            in `reference/`
      - [x] **S9.2. Re-baseline.** Profiles show where the time goes.
            - [x] B2's kernels move to trusted shader modules, and B2 also
                  times the tree walk with the checks
            - [x] The step loop gains variants that split load cost from ALU
                  cost
            - [x] Probes report the subgroup size, the quad views, and each
                  eye's focal length
            - [x] B2 splits the tree walk's iterations by the side of the
                  region each one leaves
            - [x] Xcode's Metal capture costs the tree walk's lines on the
                  desktop
            - [x] `ovrgpuprofiler` reads the counters of the tree walk, the
                  step loop, the ray query, and the launches on Quest
            - [x] A sweep at 0 to 8 steps shows the compute launch's floor.
                  The Quest app logs the GPU clock's mix for each window
            - [x] RenderDoc Meta Fork reads the register footprints of the
                  tree walk, the ray query, and the step loop on Quest
            - [x] The report lands in benchmarks.md
            - [x] Each finding becomes an experiment in S9.4
      - [x] **S9.3. A raster to beat.** A bare greedy-mesh raster of S1 and
            S5 on Quest gives the march a rate to beat
      - [x] **S9.4. Experiments.** The experiments follow the research's
            [ladder](reference/speedup-research.md#5-experiment-ladder),
            with a round of new approaches after rung 3. They rework the
            tree walk, the 64-tree, and the ray query march, one change at
            a time. Each change is timed on the desktop through `b2-times`.
            A change that wins stays. One short Quest run confirms a batch
            on Quest. B1's ids check every change against the reference
            - [x] Rung 2 runs as three changes in turn: power-of-two masks
                  for `at % size`, the f32 walk with per-ray reciprocals,
                  and the placement cull. `leave()`, `enter()`, and
                  `at % size` take 65% of the S1 walk on the desktop
                  - [x] All three win on the desktop, and B1 stays exact
                  - [x] A Quest run confirms them
            - [x] Rung 3, boxes. Two chains gained 1.5x on Quest. A
                  latency-bound loop would gain 2x, so boxes stay ahead of
                  rung 4
                  - [x] Tile boxes win on the desktop, and B1 stays exact
                  - [x] Child boxes lose on the desktop and drop
                  - [x] A Quest run confirms tile boxes
            - [x] New approaches. Each gets a prototype of about one
                  session. B1's ids check each prototype. Start bounds
                  wait for S9.5. The ray-tracing unit waits for rung 8
                  - [x] Raster, then march. A fragment pass rasterizes the
                        box around each tile's live cells and marches each
                        pixel's ray inside the box. The depth test keeps
                        the nearest hit
                        - [x] The ray query's marches run in the boxes'
                              fragments, and B1 stays exact
                        - [x] A mask march steps through a bit per cell
                              from the rasterizer's point
                        - [x] A depth probe shows that Quest rejects early
                              only fragments that write no depth
                        - [x] Face depth lets the depth test reject a
                              fragment behind a nearer hit
                        - [x] Solid boxes draw as plain faces with no
                              march
                        - [x] A Quest run times the arms against the tree
                              walk and S9.3's raster. S1 takes 1.4 ms and
                              S5 3.2
                  - [x] A headset view. The Quest app draws a scene around
                        the wearer each frame, from each eye's pose, with
                        plain shading. S1 and S5 hold 90 Hz
                  - [x] The box raster's open ends, in order:
                        - [x] The frame's CPU time. A view sorts its boxes
                              when the head has moved 25 cm. A frame
                              between sorts takes S2 0.24 ms
                        - [x] A box that holds an eye. A view from inside a
                              bush showed farther leaves until back faces
                              drew a part in 1024 nearer
                        - [x] The trims of rungs 4 and 5 on the mask
                              march. Five trims take 21% off S1, 20% off
                              S5, and 16% off S2. Stray cells as boxes
                              lose. The occupancy stays at 62% to 66%
                        - [x] Boxes that cross another placement's box.
                              A front face's fragment steps through the
                              boxes its box crosses. Every box keeps its
                              face's depth. S2 runs in 4.7 ms
                        - [x] A pass a run. A pass of one draw runs in a
                              faster mode on Quest, so a draw of 1024
                              boxes or more gives each run its own pass
                        - [x] Stray cells over a dense layer come off as
                              boxes. A large draw draws three faces a box.
                              S5 runs in 2.1 ms
                        - [ ] B3's shading load in the box raster's
                              fragments, on S1 and S5
                              - [x] The view shades as the contract's
                                    opaque surface and matches the
                                    reference's image
                              - [x] The sun's corner shadows take a bit a
                                    lattice point
                              - [x] Quest's counters put S5's fragment at
                                    650 instructions and S1's at 413.
                                    Under 1% of the time stalls on memory
                              - [x] A face reads its corners' occlusion
                                    and light from a record of its cell.
                                    S1 takes 2.41 ms and S5 4.10
                              - [x] A sun whose direction holds a small
                                    whole ratio, as S4's does. Each corner
                                    casts its ray there
                              - [x] The Quest app logs the driver's
                                    statistics for the view's pipelines:
                                    instructions, reads, and registers
                              - [x] A cell's entry leads to its material
                                    in one read, a vertex hands its
                                    fragments the eye's ray, and a tile's
                                    cells run by y first. S1 takes 2.12
                                    ms, S5 3.76, and S2 7.1
                              - [x] The bloom add, from a texture that
                                    nothing fills yet. S1 takes 2.24 ms,
                                    S5 3.89, and S2 7.31
                              - [x] A fragment reads three words of its
                                    tile where it read six. A word for
                                    each row of 16 cells holds their bits
                                    and the count that places their
                                    records. S5 takes 3.55 ms against the
                                    3.5 ms line, S1 2.16, and S2 6.70.
                                    B6's LOD remains for S5's floor past
                                    8 m
                              - [x] S1 shades 1.21 times a pixel and S2
                                    1.47. A depth pass of the solid boxes
                                    ahead of the march leaves 1.03 and
                                    1.18. It costs S5 0.48 ms and gains
                                    S2 0.21, so it drops. Bands of
                                    distance and one run nearest first
                                    lose too
                              - [x] A shadow fill on the GPU. A compute
                                    pass casts each corner's ray through
                                    the placements that can shadow its
                                    tile. S2 draws its shadows in 7.1 ms
                              - [x] The fill as casters move. A mover
                                    changes its tiles and the tiles it
                                    shadows before and after. On Quest a
                                    cell fills in 14 to 68 ns, and one
                                    median mover's tiles in 1.8 to 3.6 ms
                                    against B5's 1.0 ms line
                        - [x] A crossing box that holds the eye. Its back
                              faces draw first at the nearest depth. Its
                              fragment steps from the eye through the
                              boxes it crosses. A view from inside a bush
                              takes 5.6 to 6.4 ms where it took 8.4
                        - [x] The crossing test as placements move. It
                              takes this Mac 0.03 ms over S1's boxes and
                              13.7 ms over S2's. A mover holds 30 of the
                              26,325. S6's view runs it each frame
                        - [x] Why S1 shades a tenth more fragments over
                              three faces a box than over corners. Under
                              the shaded view it does not. A pass a run
                              now suits every draw: a model of 900 boxes
                              ran a third slower in one pass, and S1
                              takes 2.04 ms from 2.17
                  - [x] A gallery of models. The Quest app shows the
                        prompt trials' 35 Voxel Json models one at a time
                        under the view's shading. The controllers step
                        through them, walk, turn, and resize. Each model
                        holds 90 Hz on its table
                  - [x] Tiles of pixels walk together. An 8x8 tile walks
                        the empty space its rays share. Each ray then walks
                        on alone. The prototype runs only if the box raster
                        leaves a compute walk in the frame. It leaves
                        none, so the prototype does not run
            - [x] A profile of the tree walk after rung 3. Tile boxes
                  halved S1's iterations, yet the walk gained 1.11x on
                  Quest against 1.30x on the desktop. The profile runs
                  only if a compute walk stays in the frame. None stays,
                  so the counters, the registers, and the wave test do
                  not run
            - [x] Rung 4, register-resident leaves. It does not run: rung 6
                  stops the walk's tuning
            - [x] Rung 5, fetch and footprint trims. It does not run
            - [x] Rung 6, the decision. The tree walk traces S1 at 0.096
                  Grays/s after rung 3, under the ladder's 0.15, and the
                  box raster shades it at 2.90. The walk's tuning stops.
                  The box raster is the front end S9 hands on
            - [x] Rung 8, the ray-query arm. Each object's greedy quads
                  as triangles trace S1 at 0.83 Grays/s on Quest, 5.3
                  times the best march, and S5 at 0.40. The box raster
                  keeps the primary rays. The arm is at hand for second
                  rays. The march's scratch stays unread
            - [x] Rung 9, the output path. The box raster's march runs in
                  fragment passes. A compute launch costs Quest 28 ns a
                  workgroup: a pass over both eyes takes 2.55 ms at 8x8
                  groups and 0.64 at 16x16, and a fragment pass 0.25
      - [x] **S9.5. Start bounds.** Conservative start bounds join the
            experiments. A bound tells a march where it can start without
            skipping a hit, so the hits stay exact. Three bounds were to
            run: the second eye from the first, crossed halves, and the
            previous frame reprojected. The box raster starts a ray on its
            box's face, so a bound can only spare a march that misses. The
            best case ran: a prior frame's depth with the head still. Its
            read costs more than the marches it spares on every scene.
            Start bounds drop
      - [ ] **S9.6. Lossy cuts.** Lossy cuts run as tests: the lens-matched
            grid, quad views, stereo hit copying, and iteration caps. Each
            test reports its gain and how many pixels it changes on every
            scene. It also writes three images per scene: the full trace,
            the cut, and a map of the changed pixels. A cut stays when the
            project lead accepts its tradeoff from the images
            - [x] The cut that fits the box raster is fewer pixels. The
                  Quest app draws at a scale of the eyes' size. At 0.8 the
                  scenes take 71% to 79% of their time
            - [x] A frame's time splits into 0.3 ms, about 55 ns a box,
                  and 0.29 to 0.84 ns a pixel. Three faces over 7 corners
                  take S5 to 3.40 ms, under B3's line
            - [ ] The project lead tries the scale in the headset and
                  keeps it or drops it
            - [ ] Quad views, four views of 840 by 880. By the split they
                  gain S1 and S4 a quarter and cost S5. The app draws
                  them once B6's LOD holds a scene's boxes down
      - [x] **S9.7. The final run.** A final Quest run re-times B2 with the
            improved code. The report lands in benchmarks.md. R picks the B3
            arms: the box raster shades S1's both eyes in 2.03 ms and S5's
            in 3.40, under B3's 3.5 ms line

## Later

The steps after S9 get their detail as they come up. On 2026-11-12 the
project lead put the trial models' look in the headset first. The steps run
in this order:

1. Reflections that follow the contract's materials
      - [x] A rough face mirrors less. The reference showed a face's mirror
            ray by one minus its GGX alpha, which left a brass at roughness
            0.5 three quarters of a mirror. What the ray meets now shows by
            the share of the face's lobe that lands within two cells of it,
            in the reference and in the box view. The box view's ray ends
            where that share runs out
      - [x] A face casts no ray that would end within two cells. Every
            trial model then draws in its old time or less on Quest
      - [ ] Shadows and occlusion at what a mirror ray meets. The box view
            shades it flat, and a quad within two cells takes its face's
            occlusion
      - [ ] Placements that move, a model's load time, and a line in the
            frame budget
2. Emission
      - [x] The box view adds the contract's bloom, which B4 was to time.
            It draws its glowing boxes at half size before a frame's
            passes, blurs them through levels that stand in for the
            contract's octaves, and adds the halo before the tonemap. A
            model that glows takes about 0.5 ms more on Quest, against a
            line of 0.6
      - [ ] B4's depth and motion vectors, and a level for each of the
            two narrowest octaves
3. Glass
      - [x] The box view draws the contract's transmission, which B7 was
            to time. The cells that pass light leave the opaque tiles. A
            pass after the opaque ones walks each box of surfaces between
            such materials, farthest first. A dual-source blend tints what
            lies behind. The trial models' pixels off the reference
            fall from 51,760 to 37,006, and the fish tank takes 5.6 ms on
            Quest
      - [x] A glass box draws behind its tile's opaque box, which closed
            holes in a tank's tint
      - [x] The sun through glass. The shadow fill carries the light
            through each surface a ray crosses. A cell keeps the tint that
            reaches it. The trial models' pixels off fall to 31,329
            for 0.03 ms on Quest
      - [x] A glass fragment shades its surfaces in fewer instructions.
            The fish tank takes 4.9 ms on Quest, down from 5.7
      - [x] The sun on a glass surface. The shadow fill lights each
            glass cell's surfaces too. A surface scales the sun's term by
            its cell's light. The trial models' pixels off fall to
            30,151, and the fish tank takes 5.0 ms on Quest
      - [ ] The walk's cost, a mirror ray through glass and from it, an
            eye inside a material, and the order of the boxes between
            sorts
4. A pass over the bench's code and tests that drops what the box raster
   left without a use
5. The stereo view in the reference: two off-axis frusta and a head pose
6. S6, the moving scene, joins the bench scenes. Its motion is a scripted
   function of time. The box view then moves a placement each frame: its
   matrices, the crossers of its boxes, and its lit words
7. B3, one step per arm the B2 rate allows, then the shootout
8. The reference rules B5 and B6 need, each before its benchmark: the
   shadow memo's tier rules, `msaa4`, and brick LOD
9. B5 and B6. A step starts with a study where its line in the frame
   budget is in doubt. B5 is one. A fill of the tiles one mover changes
   takes 1.8 to 3.6 ms on Quest against a line of 1.0 ms
10. B8 to B12
