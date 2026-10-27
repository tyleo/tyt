# voxrender-wgpu speed-up research

_Part of the [voxrender-wgpu plan](../README.md)._ A multi-agent web research
pass wrote this report on 2026-10-22 for checklist step S9.1. It ran 11 topic
researchers, an audit of our WGSL with scratch-copy timings on this Mac, a
skeptic per angle, a completeness critic and 4 gap researchers, a synthesizer,
and a final reviewer. Device, driver, and wgpu facts date from that day and
can change. Re-confirm them before building on them. It builds on the earlier
[research](research.md) and on B2's rates in [benchmarks](benchmarks.md).

## 1. Summary

B2 measured 0.037 Grays/s on S1 and 0.304 on S5. The bars are 0.25, 0.5, and
1.7.

1. **0.25 Grays/s.** S5 is already there. S1 can credibly get there. Two
   changes measured on this Mac speed S1 up 3.3x together: an f32
   hierarchical DDA in place of leave(), and a cull of placements behind the
   nearest hit. A CPU model of S1's own walk cuts its iterations from 18.2 to
   6.1 with NAADF-style empty boxes at the tile and child levels [3]. Stacked
   with cautious Quest factors, S1 lands near 0.2 to 0.6 Grays/s (derived,
   unmeasured on Quest)
2. **0.5 Grays/s.** S5 likely gets there, near 0.5 to 0.8 after the same
   changes. S1 gets there only at the top of the stack. S9 aims here because
   0.5 is the hybrid's floor
3. **1.7 Grays/s.** No credible evidence supports a full-frame per-pixel
   march on Quest. This pass found no public rays/s for any Adreno 740 voxel
   march.
   The best published Adreno ray-query rate, a single-hit heightfield
   through Qualcomm's own custom BVH and traversal kernel, scales to about
   0.42 to 0.49 Grays/s at Quest clocks [31][32] (derived). Reaching 1.7
   also needs two ray-count cuts: a lens-matched grid and stereo reuse. Tests
   weigh each cut's image cost against research.md's Priority 2 and 4. Even
   then only the top of the range gets there. Factions, the one shipped
   Quest voxel game with a public rendering talk, rasterizes [42]

The likeliest outcome is a march fast enough for the far field, glass, and
shadows in a raster-near hybrid. It takes three changes:

1. An f32 hierarchical walk
2. Empty boxes in spare tile tags, plus child boxes from the node masks the
   walk already reads
3. Register-resident nodes with per-leaf occupancy masks

The plan still lacks a raster timing of S1 and S5 on Quest to judge the
march against.

Two measurement problems come first. Every timed kernel carries naga's
runtime checks [23][25]. On this Mac, two runtime integer divisions per step
limit B2's step loop, the cost yardstick, more than its load does.
Re-baseline before comparing levers.

## 2. Ranked levers

Each gain is one lever's Quest time factor alone against today's code. "Mac"
marks a gain the audit measured on this Mac in scratch copies. "Model" marks
the CPU iteration model of S1, which counts 18.2 iterations per ray against
S1's measured 18.6, within 3%. Everything else is estimated. Rows rank by
expected gain times confidence.

### 2.1 Levers that speed each ray

| # | Lever | Expected gain on S1 / S5 | Evidence | Effort | Confidence |
| --- | --- | --- | --- | --- | --- |
| 1 | f32 hierarchical DDA: one exit per region from per-ray reciprocals, a clamp into the neighbor, power-of-two masks for `% size`. Replaces leave() and error_at | 2.5 to 2.7x / 1.5x. Mac 2.74x / 1.48x | Mac, with 52 and 94 flipped pixels on S1 and S5 (0.002 to 0.003%). ESVO's tx_coef [6], Tree64's clamp [2]. ir3 lowers fdiv to rcp and a 32-bit imul to 3 ops [10][11] | small | high on Mac, medium on Quest |
| 2 | Register-resident tile, node, and leaf, a 64-bit occupancy mask per leaf, and the palette read only at the hit | 1.5 to 2.5x / 1.1 to 1.5x | Tree64's ancestor memo 1.9x on an iGPU [2]. Mac light cache 1.0x, since its L1 hides the re-descent. Three fixed levels need about 7 registers and no stack | medium | medium |
| 3 | Register footprint at or under 12 vec4 for 16 waves | 1.0 to 1.8x / 1.0 to 1.3x | Mesa lists Quest 3 as FD740v3 with 96 vec4 per unit. Waves: 16 at 12 vec4, 12 at 16, 8 at 24, 6 at 32 [8][9] | small | medium, unknown until stats are read |
| 4 | Trusted shader modules: no loop counters, index clamps, or division guards | 1.05 to 1.3x | wgpu-hal and naga 30.0.1 source [23][24][25][26] | small | medium |
| 5 | Leaner node fetch: dense child slots or a stored prefix popcount, 16-byte node records read as vec4<u32> | 1.05 to 1.4x / 1.0 to 1.1x | B2's u32-pair mask step costs 1.53x a plain step. A 32-bit popcount is 7 ir3 ops [12]. NAADF uses dense blocks [3] | small | medium |
| 6 | Loop shape: while-while for in-leaf steps, subgroupAny to batch jumps | 1.1 to 1.5x / small | Divergent branches run both sides [14][15]. Aila and Laine [55]. Mostly folded into lever 1 | medium | low |
| 7 | Power-of-two root strides, no runtime imul in region_at | 1.03 to 1.15x | Qualcomm: mul24 is native, 32-bit multiply is not, integer divide is very expensive [16] | small | medium |
| 8 | 16x8 workgroups, if the driver runs 8x8 as a half-empty wave128 | 1.0x, or 2x in that case | Mesa forces wave64 for groups of 64 or fewer [70]. Devsh gives a subgroup size of 64 on Quest [42], as does gpuinfo [19] | small | low |
| 9 | 4 B output or fused shading in place of the 16 B vec4<u32> hit | 1.0 to 1.1x today. Frees 1.4 to 1.6 ms at the target rate | 94.6 MB per frame is 1.85 ms at 51.2 GB/s, derived from [61]'s 4x16-bit LPDDR5 at 3.2 GHz | small | high |
| 10 | Two rays per invocation, unrolled in-leaf steps | 0.8 to 1.5x | Guthe: unrolling alone 2 to 5% [57]. Costs registers | medium | low |

Lever 1 gives up bit exactness. Its exact alternative runs the integer DDA at
the region's level with per-level error terms: two sign tests per exit, no
divide, no correction loop. It needs F = 12 and DB = 14 so a 16-cell exit
fits in 30 bits (projects/render/voxrender-wgpu/src/grid_walk.wgsl:7,
projects/render/voxrender-wgpu/src/first_hit.wgsl:5). This pass derived
it. No paper publishes it, and nothing has tested it against the reference.

Low expected gain on Quest. A cheap one can still get a test:

1. f16 or i16 walk state: 1.0 to 1.25x, and only if it drops the register
   footprint past a wave step. The exact error terms must stay 32-bit. It
   needs passthrough SPIR-V (section 4 item 4)
2. Node fetches through textures: B2 has storage buffers 15 to 24% ahead.
   Adreno lays 3D textures out as stacked 2D slices [71]
3. Morton orders, group-ID swizzles [60], and wave-uniform fetches: they pay
   only when L2 misses dominate. Nothing shows that for coherent primaries
4. Workgroup memory: over 2 KB per 64-thread group cuts waves [9]
5. Compressing the hot path: the walk is not DRAM-bound at these rates
6. Persistent threads, ray refill, packets, warp-cooperative traversal,
   dynamic fetch: they tie or lose on coherent primaries [55][56][58][59]
7. A new tree shape: 18.6 iterations sits in the 16 to 31 published for
   64-trees [1]. Shallow wide levels beat SVDAGs and deep octrees [3][5][7]

### 2.2 Levers that cut rays or iterations

| # | Lever | Expected gain on S1 / S5 | Evidence | Effort | Confidence |
| --- | --- | --- | --- | --- | --- |
| 1 | NAADF-style empty boxes: a 6x4-bit box in an empty tile's slot under free tag 6 or 7, child boxes grown at run time from the node mask, 2^3 coalescing | Model: 18.2 to 9.75 iterations for tile boxes, 6.11 with child boxes, 4.44 with 2^3 in leaves. Time 2 to 3x / 1.0 to 1.2x | NAADF primaries 3.1 to 4.4x over no field on an RTX 3090 Ti [3][4]. Tree64 2^3 coalescing 21% [2]. The model's 8x8 wave maximum is 1.06 to 1.15x its mean | small to medium | high on iterations, medium on time |
| 2 | Cull placements by slab entry against the nearest hit so far, front to back | 1.2 to 1.3x / 1.03 to 1.1x. Mac 1.29x integer, 1.21x f32 on S1 | Mac. first_hit walks every placement to completion | small | high |
| 3 | Lens-matched ray grid, resampled into the planar eye buffer | 1.4 to 1.8x net on both | Rays 0.55x for an equi-angular grid, 0.27 to 0.36x with a falloff like Meta's FFR maps [47] (derived from the map images, which state no fractions in text; unverified). NVIDIA LMS [46], Valve [45]. Resample about 0.5 to 1.3 ms | small | medium. Image cost tested against Priority 4 |
| 4 | Stereo visibility reuse, or a conservative second-eye start from the first eye's depth | 1.3 to 1.8x | Wissmann 1.28 to 1.55x on full raster frames with ray-traced hole filling [50]. Wan's terrain caster 1.6 to 1.8x on the pair, derived from its 72 to 90% right-eye savings [51] | medium | medium. Image cost tested against Priority 2 |
| 5 | Cell boxes inside mixed leaves from the leaf occupancy mask | Model: 6.1 to 4.0 iterations. 1.2 to 1.5x / small | Model. NAADF's voxel layer [3] | medium | medium |
| 6 | Per-eye beam pre-pass | 1.15 to 1.4x before boxes, about 1.05x after / about 1.0x | ESVO: iterations fell 14 to 48% at 0.8 MP for 1.02 to 1.25x in time, and time gained 1.05 to 1.40x at 3.1 MP [6]. Claybook's pre-pass 0.2 ms [53] | small | medium |
| 7 | Quad views, a foveated inset | Unknown, possibly a loss (unverified) | Meta says the outer and inset recommended sizes are currently the same, so four full views add rays unless the app shrinks the outer pair, and needs Horizon OS v85 or higher [52]. B0 probed with a 1.0 instance and no manifest tag | small to probe | low |
| 8 | Sparse tracing with an analytic face-plane fill and a compacted re-trace | 1.6 to 4x, speculative | Valve's RDM 5 to 15% [45]. Drobot's compaction 0.77 to 0.57 ms [48]. DOOM's compute VRS 1 to 2 ms [49] | medium | low. Misses thin occluders |
| 9 | Iteration cap that reports a hit, or a grazing footprint stop | 1.0x / 1.1 to 1.2x at caps of 6 to 8 | Lumen caps its GI traces [66]. S5 model: cap 8 changes 7.5% of pixels | small | medium. Untested on primaries |
| 10 | Distance LOD | 1.0x / 1.0x on the timed views | A 2 cm voxel is 1 px at 17.6 m with f = 880 (derived). Matters for S2's far field | small | high that it is 1.0x here |

Low expected gain:

1. Temporal or cross-eye start depth without a conservative bound: discards
   ran 1.5 to 4.4% in a stereo path tracer [72], far over 0.1%. Klein's 2x
   for volumes [67] does not carry over
2. A shared stereo beam: it saves under 1% of rays and loosens near starts
3. Distance fields in separate arrays: a Chebyshev or directional field adds
   a load per iteration and loses to boxes in the same bits [1][68]
4. An extra aligned 64^3 level: model 18.2 to 16.2 because every 64^3 block
   of a furnished room holds something

### 2.3 The ray-query arm

Adreno 740's RT unit tests one 64-byte node per instruction. Traversal runs
as shader code, and every AABB candidate returns to the shader [28][29][30].
In Turnip an opaque triangle hit commits inside the driver loop. This pass
assumes Qualcomm's driver, the blob, does the same. wgpu 30 has no BLAS
refit, so every edit rebuilds a BLAS. On this evidence ray query suits
shadows and glass.

| # | Lever | Expected gain on S1 / S5 | Evidence | Effort | Confidence |
| --- | --- | --- | --- | --- | --- |
| 1 | Cheaper candidates: occupancy in the Brick record, no "before" lookup for rays that start in air, per-ray reciprocals, early commit | 1.5 to 3x | Audit of ray_query.wgsl:60-90. A ray query object costs registers [15] | small | medium |
| 2 | Opaque triangle BLAS of occupancy-only greedy quads, voxel id from the hit point | 1.5 to 3x. S1 to about 0.12 to 0.25, S5 near 0.3 | Turnip commits opaque triangles in its loop [30]. A triangle boundary ran 1.06 to 2.26x over clustered boxes on RTX [36] (unverified). Intel and Arm prefer triangles [38][39] | medium | medium |
| 3 | Tight AABBs around occupied cells, no bricks without an exposed face | 1.1 to 1.3x, more at 16^3 | S1's room 16^3 boxes shrink to 23% of their volume, 4^3 boxes to 51%. Tight boxes cut 20 to 31% of frame time in a path tracer [34] | small | medium |
| 4 | Flatten static placements into one triangle BLAS | 1.1 to 2x / small | Intel up to 2x [38]. Turnip loads a 3x4 matrix per instance hit [30] | small | medium |
| 5 | Per-voxel AABBs with a slab test and commit | 1.2 to 1.8x | 1.8 to 1.9x over a 7^3 brick march on an RTX 3090 [35] (unverified) | small | medium |
| 6 | Ray flags: FORCE_OPAQUE, CULL_BACK_FACING, TERMINATE_ON_FIRST_HIT for shadows | 1.0 to 1.1x on primaries, about 1.4x on shadows | Arm 42.6% on Mali shadows [39]. Today RAY_FLAG_NONE | small | medium |

Low expected gain:

1. Block walking: 2 to 9% on first-hit rays, and sometimes 42% slower [37]
2. Subspace culling: 97.8% of baseline time on a CPU [69]
3. 8^3 chunk AABBs: Teardown's successor picked them for edits, not speed
   [40]

### 2.4 How the levers stack

1. Lever 2.1.1 subsumes the `%` masks, reciprocals, error_at's multiplies,
   and most of the loop-shape gain, because one code path serves every
   region size. Do not count those again
2. Boxes remove empty-region iterations. Register residence makes in-leaf
   steps cheap. After boxes the model leaves 3.4 single-cell moves per ray on
   S1, so the two mostly multiply. Leaf occupancy masks serve both register
   residence and cell boxes
3. Boxes, the beam, and the cull all shorten the empty prefix. After boxes
   the beam is worth about 1.05x
4. Register residence adds registers that the footprint lever removes. Read
   the footprint before and after
5. The lens-matched grid, quad views, and sparse tracing remove the same
   peripheral rays. Test each and keep the best
6. Stereo reuse and the conservative second-eye start overlap. Test both
   and keep the best
7. Output and launch fixes add no rate today but free 1.4 to 1.6 ms once the
   march nears budget
8. The box model counts iterations over every placement, as today's walk
   does. The cull already drops the placements behind the hit, where boxes
   would have cut iterations too. Time also falls less than iterations [6].
   After the f32 walk and the cull, boxes take 1.8 to 2.7x. That sits under
   the model's 2.98x iteration ratio and under lever 2.2.1's 2 to 3x against
   today's code
9. 2^3 coalescing (lever 2.2.1) and cell boxes in leaves (lever 2.2.5) skip
   the same empty cells inside leaves: model 6.11 to 4.44 and 6.1 to 4.0.
   Count one

Estimated stack on Quest (derived, every factor unmeasured on Quest):

| Step | Factor S1 / S5 | S1 Grays/s | S5 Grays/s |
| --- | --- | ---: | ---: |
| B2 today | | 0.037 | 0.304 |
| f32 walk and cull | 2.5 to 3.3x / 1.4 to 1.5x | 0.09 to 0.12 | 0.43 to 0.46 |
| Tile and child boxes | 1.8 to 2.7x / 1.0 to 1.1x | 0.17 to 0.33 | 0.43 to 0.50 |
| Register-resident leaves | 1.2 to 1.5x / 1.1 to 1.3x | 0.20 to 0.49 | 0.47 to 0.65 |
| Trusted modules, wave occupancy | 1.05 to 1.2x | 0.21 to 0.59 | 0.49 to 0.78 |
| Lens-matched grid, effective | 1.4 to 1.8x | 0.29 to 1.07 | 0.69 to 1.4 |

Stereo reuse at another 1.3 to 1.8x puts the top of both ranges over 1.7
Grays/s effective, 1.9 on S1 and 2.5 on S5. That needs every factor at its
top end and both cuts holding up in their quality tests.

## 3. Our code

The audit's problems rank by expected gain. Timings come from scratch copies
on this Mac, both eyes at 1680x1760.

1. **The region exit path.** Every region larger than a cell goes through
   leave()
   (projects/render/bench/voxrender-bench/src/tree_walk.wgsl:77-146): an f32
   guess with up to 3 divides, then up to 3 attempts of 2 axes, each with an
   f32 divide and two wrapping-i32 correction loops. error_at then does 6
   i32 multiplies
   (projects/render/voxrender-wgpu/src/grid_walk.wgsl:210-218). The
   1-cell step takes the other branch
   (projects/render/bench/voxrender-bench/src/tree_walk.wgsl:198-233), so a wave
   of mixed sizes pays for both. Fix: the f32 hierarchical DDA of lever
   2.1.1. S1 11.28 to 4.12 ms, S5 1.27 to 0.86 ms
2. **Every placement walked to the end.** The trace loop runs enter() and the
   full walk for every placement on every pixel
   (projects/render/voxrender-wgpu/src/first_hit.wgsl:166-183). S1 has 8
   placements. Fix: a slab test for entry t, a skip past the nearest hit,
   placements front to back, and the nearest distance as the walk's end.
   S1 1.29x on the integer walk, 1.21x on the f32 walk
3. **A root re-descent and a palette load every iteration.** The loop calls
   region_at (projects/render/bench/voxrender-bench/src/tree_walk.wgsl:241),
   which reads the tile slot (:34), mask words (:46-51), child slot (:61),
   leaf index word (:72), and palette (:74). The palette feeds only
   `value != inside` (:243). Fix: a per-leaf occupancy mask, the current tile
   and node in registers, the palette read at the hit, and a value-walk
   fallback when `inside != 0`. A light cache measured 8.30 against 8.27 ms
   here because the Mac's L1 hides the re-descent. Quest has to judge this
   fix
4. **Runtime signed modulo.**
   projects/render/bench/voxrender-bench/src/tree_walk.wgsl:226 and
   projects/render/bench/voxrender-bench/src/float_march.wgsl:92-93 take
   `% size` with a runtime i32. naga always wraps signed `%` into SDiv, IMul,
   and ISub with guards, even in trusted modules [25][27]. ir3 lowers a
   runtime divide to about 35 to 45 ops [13]. The float march does it on
   every iteration. Fix: `& ~(size - 1)`. S1 1.06x
5. **Checked shader modules.** Every timed kernel uses
   create_shader_module
   (projects/render/voxrender-wgpu/src/hit_trace.rs:183,
   projects/render/bench/voxrender-bench/src/ray_query_trace.rs:272,
   projects/render/bench/voxrender-bench/src/step_kernel.rs:49,
   projects/render/bench/voxrender-bench/src/chain_kernel.rs:46). naga adds a
   vec2<u32> counter to each loop [25], a clamp to each dynamic vector index,
   and tracking to each ray query initialization. The walk nests up to 4
   loops. Fix: create_shader_module_trusted with
   ShaderRuntimeChecks::unchecked() [26]. Rerun B1's ids after the switch
   because Quest's compiler once crashed on a dynamic bool vector index
   (doc/plan/open/voxrender-wgpu/implementation-decisions.md:197-201)
6. **An ALU-bound yardstick.** The B2 step loop divides by two runtime
   uniforms every step
   (projects/render/bench/voxrender-bench/src/steps.wgsl:30). On this Mac: base
   83.0 Gsteps/s, start hoisted 190, incremental address 207, scattered
   addresses 12.2. B2's "12 steps per iteration" mixes ALU and load cost.
   Fix: add hoisted and two-chain variants to B2 and re-time Quest
7. **A 16-byte hit per pixel.**
   projects/render/voxrender-wgpu/src/first_hit.wgsl:27, :190, and :200
   and projects/render/bench/voxrender-bench/src/ray_query.wgsl:38 write a
   vec4<u32> with one word unused: 94.6 MB per frame, 1.85 ms at 51.2 GB/s.
   Fix: pack to 4 or 8 bytes, or shade in the trace and write 4 B of color
8. **Placement passed by value.**
   projects/render/bench/voxrender-bench/src/tree_walk.wgsl:152 copies the
   272-byte Placement
   (projects/render/voxrender-wgpu/src/grid_walk.wgsl:17-35) from a
   storage binding (projects/render/voxrender-wgpu/src/first_hit.wgsl:20).
   The walk keeps 9 change terms live
   (projects/render/bench/voxrender-bench/src/tree_walk.wgsl:201-204). Fix: a
   slim placement record in a uniform buffer under 7 KB [15], change terms
   recomputed from the direction
9. **Mask rank.** Two popcounts and a branch on bit 32
   (projects/render/bench/voxrender-bench/src/tree_walk.wgsl:46-60). Fix: a
   stored prefix count, or dense child slots
10. **Root index multiplies.**
    projects/render/bench/voxrender-bench/src/tree_walk.wgsl:32-34 multiplies
    runtime tile counts. Fix: pad the root to power-of-two tile counts. S1's
    room goes from 10x9x10 tiles to 16^3, 3.6 KB to 16 KB
11. **Ray-query candidate setup.** Each candidate makes two dependent loads
    (projects/render/bench/voxrender-bench/src/ray_query.wgsl:63), loads the
    whole Placement, runs at_pixel, enter(), and error_at, descends from the
    root for the start cell and again for the cell before it
    (projects/render/bench/voxrender-bench/src/tree_walk.wgsl:172-181), then
    re-descends on every step. Boxes cover whole bricks plus a 1/64-cell pad
    (projects/render/bench/voxrender-bench/src/ray_query_trace.rs:24, :103-118).
    The query uses RAY_FLAG_NONE
    (projects/render/bench/voxrender-bench/src/ray_query.wgsl:55). Fix: lever
    table 2.3. Dropping the "before" lookup for rays that start in air is
    reasoned from the code but untested
12. **Float march divides.** One divide per axis per iteration
    (projects/render/bench/voxrender-bench/src/float_march.wgsl:100) and a
    quaternion transform per candidate (:18-20). Fix: per-ray reciprocals and
    a per-placement inverse affine
13. **Unknown wave size.** The Quest bench does not request SUBGROUP
    (projects/render/bench/voxrender-bench-quest/src/xr_gpu.rs:157), and every
    kernel uses 8x8 groups
    (projects/render/voxrender-wgpu/src/first_hit.wgsl:153). Fix: request
    SUBGROUP and log subgroup_size
14. **The shadow path in the primary pipeline.**
    projects/render/voxrender-wgpu/src/first_hit.wgsl:123-151 and :185-223
    stay compiled in when unlit. Removing them measured 11.27 against
    11.28 ms here. Quest is unmeasured

## 4. What the earlier research missed or got wrong

1. **The 750 Mrays/s Adreno rate** (research.md:41-47) came from a
   Qualcomm-custom BVH and traversal kernel, on rays from the depth map's own
   camera with one hit each, timed with the BVH update [31]. No Vulkan or
   wgpu app can supply either. With Solar Bay's 1.435x from Adreno 740 to
   750 [32], it scales to about 420 to 490 Mrays/s at Quest clocks. The
   scaling assumes the phone Adreno 740 runs at 680 MHz (unverified). It is
   a ceiling no wgpu path reaches. The poster also calls procedural leaves
   suboptimal. Software BVH primaries on Adreno 730 stay at 61 to 323 [33]
2. **The step yardstick.** B2's 12 steps per iteration and its outcome 4 read
   the step loop as a dependent-load cost. On this Mac the loop is bound by
   two runtime divisions (section 3 item 6). Per step, this Mac spends 92 to
   123 lane-cycles and Quest 104 to 122, derived from B2's rates. The 10x
   gap comes from lane count times clock because Adreno trades clock for
   width [18]. Quest pays no latency penalty on the step. The tree walk
   costs about 1.5x more lane-cycles per iteration on Quest. Quest-specific
   costs exist, but they are smaller than the raw gap
3. **Cache latency.** research.md:264-266 converts ns at Quest's clock.
   On-GPU latency is fixed in cycles: Adreno 730's 62 ns L1 at 900 MHz is
   about 56 cycles, about 90 to 100 ns at Quest clocks. Only the SLC figure
   carries over in ns [17]
4. **f16.** README.md:106-107 and research.md:1027 say stock wgpu cannot
   reach f16 on Quest. The only missing bit is
   uniformAndStorageBuffer16BitAccess [19]. wgpu-hal requires it for both
   SHADER_F16 and SHADER_I16 [23]. The Quest bench owns its VkDevice
   (projects/render/bench/voxrender-bench-quest/src/xr_gpu.rs:151-232), so
   passthrough SPIR-V with shaderFloat16 and shaderInt16 enabled is
   spec-clean. Section 2.1 rates the gain small
5. **64-bit atomics.** research.md:1021 records int64 atomics false. Quest
   reports shaderBufferInt64Atomics true and only the shared bit false [19].
   wgpu-hal 30 requires both [23], so a small patch exposes buffer atomicMin.
   Splatting still makes a poor primary path. Dreams' 640 M splats/s on PS4
   [43] maps to 9 to 19 ms on Quest at 1 to 2 splats per pixel (derived).
   Compute raster loses to single-draw raster on Adreno 660 [44]
6. **Quad views.** B0 found PRIMARY_STEREO alone (benchmarks.md:26-27).
   STEREO_WITH_FOVEATED_INSET is OpenXR 1.1 core and needs a QUAD_VIEWS
   manifest tag [52][65]. The bench creates a 1.0 instance
   (projects/render/bench/voxrender-bench-quest/src/xr_system.rs:54) with no
   tag, so B0 could not have seen it
7. **Trusted modules.** research.md:1003 and :1290 plan them for shipping
   and for B2, but every B2 kernel ran checked. Every B2 rate carries the
   overhead
8. **The stack rejection.** research.md rejects a per-ray software stack
   walk. It does not apply: the 64-tree has three fixed levels, so register
   residence needs about 7 registers and no stack
9. **NAADF.** research.md cites its 7029 Mrays/s. It missed that our tree is
   NAADF's 16, 4, 1 nesting without the distance fields, and that tags 6
   and 7 are free to hold them
   (projects/render/bench/voxrender-bench/src/tree64.rs:14-27) [3]
10. **The beam.** ESVO's iteration cuts of 14 to 48% gave 1.02 to 1.25x in
    time at 0.8 MP and up to 1.40x at 3.1 MP [6]. Iterations do not turn
    into time one to one. B8's shared stereo beam (research.md:1415) saves
    under 1% of rays and loosens near starts
11. **Shipped Quest precedent.** Factions, a Quest 2 and 3 voxel game,
    draws its voxels through a GPU-driven raster pipeline [42]. Devsh puts
    the usual budget near 1 M triangles per frame at 72 FPS, under the
    1.3 M to 1.8 M that research.md cites from Meta. A full-screen 8 B in,
    4 B out pass costs almost 1 ms at native resolution, about 0.65 ms at
    ours (derived). The talk calls a visibility buffer impractical on
    Adreno. In a same-stack wgpu comparison a fitted mesh had the lowest
    mean frame time on every device measured [54]
12. **Minecraft with RTX** does trace primaries, in two 742x835 passes side
    by side for a 2560x1440 output, 17 to 34% of output pixels (derived)
    [41]. research.md could not confirm it
13. **Mesh shaders.** wgpu 30 has EXPERIMENTAL_MESH_SHADER. Quest's driver
    lists no VK_EXT_mesh_shader [19]. The driver blocks them, not wgpu
14. **Output path.** The plan's compute march writes through storage.
    Qualcomm prefers fragment shaders and says compute and mutable images
    almost certainly disable UBWC [15]. Mesa lets a7xx compress storage
    images [62]. HypeHype writes mobile lighting as pixel shaders for the
    same reason [63]. A fullscreen fragment pass into the SRGB swapchain
    avoids the question. A timing probe settles it

## 5. Experiment ladder

Each rung makes one A/B change and times it on this Mac through `b2-times`.
A change stays if it wins and B1's ids stay under 0.1% on every scene. Rungs
1 and 2 share the first worn Quest run, 3 and 4 the second, 5 and the
decision the third. Numbers are Quest p50 rates unless marked.

1. **Re-baseline.** Trusted modules on every kernel. Request SUBGROUP and log
   subgroup_size. Add B2 step variants: start hoisted, two independent
   chains, vec4 loads, a subgroup-uniform address, and STEPS = 0 launch-only
   kernels in compute and fragment. Log f from XrFovf at
   projects/render/bench/voxrender-bench-quest/src/probe_visibility_mask.rs:78-80.
   Probe quad views with a 1.1 instance and the manifest tag. Read tree_walk's
   register footprint and scratch in RenderDoc Meta Fork. Decide: if two
   chains run near 2x one chain, the walk is latency-bound and rung 4 runs
   before rung 3. If tree_walk spills or uses over 24 vec4, the trims of
   rung 5 move up to rung 2
2. **f32 walk and cull.** Port the audit's f32 walk and placement cull. The
   Mac expects 3.3x on S1 and 1.5x on S5. B1 must pass on S2, S3, and S4,
   where the earlier float march crossed 0.1%. Switch: if any scene flips
   over 0.1%, take the exact per-level exit with F = 12 and DB = 14. If Quest
   S1 gains under 2x while the Mac gained 3.3x, the walk is not ALU-bound
   there, so rung 4 goes next
3. **Boxes.** Tile boxes come first. The model cuts S1 from 18.2 to 9.75
   iterations, against 18.6 measured, with no new load. Then child boxes
   from the node mask and 2^3 coalescing. Log iterations through
   counted_tree_walk. Expect S1 1.6 to 1.9x for tile boxes and 1.8 to 2.7x
   for both after rung 2. Stop: if iterations fall as modeled but time falls
   under half that ratio, the box exit costs too much. Profile before adding
   cell boxes
4. **Register-resident leaves.** Leaf occupancy masks, the cached tile and
   node, the palette at the hit, and the glass fallback. This Mac will likely
   show about 1.0x, so Quest judges it. Expect S1 1.2 to 1.5x and S5 1.1 to
   1.3x. Stop: if S1 gains under 1.1x, or the footprint crosses a wave step
   and Quest loses, keep the masks and drop the cache
5. **Fetch and footprint trims.** Prefix popcount or dense slots, vec4 node
   records, a slim uniform placement, power-of-two root strides, cell boxes
   in leaves, and 16x8 groups if rung 1 saw wave128. Each is worth 1.03 to
   1.4x. Keep a trim only if it gains 3% or more on S1 or S5 at p50. Stop
   the rung when two trims in a row miss that
6. **Decision.**
   1. S1 at or above 0.25 and S5 at or above 0.5: the hybrid's march is
      viable. Go on to rung 7
   2. S1 under 0.15 after rungs 2 to 5: stop tuning the walk. The march
      serves glass, shadows, and the far field. B3 builds (c) and (d)
   3. S1 at or above 0.5 and S5 at or above 1.0: rung 7 tests whether a full
      march could reach 1.7 effective
   4. Anything between cases 1 and 2: one profiling pass (section 6) for
      the binding cost. If it names no lever worth 1.3x or more, treat it
      as case 2
7. **Ray-count cuts and caps.** Each runs as a test that measures its gain
   and its image cost. The tests cover the lens-matched grid with its
   resample timed alone, stereo visibility reuse, and iteration caps of 6,
   8, and 12 on primary rays. B1's ids count the pixels each test changes
   on every scene. The test's report shows where those pixels fall: thin
   features, disocclusions, grazing floors, or the periphery. A cut stays
   when the lead accepts its tradeoff. Stop: if the resample costs over
   1 ms, keep the planar grid. Conservative start bounds keep hits exact
   and run under B1's line instead:
   1. The second eye from the first, one eye after the other
   2. Each eye's halves crossed with the other eye's
   3. The previous frame, reprojected through a scripted head move
8. **The ray-query arm** runs on Quest only because wgpu ray query does not
   run on this Mac. Cheaper candidates, tight boxes, and flags first, then the
   opaque triangle BLAS. Stop: if the triangle BLAS stays under 0.3 Grays/s
   on S5, ray query serves shadows and glass only
9. **Output path**, once a frame nears 5 ms. 4 B output or fused shading,
   then a fullscreen fragment pass against compute, with a UBWC timing probe
   over constant, flat-block, and noise contents. Keep the faster. Stop: if
   the 4 B output saves under 0.3 ms per frame on Quest, the write is not
   bandwidth-bound, so skip the fragment probe

Alongside the ladder, a bare multiview greedy-mesh raster of S1 and S5 with
direct draws gives the number every rung is judged against. It is B3's arm
(d). Pulling a minimal timing forward costs little.

## 6. Profiling plan

On this Mac:

1. `b2-times` with each variant. First question: does the change win at all
2. An Xcode Metal capture of the wgpu pipelines with per-line shader cost.
   First question: which lines of leave() and region_at cost most
3. counted_tree_walk.wgsl. First question: how S1's 18.6 iterations split
   between single-cell moves and region skips, which several gain estimates
   depend on
4. The CPU iteration model of S1, if checked in. First question: each skip
   rule's iteration cut before any GPU work

On Quest:

1. RenderDoc Meta Fork shader statistics on tree_walk, the step loop, and the
   ray-query march [20]. First question: register footprint and scratch,
   hence waves in flight from the Mesa table, and whether anything spills
2. The subgroup_size probe. First question: wave64 or wave128 for 8x8 groups
3. The B2 step variants. First question: latency-bound, issue-bound, or
   ALU-bound
4. ovrgpuprofiler -r for % Texture L1 Miss, % Texture L2 Miss, and % Stalled
   on System Memory [21]. Its per-draw mode, -e then -t -x, for % Shader ALU
   Capacity Utilized, % Time ALUs Working, and % Time EFUs Working [22]. It
   adds about 10% overhead. First question: memory stalls or ALU, and whether
   L2 misses matter enough to try swizzles
5. The launch-only kernels. First question: the fixed cost of a 5.91 M
   invocation dispatch, compute against fragment
6. If RenderDoc cannot capture the compute dispatch, a raw VkPipeline from
   naga's SPIR-V on the app-owned device with
   VK_KHR_pipeline_executable_properties, which Quest lists [19]
7. Mesa's ir3 compiled offline for FD740v3, as a proxy for instruction
   counts only. Nobody has tried it here. Quest runs Qualcomm's compiler,
   not ir3

## 7. Open questions and unverified claims

1. Every Quest factor in sections 2 and 5 is unmeasured. The Mac factors come
   from scratch copies that are not in the repo. Rung 2 rebuilds them
2. The f32 walk was checked against the reference on S1 and S5 only
3. Whether Qualcomm's compiler lowers 32-bit imul, popcount, and integer
   divide as Mesa's ir3 does [11][12][13]. Qualcomm's OpenCL guide agrees on
   direction [16]. The A650 issue-rate data [14] has no a7xx counterpart
4. The tree walk's register footprint and the wave size the driver picks for
   8x8 groups
5. Whether u64 ops split into 32-bit pairs on Adreno 740. Keep u32 pairs
6. Hardware figures:
   1. Quest 3's per-eye focal length, near 790 or 880 px/rad
   2. DRAM bandwidth. Qualcomm says 4x16-bit LPDDR5 at up to 3.2 GHz [61],
      which derives to 51.2 GB/s. Devsh says LPDDR5X at up to 64 GB/s [42]
   3. L2 size
   4. GMEM: about 2 MB per Meta [64], 3 MB per other sources (unverified)
7. Whether a storage or mutable swapchain keeps UBWC on the blob [15][62]
8. Whether Quest 3 offers quad views, and at what view sizes
9. Whether the blob keeps early Z or LRZ under conservative frag_depth, which
   decides any fragment-march or proxy arm
10. Triangle ray-query rates on Adreno 740 through wgpu, whether the blob
    packs triangle leaves, and whether naga's skip flags are valid without
    RayTraversalPrimitiveCullingKHR
11. Whether the blob's traversal loop matches Turnip's
12. Whether splitting a long dispatch helps the compositor preempt on the
    blob
13. Not verified by the pass: Lacewell's RTG2 chapter 37 numbers, the claim
    that Nanite voxels march bricks by DDA, the Immortalis against Adreno 740
    ray tracing figure, Cohen and Sheffer's city-block claim, Horizon Zero
    Dawn's 1/16 cloud update, and the M4 Max clock used in lane-cycle math
14. The lead ruled on 2026-10-23:
    1. An inexact f32 walk may replace the exact one inside the 0.1% budget
    2. Peripheral undersampling, stereo reuse, and iteration caps get tested.
       Each stays if its quality and performance tradeoff holds up. Its test
       shows where it breaks down

## 8. Sources

1. dubiousconst282, VoxelRT README and shaders.
   https://github.com/dubiousconst282/VoxelRT
2. dubiousconst282, a guide to fast voxel ray tracing with sparse 64-trees,
   2024. https://dubiousconst282.github.io/2024/10/03/voxel-ray-tracing/
3. Ulschmid et al., NAADF, CGF 2026.
   https://www.cg.tuwien.ac.at/research/publications/2026/ulschmid-2026-naadf/ulschmid-2026-naadf-paper.pdf
4. NAADF source. https://github.com/cg-tuwien/NAADF
5. Aokana, arXiv 2505.02017. https://arxiv.org/html/2505.02017v1
6. Laine and Karras, Efficient Sparse Voxel Octrees, technical report 2010.
   https://research.nvidia.com/sites/default/files/pubs/2010-02_Efficient-Sparse-Voxel/laine2010tr1_paper.pdf
7. Arbore et al., Hybrid Voxel Formats, arXiv 2410.14128.
   https://arxiv.org/html/2410.14128v1
8. Mesa, freedreno_devices.py.
   https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/freedreno/common/freedreno_devices.py
9. Mesa, ir3.c.
   https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/freedreno/ir3/ir3.c
10. Mesa, ir3_compiler.c.
    https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/freedreno/ir3/ir3_compiler.c
11. Mesa, ir3_nir_imul.py.
    https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/freedreno/ir3/ir3_nir_imul.py
12. Mesa, ir3_compiler_nir.c.
    https://gitlab.freedesktop.org/mesa/mesa/-/raw/main/src/freedreno/ir3/ir3_compiler_nir.c
13. Mesa, nir_lower_idiv.c.
    https://gitlab.freedesktop.org/mesa/mesa/-/raw/main/src/compiler/nir/nir_lower_idiv.c
14. freedreno wiki, A6xx SP.
    https://gitlab.freedesktop.org/freedreno/freedreno/-/wikis/A6xx-SP
15. Qualcomm, Adreno GPU mobile best practices.
    https://docs.qualcomm.com/bundle/publicresource/80-78185-2/topics/mobile_best_practices.md
16. Qualcomm, Snapdragon OpenCL general programming and optimization,
    80-NB295-11 Rev. C.
    https://docs.qualcomm.com/bundle/publicresource/80-NB295-11_REV_C_Qualcomm_Snapdragon_Mobile_Platform_Opencl_General_Programming_and_Optimization.pdf
17. Chips and Cheese, Inside Snapdragon 8 Gen 1's iGPU.
    https://chipsandcheese.com/p/inside-snapdragon-8-gen-1s-igpu-adreno-gets-big
18. Chips and Cheese, The Snapdragon X Elite's Adreno iGPU.
    https://chipsandcheese.com/p/the-snapdragon-x-elites-adreno-igpu
19. Vulkan hardware database, Quest 3 report 48433.
    https://vulkan.gpuinfo.org/displayreport.php?id=48433
20. Meta, RenderDoc Meta Fork shader statistics.
    https://developers.meta.com/horizon/documentation/native/android/ts-renderdoc-shaderstats/
21. Meta, ovrgpuprofiler.
    https://developers.meta.com/horizon/documentation/native/android/ts-ovrgpuprofiler/
22. Meta, per-draw-call metrics.
    https://developers.meta.com/horizon/documentation/native/android/ts-draw-call-metrics/
23. wgpu-hal 30.0.1, vulkan/adapter.rs.
    https://github.com/gfx-rs/wgpu/blob/v30.0.1/wgpu-hal/src/vulkan/adapter.rs
24. wgpu-hal 30.0.1, vulkan/device.rs.
    https://github.com/gfx-rs/wgpu/blob/v30.0.1/wgpu-hal/src/vulkan/device.rs
25. naga 30.0.1, back/spv/block.rs.
    https://github.com/gfx-rs/wgpu/blob/v30.0.1/naga/src/back/spv/block.rs
26. wgpu-types 30.0.1, ShaderRuntimeChecks.
    https://docs.rs/wgpu-types/30.0.1/wgpu_types/struct.ShaderRuntimeChecks.html
27. wgpu issue 8191, signed remainder. https://github.com/gfx-rs/wgpu/issues/8191
28. Abbott, Ray Tracing for Adreno GPUs on Turnip, XDC 2024.
    https://indico.freedesktop.org/event/6/contributions/302/attachments/225/305/Ray%20Tracing%20for%20Adreno%20GPUs%20on%20Turnip%20(XDC%202024).pdf
29. freedreno wiki, a7xx ray tracing.
    https://gitlab.freedesktop.org/freedreno/freedreno/-/wikis/a7xx-ray-tracing
30. Mesa, tu_nir_lower_ray_query.cc.
    https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/freedreno/vulkan/tu_nir_lower_ray_query.cc
31. Biswas and Bourd, Linear Ray Tracing, HPG 2025 poster.
    https://highperformancegraphics.org/2025/publications/posters/Linear%20Ray%20Tracing%20-%20HPG2025.pdf
32. GSMArena, Snapdragon 8 Gen 3 benchmarks.
    https://www.gsmarena.com/we_benchmark_the_snapdragon_8_gen_3-news-60774.php
33. Frolov et al., Cross-RT, arXiv 2409.12617. https://arxiv.org/abs/2409.12617
34. Hansson-Soderlund and Akenine-Moller, tight bounding boxes for voxels and
    bricks, EG 2023.
    https://research.nvidia.com/publication/2023-05_tight-bounding-boxes-voxels-and-bricks-signed-distance-field-ray-tracer
35. Hansson-Soderlund, Evans, and Akenine-Moller, JCGT 2022.
    https://jcgt.org/published/0011/03/06/paper.pdf
36. Wald, Zellmann, and Morrical, empty space skipping, EGPGV 2021.
    https://www.sci.utah.edu/~wald/Publications/2021/spaceskip/ss.pdf
37. Derin, Harada, Takeda, and Iba, block walking, SIGGRAPH Asia 2021.
    https://gpuopen.com/download/SA2021_BlockWalk.pdf
38. Intel, real-time ray tracing in games.
    https://www.intel.com/content/www/us/en/developer/articles/guide/real-time-ray-tracing-in-games.html
39. Calvo Lista, Mobile Ray Tracing Demystified, Vulkanised 2026.
    https://vulkan.org/user/pages/09.events/vulkanised-2026/Mobile-Ray-Tracing-Demystified-Iago-CalvoLista-Arm.pptx.pdf
40. Rundlett and Gustafsson, ray tracing voxels in Teardown and beyond, GPC
    2025.
    https://static.graphicsprogrammingconference.com/public/2025/talks/raytracing-voxels-in-teardown-and-beyond/Rundlett-Gustafsson-raytracing-voxels-in-teardown-and-beyond.pdf
41. Galvan, Minecraft RTX frame analysis.
    https://alain.xyz/blog/frame-analysis-minecraftrtx
42. Silva, GPU-Driven Rendering in the Quest 2 and 3, Vulkanised 2026.
    https://vulkan.org/user/pages/09.events/vulkanised-2026/1415-Lucas-Silva-DevshGraphicsProgramming.pptx.pdf
43. Evans, Dreams, SIGGRAPH 2015 Advances.
    http://advances.realtimerendering.com/s2015/AlexEvans_SIGGRAPH-2015-sml.pdf
44. Tellusim, compute raster. https://tellusim.com/compute-raster/
45. Vlachos, Advanced VR Rendering Performance, GDC 2016.
    https://media.steampowered.com/apps/valve/2016/Alex_Vlachos_Advanced_VR_Rendering_Performance_GDC2016.pdf
46. NVIDIA, Lens Matched Shading and UE4 integration.
    https://developer.nvidia.com/lens-matched-shading-and-unreal-engine-4-integration-part-1
47. Meta, fixed foveated rendering.
    https://developers.meta.com/horizon/documentation/native/android/os-fixed-foveated-rendering/
48. Drobot, software VRS in Call of Duty, SIGGRAPH 2020.
    https://www.activision.com/cdn/research/Sig20_VRS_final_final_DROBOT.pptx
49. Microsoft, variable rate compute shaders in DOOM: The Dark Ages, 2026.
    https://developer.microsoft.com/en-us/games/articles/2026/04/variable-rate-compute-shaders-doom-the-dark-ages/
50. Wissmann et al., stereo rendering with reprojection and ray tracing, IEEE
    VR 2020.
    https://downloads.hci.informatik.uni-wuerzburg.de/2020-ieeevr-stereo-rendering-preprint.pdf
51. Wan, Zhang, Kaufman, and Qu, stereoscopic voxel terrain, IEEE VR 2000.
    https://cse.hkust.edu.hk/~huamin/vr_00.pdf
52. Meta, stereo with foveated inset.
    https://developers.meta.com/vr/documentation/native/android/os-stereo-with-foveated-inset/
53. Aaltonen, Claybook, GDC 2018.
    https://gpuopen.com/download/gdc_2018_tutorial_clay_sim_and_raytracing_in_claybook.pptx
54. Malyshau, Six Ways to Draw Vangers with WebGPU, arXiv 2608.17390.
    https://arxiv.org/abs/2608.17390
55. Aila and Laine, ray traversal efficiency on GPUs, HPG 2009.
    https://users.aalto.fi/~ailat1/publications/aila2009hpg_paper.pdf
56. Aila, Laine, and Karras, technical report 2012.
    https://users.aalto.fi/~laines9/publications/aila2012tr1_paper.pdf
57. Guthe, latency considerations of BVH traversal, EG 2014 short.
    https://diglib.eg.org/bitstream/handle/10.2312/egsh.20141013.053-056/053-056.pdf
58. Ylitie, Karras, and Laine, compressed wide BVHs, HPG 2017.
    https://users.aalto.fi/~laines9/publications/ylitie2017hpg_paper.pdf
59. Lier et al., warp-cooperative traversal, HPG 2018.
    https://diglib.eg.org/bitstream/handle/10.1145/3231578-3231583/07-1038-lier.pdf
60. Bavoil, thread-group ID swizzling, NVIDIA.
    https://developer.nvidia.com/blog/optimizing-compute-shaders-for-l2-locality-using-thread-group-id-swizzling/
61. Qualcomm, Snapdragon XR2 Gen 2 product brief.
    https://docs.qualcomm.com/bundle/publicresource/87-73689-1_REV_A_Snapdragon_XR2_Gen_2_Platform_Product_Brief.pdf
62. Mesa, tu_image.cc.
    https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/freedreno/vulkan/tu_image.cc
63. HypeHype, lighting notes, SIGGRAPH 2025 Advances.
    https://advances.realtimerendering.com/s2025/content/s2025_stb_lighting_v1.1_notes.pdf
64. Meta, advanced GPU pipelines.
    https://developers.meta.com/horizon/documentation/native/android/po-advanced-gpu-pipelines/
65. Khronos, OpenXR 1.1 specification.
    https://registry.khronos.org/OpenXR/specs/1.1/html/xrspec.html
66. Wright et al., Lumen, SIGGRAPH 2022 Advances.
    https://advances.realtimerendering.com/s2022/SIGGRAPH2022-Advances-Lumen-Wright%20et%20al.pdf
67. Klein et al., empty space skipping with frame coherence, IEEE
    Visualization 2005. https://hgpu.org/?p=1179
68. Deakin and Knackstedt, accelerated volume rendering with Chebyshev
    distance maps, CVM 2020.
    https://www.sciopen.com/article/10.1007/s41095-019-0155-y
69. Yoshimura and Harada, subspace culling, I3D 2023.
    https://gpuopen.com/download/publications/I3D2023_SubspaceCulling_updated.pdf
70. Mesa, ir3_nir.c.
    https://gitlab.freedesktop.org/mesa/mesa/-/raw/main/src/freedreno/ir3/ir3_nir.c
71. Mesa, fd6_layout.c.
    https://gitlab.freedesktop.org/mesa/mesa/-/raw/main/src/freedreno/fdl/fd6_layout.c
72. Stereoscopic path tracing with spatiotemporal reprojection, VISIGRAPP
    2019. https://www.scitepress.org/Papers/2019/76921/

## 9. Review notes

The final review on 2026-10-22 re-fetched the sources behind the Summary and
the top ten levers of 2.1 and 2.2, and checked every repo path:line against
the code. Every repo cite matched.

Changed:

1. Stack: boxes after the f32 walk and cull drop from 2 to 3x to 1.8 to
   2.7x, since the cull and the iteration model overlap and time falls less
   than iterations. S1 now lands at 0.2 to 0.6, 0.29 to 1.07 with the
   lens-matched grid. Added stacking notes 8 and 9. Renamed the
   "occupancy" row to wave occupancy, since leaf occupancy masks already sit
   in the register row
2. Summary: the 0.42 to 0.49 Grays/s ceiling is the best published Adreno
   ray-query rate scaled, not a hardware cap. "No public rays/s exists"
   became "this pass found none". Factions is the one shipped Quest voxel
   game with a public rendering talk, not provably the only one
3. Model 18.2 against measured 18.6 is within 3%, not 2%
4. ESVO beam: 14 to 48% fewer iterations at 0.8 MP gave 1.02 to 1.25x. The
   1.05 to 1.40x comes from 3.1 MP. The old row mixed the two
5. Quad views: Meta says outer and inset sizes are currently equal, so the
   "up to 2x fewer rays" had no support
6. Numbers that did not match their sources:
   1. Wan's stereo gain is 1.6 to 1.8x
   2. Devsh's triangle budget is about 1 M, not 0.5 to 1 M
   3. Tight boxes cut 20 to 31%, not 20 to 29%
   4. The fitted mesh won on every device measured, not every desktop GPU
   5. Minecraft RTX traces 17 to 34% of output pixels
7. 51.2 GB/s derives from the brief's 4x16-bit LPDDR5 at 3.2 GHz. The brief
   gives no GB/s. The signed `%` claim now cites naga's source [25]
   as well as the issue [27]. The lane-cycle claim is marked derived
8. Ladder: rung 3 uses the model's own baseline. Rungs 4, 5, and 9 gained
   stop numbers. Rung 6 gained a rule for results between its cases

Confirmed: [2] 1.9x and 21%, [1] 16.0 to 30.9 iterations, [3] 3.1 to 4.4x
and 7029, [6] tx_coef, [8][9][70] the wave table and wave64 rule, [11][12]
imul and popcount op counts, [15] UBWC, 7372-byte constants, ray query
GPRs, [16] mul24 and integer divide, [17] 62 ns at 900 MHz, [19] the 16-bit,
int64 atomic, and extension bits, [23][25][26] the wgpu and naga claims,
[28][30] the Turnip traversal, [31][32] the poster and Solar Bay ratio,
[39] 42.6%, [42], [45] 5 to 15%, [48] 0.77 to 0.57 ms, [49] 1 to 2 ms,
[50] 1.28 to 1.55x, [53] 0.2 ms, [57] 1.9 to 4.9%.

Still doubtful:

1. [35] and [36] could not be confirmed from the PDFs' text, so they carry
   "(unverified)". [14] and [29] were blocked to this review. Sources cited
   only outside the Summary and top ten were not re-fetched
2. The 680 MHz phone clock behind the 420 to 490 Mrays/s scaling
3. The FFR ray fractions, read off map images
4. The box factor of 1.8 to 2.7x is a judgment, not a measurement. Rung 3
   replaces it
