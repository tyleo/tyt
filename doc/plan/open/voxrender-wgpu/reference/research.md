# voxrender-wgpu research

_Part of the [voxrender-wgpu plan](../README.md)._ A multi-agent web research
pass wrote this report on 2026-10-04. It ran ten topic researchers, a skeptic
who re-checked each topic's load-bearing claims, a completeness critic, and a
round on the critic's gaps. Device, driver, and wgpu facts date from that day
and can change. Re-confirm them before building on them.

## Priority

Since 2026-10-04 the VR experience comes first in every finding below. When
two goals conflict, the higher one in this list wins:

1. Hold the frame rate. Every frame lands at 90 Hz because a dropped frame
   judders and can make the wearer sick
2. Keep the image steady under head motion: no shimmer, crawl, pops, smear, or
   disagreement between the eyes
3. Keep the wearer comfortable: fades over teleports and snap turns, and AppSW
   only as an emergency
4. Keep the image sharp, at the default eye-buffer resolution or above
5. Keep the lighting faithful, with per-corner shadows ahead of bloom

A frame that runs over gives things up from the bottom of the list. Bloom goes
first, then per-corner shadows fall to per-face, then the render scale drops.
MSAA and the frame rate stay. Golden exactness never costs Quest frame time.
Where exactness would cost frame time, the tier takes a mirrored rule or the
test loosens. PCVR follows the same list at its refresh rate. Phones and flat
desktop come after the headsets. Where their needs conflict with Quest's,
Quest wins.

## 1. Verdict

Most of the proposal's direction holds. Visibility cost should scale with
pixels, LOD should follow footprint, antialiasing should not depend on history,
shadows should come from an object-space cache shared by both eyes, and every
frame should emit depth and motion vectors. Its central premise does not hold
on current evidence: a full-density primary march on Quest 3 is unlikely to
fit. Three independent estimates agree, though none measures our walk on the
device:

1. **Ray rates.** The frame budget in 3.1 leaves about 3.5 ms for primary
   visibility and shading. At the default eye buffer that is 5.91 M rays in 3.5
   ms, about 1.7 Grays/s [1]. The best Adreno rate on record is about 750
   Mrays/s for a single-hit heightfield through hardware ray query on Adreno
   750, about 480 to 535 Mrays/s scaled to Quest clocks [2]. Software BVH
   primaries on Adreno 730 run 61 to 323 Mrays/s [3]. The shortfall is 3x or
   more.
2. **ALU.** At 545 MHz, 3.5 ms gives about 500 lane-cycles per pixel, room for
   3 to 5 hierarchical steps after shading (derived). Primary rays through
   64-trees take 16 to 31 iterations [4].
3. **A shipped compute marcher.** Claybook traced primaries and AO at 720p in
   1.5 ms, plus a 0.2 ms cone pre-pass, on a base Xbox One [5]. Scaled by
   pixels to Quest's default stereo frame, that is about 11 ms (derived).

The likely Quest path is therefore a raster and march hybrid: greedy-mesh
raster with 4x MSAA where voxels are large on screen, and a march only where
they are small, where materials are transparent, and for shadows. The benchmark
decides. Raster is a co-equal candidate, not a fallback. Every shipped mobile
or VR voxel title rasterizes [6][7][8]. The full march remains the desktop
plan.

Changes the evidence calls for:

1. **Quest primary visibility is a benchmark result, not a premise.** B3 tests
   five front ends over one per-object structure, with the hybrid leading:
   - a near-field raster plus far-field fragment march in one multiview MSAA
     render pass
   - greedy-mesh raster with 4x MSAA everywhere, with brick LOD meshes far away
   - a fragment-shader march over rasterized front-face proxies
   - a compute march with tile binning
   - hardware ray query over a placement TLAS
2. **Traversal is exact integer arithmetic everywhere.** Even perfectly
   rounded, never-fused f32 disagrees with f64 on 2.3 to 8.0% of corner shadow
   samples under a 45-degree light [9]. WGSL cannot forbid contraction or
   reassociation, and our backends run with the loosest float defaults
   [10][11]. The reference and every tier run one integer DDA on inputs
   quantized once by shared Rust code. This changes the contract.
3. **One frame budget sets every threshold.** Post must be fused into the
   primary pass. A conventional bloom and tonemap chain costs 2 to 4 ms on
   Quest-class hardware [12][13], which would leave at most 2 ms for primaries.
4. **The per-object structure is a shallow wide tree that indexes material
   boundaries.** Use 4^3 nodes with 64-bit masks (u32 pairs in WGSL) and nodes
   that collapse uniform-material regions, optionally with NAADF-style
   empty-box distances [14][15]. Do not use occupancy mips, which cannot skip a
   glass interior, or DAGs, which save little at 2^27 cells and trace slower
   [16][17][15].
5. **The top level for primaries is not a per-ray software tree walk.** Use the
   rasterizer, screen-tile placement lists [18][19], or a hardware TLAS [20].
   Keep a TLAS for incoherent shadow rays and a light-space 2D grid for
   directional shadows [19].
6. **The step cap becomes a counted diagnostic.** Traversal is bounded by
   construction. Any limit that changes a pixel is a mirrored contract rule.
7. **LOD is chosen per brick from the head center, with OR occupancy and a
   deterministic distance-weighted cross-fade band.** Popping without history
   should be assumed visible in a headset [21][22][23].
8. **Antialiasing gets MSAA-4 semantics in the contract, with a far-field
   variant.** At coarse LOD the samples the center face misses share one
   sub-ray. Without it, edge work approaches 4x exactly where LOD was meant to
   cap cost.
9. **The shadow memo stores visibility, keyed and invalidated exactly.** An
   invalidated entry is a miss, never a stale value. Light changes commit
   double-buffered. The compute arm fills per face inline. The raster and
   fragment arms fill per brick before the render pass. Teleports and snap
   turns hide cold fills behind a fade.
10. **Transparency runs one compute ordered walk on every arm**, bounded by the
    opaque depth.
11. **Foveation on Quest 3 is fixed only.** The headset has no eye tracking
    [24]. AppSW is the last rung of an adaptive ladder [25][26].
12. **Phones are their own tier with a named floor device.** Floor phones
    rasterize.
13. **The contract gains rules, and the reference implements them before any
    GPU tier is judged:**
    - the integer traversal, its quantization, a dyadic corner inset, and a
      boundary-ownership rule in place of the bias
    - a stereo view with four-angle frusta and a head pose
    - the AA rule with its far-field variant
    - the LOD rule with occupancy, mid-walk level changes, the cross-fade band,
      and the level shadow rays use
    - a far-field rule and a transparency termination rule
    - bloom at quarter resolution with a named pyramid kernel, and an angular
      radius for VR
    - which hit supplies depth and motion vectors under glass
    - a shadow rule for continuously moving lights on VR tiers

## 2. What real systems do

| System | Primary visibility | LOD | Lighting | AA | Platform | Cost scales with |
|-|-|-|-|-|-|-|
| Teardown (2020-22) [27][28] | Raster one 36-vertex OBB per object, fragment DDA in a 1 B/voxel 3D texture with 2 occupancy mips, gl_FragDepth with hand-rolled early-Z | None (mips only skip) | Stochastic shadows and AO through a 1-bit world volume, temporal denoise | Jittered TAA, screen-door glass | Desktop (GTX 1060 min) | Covered pixels x box overdraw x steps |
| Claybook (2018) [5] | Compute SDF march of one 1024x1024x512 world volume, 8x8 cone pre-pass; moving bodies meshed and rasterized | Per-step mip by cone footprint | Traced AO and shadows | Not stated | Xbox One, PS4, PC | Pixels x steps; 8 MB of 512 MB touched per frame |
| Teardown successor (unreleased) [29][30] | HW RT, intersection shaders over 8^3 chunk AABBs | Not stated | HW RT, DLSS Ray Reconstruction | DLSS RR | Desktop | Rays x BVH depth |
| Dwyer devlogs (2022-25, unshipped) [31][32][33] | Raster 8^3 surface bricks plus fragment march (2022); compute 64-tree march (2024) | Footprint | DDGI on a fixed ray budget | FXAA (MSAA failed on marched edges) | Desktop, iGPU, WebGPU | Pixels x steps |
| Dreams (PS4, PSVR 2020) [34][35] | Shipped: compute point splatting. Unshipped prototype: rasterized 8^3 bricks with in-brick march | Stochastic cluster LOD | Stochastic | Heavy TAA | Console, PSVR | Splats near one per pixel |
| UE 5.7+ Nanite Voxels [36][37] | 4^3 bricks with 64-bit masks traced inside the Nanite raster pass | Voxels replace far triangle clusters at about 1 px error | Lumen | TSR (stochastic normal pick) | Console, desktop; Nanite lists VR stereo as unsupported [38] | Pixels covered by far clusters |
| Aokana (I3D 2025) [18] | Compute march of chunked shallow SVDAGs, 8x8 tile-chunk pairs, two-pass Hi-Z, 64-bit atomic visibility buffer | 256^3 chunks, 2-of-8 occupancy (not conservative) | Not the focus | Not stated | RTX 3060 Ti | Tile-chunk pairs x steps |
| NAADF (EG 2026) [15] | Compute march of 3 nested 4^3 levels with empty-box distances | None | ReSTIR GI | 32-frame TAA | RTX 3090 Ti | Pixels x iterations; 7029 primary Mrays/s at 2160p |
| VoxelRT Tree64 (2024-25) [14][4] | Compute march, 64-bit mask trees | None | Benchmark only | None | Unnamed iGPU | Pixels x 16-31 iterations |
| SVDAG (2013) [16] | Primaries rasterized; DAG traced for shadows and AO | Footprint | DAG shadows 240 Mrays/s | n/a | GTX 680 | Shadow rays x DAG depth |
| Lumen (UE5) [19] | Raster (Nanite); software traces for GI and reflections | Detail traces, then SDF clipmaps | Surface cache, fixed texel budget, about 16-frame refresh | TSR | Console, desktop | Fixed budgets |
| Minecraft with RTX (2020) [39] | Path-traced lighting over chunk geometry on RTX GPUs; whether primaries are traced was not confirmed here | Chunk distance | Path tracing, DLSS | DLSS | Desktop RTX | Rays |
| Minecraft Vibrant Visuals [6] | Raster chunk meshes | Chunk distance | Deferred PBR, shadows snapped to the pixel grid | TAAU or bilinear | Adreno 640+, iOS, consoles | Visible faces |
| QuestCraft, Vivecraft [7][8] | Raster, whole level once per eye | 4-6 chunks | Vanilla smooth lighting | 80% resolution | Quest 2, 3 | Faces x eyes; per-scene tuning |
| Veloren, Roblox [40][41] | Raster greedy quads | Chunk LOD | Per-face light atlas; 4^3 voxel light grid | MSAA or none | wgpu desktop; mobile, Quest | Faces |
| HypeHype (2023-25) [42][43] | Raster, pixel shaders preferred on tilers | n/a | Stochastic tile lighting | Not stated | Phones from Adreno 500 / Mali Bifrost up | Pixels, fixed light samples |
| Enshrouded GI [44] | Raster | n/a | Probe cascades on a 100k-500k ray budget | TAA | Desktop, Steam Deck | Fixed ray budget |
| gvox_engine, Bevy Solari [45][46] | HW RT over brick AABBs; wgpu ray query over triangles | n/a | ReSTIR, DLSS RR | FSR2, TAA, DLSS | Desktop | Rays plus denoise |
| Schutz splatting (HPG 2022) [47] | Compute 64-bit atomicMin splats | Batch LOD | None | Dilation | RTX 3090, Valve Index | Points after culling |

Three things stand out:

1. No shipped non-RT marcher draws many independently transformed placements
   from a compute top level. Claybook marched compute rays through one world
   volume and rasterized its moving bodies [5]. Teardown, the one shipped
   marcher over many transformed objects, lets the rasterizer handle the top
   level [27]. Dreams' brick prototype and Dwyer's 2022 renderer did the same,
   but neither shipped.
2. Everything that looks good relies on stochastic sampling plus temporal
   accumulation. Our contract allows neither, so on lighting the nearest
   shipped precedents are Minecraft smooth lighting, Veloren's face atlas, and
   Roblox's light grid.
3. The only shipped VR voxel renderers rasterize, and the one compute marcher
   with published per-pass timings ran at 720p on a console [5].

## 3. Design by component

### 3.1 Frame budget

**Inputs.**

1. The frame is 11.1 ms at 90 Hz. [B0](benchmarks.md#b0-quest-3-2026-10-18)
   measured the compositor at up to 0.69 ms of GPU time at p99. The
   compositor time and 13% headroom leave about 9.1 ms of app GPU time.
2. The default eye buffer is 1680x1760 per eye, 5.91 Mpx for both eyes [1].
   Panel resolution is 9.11 Mpx.
3. Thresholds are set at GPU level 4, 545 MHz. Levels 0 to 6 run at 285, 350,
   456, 492, 545, 599 and 640 MHz. Passthrough caps the GPU at 456 MHz
   [49][50].
4. Peak memory bandwidth is 51.2 GB/s per Qualcomm's brief [51]. Wikipedia's 68
   GB/s conflicts [52]. One 4 B/px stereo transfer at default resolution costs
   0.46 ms at peak (derived).

**Line items, default resolution, 90 Hz, level 4, fused post.**

| Item | ms | Set by |
|-|-|-|
| Engine reserve and submission | 0.5 | B10 |
| Quarter-res pre-pass: bloom source, AppSW depth and motion vectors | 0.3 | B4 |
| Bloom pyramid at quarter resolution and below | 0.3 | B4 |
| Primary visibility, fixed shading, bloom add and tonemap in-shader, one output write | 3.5 | B3 |
| AA extra rays or samples | 1.0 | B6 |
| Shadow memo: dedup, fill, steady-state misses | 1.0 | B5 |
| Transparency surcharge | 1.0 | B7 |
| Contingency (S3 may spend it on transparency) | 1.5 | B10 |
| Total | 9.1 | B10 |

**Rules.**

1. Fused post is a requirement, not an optimization. A conventional composite
   plus bloom chain costs 2 to 4 ms at default resolution (3.10). With it,
   primaries get about 0.1 to 2 ms.
2. At 120 Hz the app budget is about 6.6 ms. Every line scales by about 0.7, or
   the render scale drops.
3. At panel resolution every transfer costs 1.54x.
4. Under passthrough, fixed work takes 1.20x longer at 456 MHz. An MR mode runs
   at about 0.84x the pixels to restore the budget (derived).
5. Each benchmark is judged against its own line. B10 checks the sum.

### 3.2 Primary visibility

**Choice.** Select the Quest front end by benchmark (B3). Candidates, in order
of current expectation:

1. **Hybrid in one multiview MSAA render pass.** Near bricks are greedy quads,
   far bricks are marched in the fragment shader over proxy shells. The handoff
   is specified below.
2. **Greedy-mesh raster everywhere with 4x MSAA**, using brick LOD meshes past
   the near field.
3. **Fragment march over front-face proxies** (placement OBBs or brick-cluster
   shells) in one multiview pass, with `@early_depth_test(greater_equal)` or
   `less_equal` matching the depth compare. A separate path handles proxies
   that contain the near plane.
4. **Compute march with tile binning**, split into screen-tile dispatches well
   under 1.5 ms each.
5. **Hardware ray query** over a placement TLAS with one AABB per occupied
   brick.

On desktop, run the compute march, with hardware ray query as the top level
where available.

**Hybrid handoff.**

1. The CPU classifies each brick of each placement from the head center. A
   brick is raster when one of its voxels projects to at least N px, with N = 4
   or 8. The size is that placement's voxel size times its scale, so a
   fine-voxel placement near the eye can march while a coarse one behind it
   rasterizes.
2. One multiview render pass at 4x MSAA uses transient MSAA attachments
   resolved in tile. Raster quads draw first, then marched bricks as front-face
   proxy shells.
3. A proxy fragment marches its brick cluster from the proxy entry. It writes
   `frag_depth` under an early depth test matching the compare direction, and
   `sample_mask` from its own coverage (3.7). The depth test then merges per
   sample. A marched surface in front of a raster one wins its samples. A
   raster surface in front of a proxy rejects the proxy's fragments early, when
   the driver keeps LRZ.
4. One `frag_depth` per pixel serves every covered sample. It differs from
   per-sample depth only where two surfaces cross inside one pixel. B6 counts
   those pixels.
5. No compute pass reads the MSAA attachments. wgpu has no depth resolve or
   input attachments [53]. A march after the pass would store 4x depth and
   color, 16 B/px each, about 3.7 ms at default resolution (derived). One pass
   avoids this.
6. Transparent placements go to the ordered walk (3.9).
7. Classification is CPU-side, so it does not depend on indirect draws. If B0
   finds indirect draws work, GPU culling is an optimization. If not, the pass
   uses direct draws over precomputed index ranges per brick cluster and level,
   and B3 measures the CPU draw cost.
8. Classification uses no hysteresis. It changes no pixel at LOD 0, so it needs
   no contract rule.

**Evidence.**

1. **Rate and ALU.** See section 1. The ALU figure uses 1536 FP32 lanes [50]
   (single source). Memory latency may bind first. L1 texture hits take 62 ns
   on Adreno 730, about 34 cycles at 545 MHz, and the 4 MB system cache about
   243 ns, about 132 cycles [54]. Qualcomm says variable step counts increase
   wave divergence and cache miss rates [2], and advises against 3D textures
   and random access [55].
2. **Tilers favor fragment work.** HypeHype's 2025 rule is that tilers "must
   use pixel shaders (instead of compute shaders)", while noting compute "can
   further improve performance, depending on the hardware architecture" [43].
   FDM reduces only fragment invocations, so compute never gets it [56][57].
   Turnip keeps LRZ when the declared depth layout matches the compare. It
   drops LRZ writes for shaders that discard and drops LRZ entirely for shaders
   with side effects [58]. Quest 3 reports SHADER_EARLY_DEPTH_TEST [20]. The
   proprietary driver's LRZ behavior is unverified.
3. **Counterpoints for compute on Adreno 740, all from Mesa.** Adreno 7xx keeps
   compression on storage images [59]. Passes with few draws run direct to
   memory, so a full-screen raster pass gains little from tiling [60][12].
4. **Compute output.** Rgba8UnormSrgb has no STORAGE_BINDING on Quest, but
   VIEW_FORMATS is true [20]. A compute pass can write a Rgba8Unorm view and
   encode sRGB itself, provided the OpenXR swapchain accepts STORAGE and a
   mutable format. XR_META_vulkan_swapchain_create_info passes the usage
   through and errors if unsupported [61]. Otherwise a copy costs about 0.9 ms
   at default resolution (derived).
5. **Preemption.** Turnip bounds non-preemptible time by tile duration and
   shrinks tiles to stay under about 1.5 ms. Long untiled work risks the
   compositor's deadline [62][60]. Whether Quest's driver behaves the same is
   an inference. A compute march must therefore be split into short dispatches.
6. **Hybrid bound.** A voxel at least N px wide lies within d = s*f/N of the
   eye, with f about 1432 px/rad at 25 ppd. That is 3.6 m for 2 cm voxels at N
   = 8. The first visible layer holds at most pixels/N^2 faces, about 92K for
   both eyes at N = 8. Depth complexity inside the shell adds to that. Meta's
   budget is 1.3M to 1.8M triangles [63], and on-tile MSAA runs "close to the
   same speed as non-MSAA" [64]. Nanite Voxels is the shipped analog, with
   triangles near and voxels far [37].
7. **Hardware ray query.** Quest 3 exposes VK_KHR_ray_query and acceleration
   structures but no RT pipelines [65]. wgpu-info reports
   EXPERIMENTAL_RAY_QUERY [20], and wgpu 30 adds AABB BLAS geometry [66].
   Adreno 740 tests one 8-wide node per instruction, with the traversal loop in
   the shader [67]. Without intersection shaders every AABB candidate returns
   to the shader. Wald found AABB plus software leaf slower than a pure
   software BVH for volumes [68].

**Rejected.**

1. A full-density compute march as the assumed Quest path.
2. Compute splatting and Nanite-style software raster. Both need 64-bit
   atomics, which wgpu cannot enable on Quest [69][70], and both break exact
   faces or the transparency walk.
3. Per-voxel billboards, whose cost scales per voxel [71].
4. Back-face proxies with conservative depth. The marched depth moves toward
   the camera, so no matching layout exists [28][58].
5. A march pass after an MSAA raster pass that reads the stored attachments
   (cost above).
6. Wave-ballot ray refill. Claybook found a coarse cone pre-pass "simpler and
   does the job better" [5].

**Contract impact.** Every front end renders the same LOD 0 surface under the
integer traversal (3.3), so the near/far handoff needs no rule. Edge pixels
need the AA rule. The hybrid's per-pixel `frag_depth` is an approximation that
B6 counts.

### 3.3 Exact traversal and precision

**Choice.**

1. **One integer DDA in the reference and every tier.** It generalizes Cohen's
   integer 3D line traversal [72]:
   - the origin O is fixed point with F fraction bits
   - the direction D is integers, mirrored per octant so every component is
     non-negative
   - the axis choice is the sign of E_ab = (N_a - O_a) * D_b - (N_b - O_b) *
     D_a, where N is the next boundary on each axis
   - a step along a adds 2^F * D_b to E_ab, and a step along b subtracts 2^F *
     D_a
2. **The loop fits i32.** Between steps abs(E_ab) < 2^F * max(D_a, D_b),
   whatever the ray length, so i32 holds it while F + DB <= 30, where DB is the
   direction bit count. F = 13 and DB = 16 gives a direction resolution of
   about 0.0009 degrees, about 46x finer than a Quest pixel. The prototype
   peaked at 2^29 [9]. A coarse level L adds L bits.
3. **The step costs** 2 integer compares and 2 integer adds, close to
   Amanatides and Woo's float step [73].
4. **Exact ties go to the lowest axis**, x then y then z, matching the
   reference's `min_by` [74].
5. **Grid entry from far away runs in i32 too.** Exact entry needs about R +
   F + DB bits, and WGSL has no 64-bit integer [10]. An f32 guess picks the
   entry face e and the cell c on each other axis a. The remainder O_a * D_e -
   O_e * D_a - c * 2^F * D_e is exact in wrapping i32 within two cells of the
   ray, and its sign corrects the guess. A near-tie that picks a face too
   early leaves the ray behind the grid on another axis, and the entry moves
   to that axis. The guess holds up to 32 ulp of error per float op [9]. An
   origin and its mirror must fit i32, so a ray starts within 2^18 cells of a
   grid less the grid's size: 2.6 km at 1 cm voxels. This runs once per ray
   per placement. Mip jumps update E with wrapping multiply-adds, which are
   exact modulo 2^32.
6. **Shared Rust code quantizes every input to a discrete decision once on the
   CPU**, and the reference uses the same code:
   - primary directions per eye and placement as D(i,j) = (G0 + i*Gx + j*Gy) >>
     S, with S guard bits keeping the error under 1 LSB across 2064 pixels
   - per-corner shadow origins as lattice points plus a dyadic inset such as
     2^-10, replacing the 1e-3 inset, which has no exact binary value [76]
   - a boundary-ownership rule, where a point on a boundary belongs to the cell
     the ray moves into, replacing the 1e-4 bias
   - point-light directions as L - O in integers
7. **Other discrete decisions use integer compares too:** LOD switch distances
   by cross-multiplication, the slab entry axis at grid edges, and the merge
   order of coplanar hits between placements (the earlier placement wins).
8. **Shading stays float.** f32 GPU shading against f64 reference shading must
   fall within the per-channel tolerance.
9. **Per-pixel shadows** need one exact integer division per pixel to place the
   hit point. That is acceptable on desktop only.

**Evidence.**

1. **No backend promises bit-identical f32.**
   - Vulkan bounds division at 2.5 ULP and leaves rounding direction to the
     implementation unless the shader declares a mode. Operations may contract
     unless decorated NoContraction [77][78].
   - WGSL may fuse, may reassociate with no opt-out, and may flush subnormals
     [10].
   - Metal defaults to relaxed math on Apple silicon and fast math on Intel and
     AMD Macs [79][80].
   - D3D allows division as x*(1/y) [81].
2. **Drivers lower division to a reciprocal multiply.** RADV and freedreno set
   lower_fdiv. Apple's frcp is sometimes 1 ULP off. NVIDIA's div.approx is
   within 2 ULP [82][83][84][85]. Freedreno has no fused FMA path [86].
   Qualcomm's own compiler is closed.
3. **Our stack runs the loosest defaults.** naga emits no NoContraction,
   RoundingModeRTE or DenormPreserve, and wgpu-hal never sets Metal's mathMode
   [11]. Integer ops are exact modulo 2^32 on every backend [78][10].
4. **Quest 3's driver** reports denormal preserve false, flush-to-zero true,
   RTE true, int64 true, float64 false, and neither float_controls2 nor
   shader_fma [69].
5. **Simulation**, with f32 emulated as perfectly rounded and never fused,
   which is better than any spec allows [9]:
   - Under 45-degree lights, f32 and f64 disagree on 54 to 184 of 2,304 corner
     shadow samples per scene, 2.3 to 8.0%. sin 45 and cos 45 round to the same
     f32 value but differ by 1 ULP in f64. The built-in key light at azimuth
     -30 showed no disagreement.
   - Primary rays landed on a different cell for 8 and 22 of 16,384 pixels in
     axis-aligned perspective views, and 0 in oblique and orthographic views.
   - True division and reciprocal-times-multiply gave identical counts, so ULP
     bounds are not the cause. Input rounding is.
6. **f64 is not ground truth either.** Against exact rational traversal of its
   own inputs, the f64 walk takes a different step order on 18% of 45-degree
   corner rays, from accumulated `t_max += t_delta` and the non-dyadic inset
   [9]. The contract must name an exact traversal rather than ask tiers to
   match f64.
7. **Prior art.** Woop: double precision "only reduce[s] the probability" of
   edge failures [87]. Ize's conservative traversal is watertight but not
   deterministic [88]. Shewchuk argues for exact predicates [89]. ESVO places
   the grid in coordinates from 1 to 2 so float bits give cell indices [90].
8. **Integer cost on Adreno is uncertain.** INT32 add throughput through Vulkan
   on Adreno X1 was "poor" for reasons the author could not explain, and INT64
   is mediocre to very poor [91]. Single source. B1 measures it.

**Rejected.**

1. An f32 DDA with a mirrored tie order, the draft plan. B1 brings it back on
   Quest only if the integer step proves too slow there.
2. f64 on the GPU. Quest has no FP64 [69].
3. An f32 fast path with an exact check near ties. WGSL's reassociation makes
   any error bound non-rigorous, and the check needs the integer ray anyway.

**Contract impact.** The traversal becomes the integer DDA, with F, DB and S
stated, the quantization rules, the 2^-10 inset, boundary ownership, and the
tie order. The reference changes first and the goldens regenerate. "Floats stay
f64 until a GPU upload boundary" becomes "until the quantization boundary that
the reference and every tier share" [92].

### 3.4 Per-object acceleration structure

**Choice.**

1. A dense root over the object's bounds in 4^3 or 16^3 tiles. Inner nodes are
   4^3 with a 64-bit mask as two u32, a 32-bit pointer to popcount-compacted
   children, a 16-bit LOD material, and flags, in 12 to 16 B per node.
2. Leaves are 4^3 with a leaf-local palette of 0, 1, 2, 4 or 8 bits per cell,
   deduplicated by hash across objects.
3. Nodes collapse uniform-material regions, empty included. Masks index
   material-boundary cells.
4. Optionally, NAADF's per-cell empty-box distances, benchmarked against masks
   alone.
5. Bricks stay contiguous in read-only storage buffers by default. 3D textures
   are used only if B2 shows them faster. Objects shard across bindings, since
   Quest caps a storage binding at 128 MiB and 3D textures at 2048 per axis
   [20].

**Evidence.**

1. On one unnamed iGPU, Tree64 reaches 93 to 209 Mrays/s at 16 to 31 iterations
   per ray, ESVO 57 to 137 at 33 to 71, and flat DDA 24 to 33 at 354 to 512
   [4]. Integrated GPUs alone span about 2x on identical code [3], so these
   rates are only relative.
2. NAADF reaches 7029 primary Mrays/s, 7x SVDAG, at about 2x SVDAG memory.
   About a third of the gain comes from shallow nesting alone [15].
3. At 512^3, nested raw 8^3 levels are fastest [93].
4. At 2K^3 a DAG is often no smaller than a pointerless SVO [16]. The 6.1x
   material cost of 4-bit ids in DAG leaves comes from voxelized textured
   meshes, near a worst case [94].
5. Layout matters on mobile. Claybook's sparse 8^3 virtual-texture version was
   13% slower from indirection [5]. Aaltonen calls 3D tiling layout "a big
   deal" for volume rendering on phones [42]. Qualcomm advises against 3D
   textures [55].
6. Tree64's ancestor stack lives in group-shared memory, limited to 32 KB per
   workgroup on Quest [69]. NAADF's stackless layout is safer.

**Rejected.** DAG families (SVDAG, SSVDAG, TSVDAG, HashDAG) cost 10 to 40%
traversal for 13 to 35% memory [95][17]. Occupancy mips cannot skip glass.
Dense 1 B per cell storage costs 128 MiB per 2^27-cell object. ESVO contours
violate flat faces. SDF bricks are approximate surfaces [96]. Lossy color
blocks break exact materials.

**Contract impact.** None beyond 3.3. The integer walk descends nodes with the
same error terms.

### 3.5 Top-level structure over placements

**Choice.**

1. Primaries on the raster and fragment arms: the rasterizer, via quads and
   proxies. On the compute arm: per-eye screen-tile lists of culled placement
   or brick-cluster OBBs, sorted by entry depth, with early-out once the
   nearest hit is closer than the next entry. Add two-pass Hi-Z, where the
   previous frame only accelerates culling and a re-test keeps the output
   exact.
2. Large sparse placements split into occupied brick-cluster leaves, the voxel
   form of re-braiding.
3. Corner and face shadow rays use a TLAS, CPU binned SAH or compute LBVH,
   rebuilt every frame. Directional lights get a light-space 2D grid of object
   lists traced unordered, with a zero-throughput early-out.
4. Desktop uses a hardware TLAS with instance transforms.

**Evidence.**

1. Overlap is the remaining cost. Plain two-level is 1.18 to 2.1x slower than
   re-braided and up to 2.9x slower than a single BVH, with Rungholt, a voxel
   city, worst [97]. These are CPU, 8-bounce, incoherent-ray numbers.
2. Lumen dropped a BVH for per-cell object lists to get "a very simple and
   coherent tracing kernel", and uses light-space grids for directional shadows
   [19].
3. Claybook's 8x8 cone pre-pass took 0.2 ms at 720p and 4K alike [5].
4. Builds are cheap. H-PLOC builds a TLAS over 1M instances in 2.21 ms on an RX
   7900 XT [98]. wgpu builds a hardware TLAS from CPU instance lists at about
   50 ns per instance [99].
5. Shadow throughput is a product of passes, so order does not matter [92].
   Under the integer walk, float rounding of the product is the only order
   effect.

**Rejected.** A per-ray software stack walk for primaries on Quest. Aokana's
64-bit atomic visibility buffer, unavailable through wgpu on Quest [69][70];
merge in registers per tile instead. A merged world shadow volume, which
quantizes rotated and scaled placements [27][29].

**Contract impact.** Culling changes no pixels. The reference needs culling too
before it can render giant-scene goldens, since it slab-tests every placement
per ray today [74]. The coplanar tie order between placements is written into
the contract.

### 3.6 LOD and filtering

**Choice.**

1. **Level per brick** from the distance between the brick bounds and the head
   center, shared by both eyes, compared in integers against quantized
   thresholds. It accounts for lens distortion and the local foveated sample
   spacing.
2. **Bias** so coarse cells project to about 1.4 px. A 1 px square rotated 45
   degrees can miss every pixel center.
3. **Neighboring bricks differ by at most one level**, a 2:1-balanced level
   field like clipmap nesting [100].
4. **Coarse occupancy is OR**: a coarse cell is live when any child is live.
   Its material is the per-direction mode of exposed child faces, ties to the
   lowest palette id. For corner occlusion it counts as occupied only when that
   material's pass is zero.
5. **Mid-walk level changes.** A ray entering a brick at another level re-reads
   the inside material from that brick's cell at that brick's level. It
   registers a hit at the entered face only if that material differs from the
   one it was inside, under the existing surface rule.
6. **A deterministic cross-fade band** around each switch distance. Bricks
   inside the band are walked at both levels, and the pixel blends the two
   results by a weight computed from the hit's distance to the head center,
   never from time or noise. The blend covers shading, corner occlusion and
   shadows. Clipmaps use a band of n/10 grid units, at least 2 [100][101].
7. **Shadow rays walk at the receiver face's level** for their whole length, so
   an entry depends only on its key. Near receivers then walk far casters at
   full detail, and B5 measures that cost.
8. **Corner occlusion** is computed on each level's own grid. The band hides
   the shading seam this causes at a level border.
9. **Roughness rises** at coarse levels by the normal mix of the children.
10. **B6 also tests a filtered variant**: averaged attributes, with
    per-direction coverage used as alpha.

**Evidence.**

1. **LOD matters only far away.** At 25 ppd a pixel is 0.000698 rad. A 1 cm
   voxel reaches 1 px at about 14 m, and level 1 engages at about 20 m under
   the 1.4 px bias (derived). The default eye buffer is less dense than the
   panel, so these distances are shorter in practice.
2. **Seams, not holes, are the problem.** A conservative solid hierarchy opens
   no holes; Nanite's author gives "Voxels don't have cracks" as the reason
   [102]. But OR cells grow the surface outward, with "very noticeable
   artifacts near the silhouettes" once voxels exceed a pixel [103]. Distant
   Horizons documents coarse blocks sticking out around fine ones as intended
   behavior [104]. An OR cell at level L stands up to 2^L - 1 fine cells proud,
   so under 2:1 balance a border ledge is at most about one coarse cell, 1.4 to
   2.8 px (derived). Aokana's 2-of-8 rule is not conservative and can drop thin
   walls (inference) [18].
3. **Every system checked hides level boundaries with an extra mechanism:** a
   spatial blend band [100][105], continuous mip filtering [106], a seam-aware
   post filter [103], dither fades paired with TAA [107][108], or sub-pixel
   error plus TAA [102]. None relies on a bare hard switch.
4. **Popping should be assumed visible in VR.** Hard switches are "very
   noticeable" on desktop [21]. Nanite hides sub-pixel switches only because
   TAA blends them, and still sees pops on "a bulk change" next to a neighbor
   [102]. Clipmap borders "become apparent" with too small a band [100]. In VR
   "your head never stops moving" [22], flicker sensitivity peaks in the
   mid-periphery [23], and foveated rendering needed temporal antialiasing to
   keep the periphery stable [109]. No HMD study of LOD popping was found. The
   search budget ran out first, so this is unsearched rather than absent.
5. **Per-brick selection has its own failure.** Rays agree inside a brick, but
   the whole brick flips at once, which is Nanite's worst case [102]. The band
   addresses it. Per-ray selection instead slides the boundary as the head
   moves, and per-eye selection can pick different levels for the same point in
   the two eyes (inference).
6. **Bayer dither is deterministic**, so the contract does not forbid it.
   Without TAA it shows a screen door. Shipped engines pair dither with TAA
   [108]. That it looks bad in a headset is an inference.
7. **Shading seams.** CDLOD reports pops from vertex-based lighting on level
   swaps [105], and ESVO reports "seams... visible at hierarchy level changes"
   without a neighbor-clamped filter [103].
8. **Averaging and footprint costs.** Footprint LOD visibly changes the image:
   Aokana's SSIM is 0.80 to 0.89 at its default [18]. Averaging opaque colors
   reads as semi-transparency [110]. A constant ray-cone angle over-filters the
   periphery of a planar projection [111].

**Rejected.** Per-ray, per-eye footprint LOD. Color averaging, which invents
materials outside the palette. Majority or density-threshold occupancy, which
drops thin features. Correlation-aware SDF prefiltering, which breaks flat
faces [112]. A bare hard switch.

**Contract impact.** A new rule, `lod: none | brick(bias, band)`. It carries
the LOD eye input separate from the rendered view, the 2:1 balance, OR
occupancy with the mode material, the mid-walk rule, the band weight, the
shadow level rule, roughness widening, and the mip builder in voxcore or
voxsurface. Goldens are fixed only after the band is.

### 3.7 Antialiasing

**Choice.** Add a contract rule `antialias: none | msaa4`:

1. Visibility at the standard 4x positions (0.375,0.125), (0.875,0.375),
   (0.125,0.625), (0.625,0.875) [113].
2. Each distinct surface is shaded once, at the center ray's plane intersection
   clamped into the face, and weighted by covered samples.
3. Transparency is walked per sample.
4. **Far-field variant.** For hits at level 1 or coarser, the samples the
   center hit's face does not cover share one sub-ray, cast through their
   centroid. The reference mirrors it.

The realtime march casts one center ray. Coverage of the 4 samples by the hit
face is analytic, from 4 ray-plane tests against the face square. Uncovered
samples come from neighbor cells for coplanar material edges and creases, using
the 2x2x2 neighborhood corner occlusion reads, when the footprint spans at most
one cell per axis. Grazing footprints and silhouettes cast sub-rays, starting
before the 3x3 minimum depth. A fixed per-tile silhouette budget falls back to
a GBAA-style neighbor blend, and the fallback is mirrored [112]. Discontinuity
detection stays inside the dispatch, in workgroup memory or subgroup
operations, so no full-resolution round trip returns (3.10). The raster path
uses hardware 4x MSAA resolved in tile. Hybrid proxies emit `sample_mask`.

**Evidence.**

1. VR titles use spatial AA. Valve's minimum is 4x MSAA because "your head
   never stops moving" [22], and Meta advises 4x and no higher [114].
2. Quest MSAA cost was measured at the 1216x1344 Quest 1 resolution and is
   workload dependent: +0.5 ms render and +0.27 ms resolve for a normal load
   [114].
3. ESVO measured 4 spp at about 3x the cost of 1 spp. Shrinking voxels to the
   pixel does not remove the need for AA [90].
4. Under msaa4 the share of pixels whose samples disagree is 1-(1-0.75/p)^2 for
   p px voxels: 14% at 10 px, 34% at 4 px, 61% at 2 px, and 78% at the 1.4 px
   LOD floor (derived). Without the far-field variant, edge work approaches 4x
   exactly where LOD was meant to cap cost. With it, a far pixel costs at most
   2 rays.
5. At 1.4 px cells a frontal footprint is about 0.71 cells, so the 2x2x2
   neighborhood covers coplanar cases there. Grazing footprints exceed a cell
   and fall to sub-rays.
6. WebGPU standardizes patterns only for 1 and 4 samples [113]. WGSL has
   `sample_mask` and `frag_depth` but no `interpolateAt*` [10].
7. Corner occlusion is continuous across coplanar faces. Per-corner shadows are
   not exactly continuous, because corner rays are inset into each face
   [76][115].

**Rejected.** TAA, blue-noise dither and stochastic transparency, for
determinism and VR smear. FXAA, SMAA and CMAA2 on the headset, whose edges
crawl under head motion; CMAA2 stays an optional phone tier [116]. An msaa8
level, which has no portable raster pattern and exceeds Meta's ceiling.
Neighbor color contrast flags, which flip across GPUs.

**Contract impact.** A new rule with the far-field variant and the silhouette
fallback. Supersampled goldens also make cross-GPU matching more robust, since
a flipped edge sample moves a pixel by a quarter of the contrast. The raster
path must use per-unit-face corner values: voxsurface's merged spans read only
their outer corners, which differs from the reference inside a span [115].

### 3.8 Lighting, shadows, and occlusion

**Choice.**

1. **An exact memo** keyed by (placement, LOD, cell, face axis and sign),
   shared by both eyes.
2. **Per light, one value per face or four per corner:** 1 bit while the ray
   crossed only opaque or empty cells, f16 RGB packed through `pack2x16float`
   once it crossed a transmissive pass.
3. **Invalidation is exact, so there are no stale entries.** Epochs run per
   light, per light-space tile under a moving caster's old and new bounds and
   under edits, and per placement. An entry is valid or missing. An invalidated
   entry is a miss.
4. **Fill on the compute arm.** A tile workgroup collects its hit face keys in
   shared memory, dedups them, and looks them up in a global hash table. Hits
   shade inline. The workgroup traces its misses cooperatively in the same
   dispatch, then shades those pixels. An entry is claimed with a 32-bit atomic
   and published after its value is written. Two tiles missing the same key
   both trace it and write the same value, since each ray is a pure function of
   its key [76]. The race wastes work but cannot change a pixel.
5. **Fill on the raster and fragment arms.** A fragment cannot wait on a
   compute pass inside its render pass, and a fragment shader with side effects
   loses LRZ on turnip [58]. So the memo fills per brick before the pass. When
   a brick first becomes a draw candidate, by entering the frustum, changing
   level, or being invalidated, a compute pass traces every exposed face of
   that brick. The render pass only reads. Cost tracks bricks entering the draw
   set, plus the hidden faces of drawn bricks. B5 measures that overhead.
6. **Light changes commit double-buffered.** When a shadowed light moves, the
   renderer keeps the old light state, BRDF terms included, until the new memo
   is complete, then swaps. Every frame is the converged image of a valid light
   state, late by the fill time, never a mix of two. Authored light motion such
   as a sun steps in increments.
7. **Continuously moving lights invalidate every entry every frame.** That
   covers camera-frame and node-frame lights that follow the head, and a
   smoothly moving sun. Per-face for every visible face would cost about 222K
   to 740K rays per frame, 1.4 to 16 ms at the estimated rate (derived). On
   Quest such a light takes a mirrored tier rule, which B5 settles:
   - a light that follows the head casts no shadow. Its shadows fall behind
     their casters, hidden from both eyes but for a sliver from the eye offset
   - a hand-held light keeps per-face shadows inside its range and cone where
     B5 shows they hold the budget, and casts none otherwise
   - a sun moves in steps

   Desktop casts per-pixel shadows inline, a cost bounded by pixels.
8. **Moving casters and edits** invalidate the light-space tiles under them.
   Those misses are traced in the frame. Their cost scales with the receivers
   in the caster's light-space footprint.
9. **Over-budget frames fall back deterministically.** A missed per-corner
   entry is traced at per-face granularity that frame, one ray instead of four,
   flagged provisional, and upgraded later. Such a frame differs from the
   golden until upgraded, and the counter reports it.
10. **Teleports and snap turns hide behind a fade**, standard VR comfort
    practice (no source found in this pass). The destination pose is known when
    the move starts, so the memo prefills from it during the fade-out, and
    frames at full black skip primaries and spend the whole budget on the fill.
    Smooth turns are cheap: at 200 degrees/s and 90 Hz the view moves 2.2
    degrees per frame, about 2% of a 110-degree field, so about 4.4K new faces
    and 18K corner rays per frame (derived).
11. **Corner occlusion** is computed inline from an opaque-bit plane, reading 9
    cells per face, with no cache and no rays [115].
12. **BRDF terms** stay per pixel and per eye: Lambert, GGX with the per-eye
    view vector, point falloff, spot cone.

**Evidence.**

1. The reference casts face and corner rays from fixed face points [76]. Each
   ray is a pure function of its key, which makes exact memoization possible.
2. Lumen spends a fixed direct-lighting budget with feedback priority, and a
   full refresh takes about 16 frames [19]. That lag is designed in. This
   design moves the lag out of the cache and into the light commit, where the
   image stays consistent. UE5 VSM supplies the invalidation patterns [117],
   and Texel Shading tracks shade age per tile [118].
3. Fine-grained visibility is mandatory. In SAS, per-object culling needed 11
   to 96x the atlas of per-patch visibility [119].
4. Visibility is over 90% coherent frame to frame [120]. ORCA shows a
   history-free cache [121]. Texture-space neural materials hold over 90 FPS on
   Quest 3 by amortizing shading across space and time, which the contract does
   not allow, but which is a Quest precedent for an object-space cache [122].
5. Baker abandoned shared VR shading because its visibility was too coarse and
   its memory too high [123].
6. Costs (estimates):
   - Per-pixel shadows need about 5.9M rays per light per frame.
   - Shadow throughput on Quest is unmeasured. Scaling the GTX 680 SVDAG figure
     gives 45 to 155 Mrays/s [16].
   - A cold per-corner fill is about 890K rays at 16 px per face and about 3.5M
     at 4 px per face, so 6 to 80 ms. That is why cold fills hide behind fades
     and fall back to per-face.
7. N.L is not constant across a face for point or spot lights. Only its sign
   is.

**Rejected.**

1. Radiance caching, which bakes in per-eye GGX and per-pixel falloff.
2. Probe, surfel and cascade GI, which are stochastic and temporal and have no
   contract term [124].
3. Shadow maps. A shadow-map texel cannot reproduce the contract's rays from
   exact per-corner points [76]. This is our reasoning, not a cited result.
4. A refresh budget that shows stale entries until refreshed. That is wrong
   pixels during motion, the lag Lumen accepts.
5. Inline miss tracing in fragment shaders. It drops LRZ, duplicates rays per
   pixel, and runs long divergent walks inside wave128 fragment waves [50].
6. Cross-eye color reprojection. The barrier is the contract's per-eye GGX and
   glass [125][126].

**Contract impact.**

1. An optional maximum shadow distance.
2. Per-face shadows for faces at or above a coarse LOD, since per-corner costs
   more than per-pixel below about 4 px per face.
3. A VR tier rule for continuously moving lights: none for a head-locked light,
   per-face or none for a hand-held one, steps for a sun.
4. Goldens compare converged frames, or run an unlimited-budget mode.
5. The dyadic inset and boundary ownership (3.3) and the shadow level rule
   (3.6).

Desktop per-pixel shadows can be culled exactly with a per-face beam test,
classifying each face as lit, shadowed or mixed, cached and shared between
eyes. It is untested on voxels [16].

### 3.9 Transparency

**Choice.**

1. **Split placements.** A placement holding any material with nonzero pass is
   transparent. Opaque placements go through the arm's front end. Transparent
   placements go through one compute ordered walk on every arm.
2. **The walk.** Per screen tile, list the transparent placements overlapping
   it, sorted by entry distance. Each pixel steps their walks together by
   distance, merging hits as the contract does, from the eye to the opaque
   depth. It then adds throughput times the opaque color behind. Each
   placement's walk can restart from any t, since the inside material is the
   current cell's material and a point lookup recovers it.
3. **Opaque depth and color per arm:**
   - compute arm (a): in registers, in the same dispatch
   - raster and fragment arms (b, c, d): the opaque pass stores its depth when
     glass is visible, which the CPU knows from culling. At 4x MSAA that is 16
     B/px for D32 or D24S8, about 1.85 ms at default resolution before
     compression, or about 0.9 ms at D16 (derived). In those frames the opaque
     pass writes linear HDR in Rg11b10Ufloat instead of tonemapped output, and
     the walk tonemaps.
4. **Per-sample transparency** on raster arms reads the resolved opaque color.
   That is exact wherever the glass covers all four samples, and approximate
   only at glass silhouettes. B7 counts those pixels.
5. **Termination.** The contract already ends a walk at zero throughput or when
   the ray leaves every grid [92], so layers are bounded by the cells crossed.
   The new rule exists to bound cost in glass-dense scenes: a throughput
   epsilon or a layer cap, mirrored.
6. **Shadow rays through glass** run unordered, since throughput is a product.

**Evidence.** Multi-hit traversal is the ordered prior art [127]. Aokana is
opaque only [18]. Teardown glass is screen-door plus TAA [27]. The depth store
on raster arms may make glass-heavy frames cheaper on the compute arm. B7
decides.

**Rejected.** Single nearest-hit visibility buffers. Approximate OIT such as
AVBOIT, which fails layered-glass goldens. Raster blending of placements sorted
back to front, which fails interpenetrating glass. Depth peeling, where each
layer is another full pass with a tile store and reload.

**Contract impact.** The termination rule. A rule for which surface supplies
depth and motion vectors under glass, for example the first surface with zero
pass.

### 3.10 Post: bloom, tonemap, motion vectors

**Choice.** A fused path:

1. **A quarter-resolution pre-pass**, 420x440 per eye, about 6% of the primary
   rays, outputs the emission over the threshold as the bloom source, plus
   AppSW depth and motion vectors. Raster arms run it as a quarter-resolution
   raster of the same geometry.
2. **The bloom pyramid** runs at quarter resolution and below, as one compute
   dispatch or as fragment passes (B4).
3. **The primary pass** samples the halo bilinearly, adds it in linear light,
   applies PBR Neutral at the end of its shader, and writes the output once.
4. **Frames with visible glass on raster arms** tonemap in the transparency
   walk (3.9).
5. **HDR intermediates** use Rg11b10Ufloat, 4 B, which Quest can render, store,
   blend and filter [20].

**Evidence.**

1. Measured costs on Quest-class hardware:
   - a resolve costs 0.14 to 0.17 ms per eye on Quest 1 at 1216x1344 without
     MSAA, and 0.30 to 0.63 ms with 4x [114]
   - "producing a second image ... another resolve ... about 1ms" [128]
   - a full-screen pass without depth took 2.01 ms at 1216x1344 [12]
   - Godot's mobile glow chain takes 1.1 to 2.25 ms and its tonemap pass 1.3 to
     2.3 ms on Adreno 530 and 640 and Mali-G710 at phone resolution [13]
   - fusing the tonemap into the 3D shader saves "around 0.5 to 1ms" on mobile
     VR-class targets [129]
   - dual-filter bloom took 2.8 ms at 1080p on a Mali-T760 [130]
2. Tonemap can stay on chip, bloom cannot [130][131]. Adreno runs depthless
   full-screen passes direct to memory [12][62].
3. wgpu has no subpasses or input attachments; issue 10148 is open [53]. The
   Quest driver lacks local_read and shader_tile_image [65].
4. Shipped practice: Meta's forward-shading Unreal samples disable mobile HDR
   [132], Meta calls true bloom "extremely time consuming" [128], and Godot's
   XR maintainers recommend disabling glow [133].
5. AppSW motion vectors and depth "do not need to be full resolution", and more
   than the recommended size gains nothing [26]. Meta's Unity sample uses
   368x400 [134].
6. A march resolves transparency in its shader, so a fused tonemap is exact.
   Godot's fused path is approximate only because raster blending happens after
   the shader [129].
7. Estimates at default resolution (derived): a separate tonemap pass 1 to 2
   ms, a full-resolution bloom chain 1 to 2 ms, the fused path 0.4 to 0.8 ms in
   all.

**Rejected.** A conventional composite pass on Quest. A full-resolution bloom
source. Full-resolution motion vectors taken from primary outputs, which add
two full-resolution writes and a downsample, about 1.4 ms (derived).

**Contract impact.**

1. The bloom source and its octaves are defined at quarter resolution. The
   narrowest octaves, sigma 3.2 and 1.6 px at a 1680 px side out of six from
   sigma 50, are dropped. Godot found the first level "by far the most
   expensive" [13].
2. The bloom kernel becomes the pyramid filter the GPU runs, dual filter [130]
   or the 13-tap filter Godot uses [13], instead of a per-octave Gaussian [92].
3. An angular bloom radius for VR views.
4. The depth and motion-vector source rule under glass (3.9).

### 3.11 The VR pipeline

**Choice.**

1. **Stereo.** One multiview pass or one dispatch covers both eyes. The memo,
   LOD selection and tile lists are shared from the head center. Each eye's
   primary directions are quantized by the shared code (3.3). A shared
   conservative beam enclosing both eye origins is a benchmark item (B8).
2. **Foveation.** Benchmark three fixed routes:
   - quad views (v85+), where each view is an ordinary off-axis camera
   - in-app radial ray density with a deterministic fill that reuses a traced
     neighbor's face only when traced neighbors agree
   - FDM for the fragment and raster arms, with an app-owned density map
     through a wgpu-hal change
3. **Timewarp.** Submit depth every frame. Read the lens mask through
   XR_KHR_visibility_mask [24] and skip masked tiles. Late-latch poses.
4. **SpaceWarp.** Emit NDC motion vectors from the pre-pass when AppSW or
   XR_EXT_frame_synthesis is on [61]. AppSW is the last rung of a GPU-timer
   adaptive ladder [135].

**Evidence.**

1. Meta's FFR savings of 6.5, 11.5 and 21% come from one ALU-bound scene on an
   unnamed device, and simple shaders gain little [136]. A per-fragment march
   is the heavy-shader case.
2. App-generated density maps work on Quest 3 [137]. FDM forces tiled mode in
   turnip [60]. Godot's subsampled-FDM numbers compare subsampled against
   non-subsampled with FDM on in both runs, so they do not measure FDM against
   none [138].
3. Quad views claim a 50% pixel cut, but the outer views must cover the full
   field, inset included, and must not stack FFR. The pre-v85 Quest 3 inventory
   lists only stereo [139][24].
4. AppSW gives up to 70% more compute at 36/72 Hz on Quest 2, and less at 45/90
   [25]. Its artifacts are worst on flat backgrounds, glass and silhouettes
   [26].
5. The depth swapchain may be compute-writable:
   XR_META_vulkan_swapchain_create_info passes STORAGE usage through [61], and
   D32F and RGBA16F are storage-capable on Adreno 740 [20].
6. Gaze-driven foveation papers report 2.5 to 16x but rely on eye tracking,
   stochastic sampling or TAA [140][141][142][143].

**Rejected.** Cross-eye color reprojection. Gaze-centered log-polar resampling,
since Quest 3 has no gaze tracking. AppSW in the base budget.

**Contract impact.**

1. A stereo view: two off-axis projections with four angles each (XrFovf) and a
   head pose. Today the reference's projection is a symmetric vertical FOV
   [92].
2. The angular bloom radius (3.10).
3. Foveation is a tier with goldens off, unless its pattern is a pure function
   of pixel position that the reference mirrors.
4. Synthesized frames, compositor blends and runtime FDM maps stay outside
   goldens.

### 3.12 Scale, streaming, edits, and moving objects

**Choice.**

1. Placements of one object share its data.
2. One fixed brick pool behind a software page table, since wgpu has no sparse
   resources. A proposed 0.5 to 1 GiB of the 5.75 GiB PSS limit [144].
3. Residency predicted from pose and LOD error, with a rotation guard band.
   Coarse levels stay resident, and both band levels stay resident inside a
   band. Goldens run fully resident.
4. Edits use dirty-brick lists. Mips and LOD cells are recomputed only for
   touched parents, about 1/7 extra. Leaves are refit, and memo entries are
   invalidated through the light-space tile index.
5. Moving placements bump their own epoch and the light tiles under them.
6. Far field: offline HLOD proxy objects, which the reference renders as
   ordinary placements, in preference to a world-aligned merged clipmap.

**Evidence.**

1. Lumen streams into a 320 MB pool chosen by camera distance [19]. Aokana
   keeps 2% of a 64K scene resident and 5% at 32K [18]. A Claybook frame
   touched 8 MB of a 512 MB volume with a 99.85% cache hit rate [5].
2. GigaVoxels' ray feedback lags a frame, which head rotation exposes [106].
3. Lumen's 200 m HWRT cap "had a profoundly negative impact on the overall
   look" [19].
4. Nanite: "merged unique proxies must replace instances at extreme distances"
   [102].
5. NAADF's distance fields fill in background queues after a load or edit, so
   frame time depends on history until they finish [15].

**Rejected.** HashDAG and GPU SVDAG editing, which are CUDA-based with GB-scale
tables [94][145]. Pure ray-guided streaming. A world-aligned merged far field
for primaries, which changes silhouettes of rotated placements.

**Contract impact.** The far-field rule is a mirrored LOD rule. Edits change no
rules, but the epochs must make goldens independent of edit history at
convergence. If NAADF distances are adopted, their build finishes before first
display or runs as a bounded, counted per-frame step.

### 3.13 wgpu and platform integration

**Choice.**

1. Target wgpu 30.0.1 and track v31, which carries the ray-query-in-loop fix
   #9945 [66].
2. Use Quest-proven OpenXR glue in the bevy_mod_openxr pattern, raised to
   Vulkan 1.3 [146]. indite targets wgpu 30 but shows no Android path and warns
   of soundness holes [147].
3. Features by tier:
   - Quest required: MULTIVIEW, TIMESTAMP_QUERY, SHADER_EARLY_DEPTH_TEST,
     binding arrays, SUBGROUP
   - Quest optional: EXPERIMENTAL_RAY_QUERY
   - phone floor: none of SUBGROUP, SHADER_INT64 or 64-bit atomics
4. Serialize xrBeginFrame, xrEndFrame, acquire and release with Queue::submit
   [61]. wgpu exposes one queue [148], so async compute is unreachable. That
   rules out Claybook's 19% async gain and Arm's two-queue fix [5][149].
5. Ship hot shaders through create_shader_module_trusted after debug
   validation. naga otherwise adds a 64-bit loop counter to every loop [150].
6. Use fp32 shading, 32-bit atomics and keys, integer traversal, and no
   assumption about wave size.
7. Indirect draws reportedly render nothing on Quest, from one unreproduced
   report [151]. B0 tests it. The design does not depend on it: the hybrid
   classifies on the CPU, the raster arms fall back to direct draws over
   precomputed index ranges, and indirect dispatch is tested separately for the
   compute arm.
8. Treat FDM as a maintained wgpu-hal fork or an upstream feature. It needs
   render-pass key, attachment, layout-tracking, subsampled-flag and sampler
   changes, and no wgpu issue tracks it [70].
9. VK_QCOM_tile_shading would let compute work in tile memory inside a render
   pass [152]. wgpu does not expose it.
10. Defer visionOS until Metal ray queries are fixed [153].

**Evidence.**

1. Measured Quest 3 adapter: SHADER_F16 false, int64 atomics false, subgroups
   64 to 128 with no size control [20][69].
2. Ray query is enabled on DX12 and Metal in v30.0.1's hal, despite stale docs
   [66].
3. About a third of Vulkan devices in gpuinfo expose ray query [154].

**Rejected.** Designs that depend on fp16 through stock wgpu, on 64-bit atomics
on Quest, on RT pipelines on Quest, on mesh shaders, on async compute, or on
indirect draws.

**Contract impact.** None directly. The quantization code (3.3) is shared
between the reference and the GPU upload.

### 3.14 Phones

**Choice.**

1. **Two tiers with named devices.** The floor tier targets a mid-range Adreno
   6xx such as Adreno 610 and a mid-range Mali Bifrost or Valhall part. The
   flagship tier targets an Adreno 740 phone such as the Galaxy S23 and an
   Apple A17-class phone.
2. **The floor tier rasterizes.** Greedy meshes with 4x MSAA, fused forward
   shading, per-face or per-corner memo filled per brick, and dynamic
   resolution, with no subgroups, no int64 and no 64-bit atomics.
3. **The flagship tier runs the Quest hybrid** at phone resolution.
4. **On Mali, compute stays off the path between fragment passes**, and storage
   usage stays off render targets.

**Evidence.** Phones are not easy:

1. A 1080x2400 panel at 60 Hz is 156 Mpx/s, about 30% of Quest's 532 Mpx/s. An
   Adreno 610 is 243 GFLOP/s and 14.9 GB/s, and took 2.17 ms for direct
   lighting at 745x360 [43]. Quest's GPU is about 1.8 TFLOPS [50], about 7x
   more. So a floor phone has roughly half Quest's budget per pixel (derived).
2. The bottom half of phones have 16 KB uniform buffers, slow SSBOs, small or
   emulated group-shared memory, wave intrinsics on under 10% of devices, and
   no 64-bit atomics [42].
3. On Mali, compute shares the hardware queue with vertex and binning work. A
   fragment, compute, fragment sequence stalls fragment work [149], and a
   storage usage flag disables framebuffer compression [155]. wgpu's single
   queue cannot use Arm's two-queue fix.
4. The Galaxy S23 driver reports shaderInt64 false [75]. Godot's Mobile
   renderer supports compute "with a performance penalty on older devices"
   [156].
5. Thermal throttling on phones was not measured in this pass.

**Contract impact.** None new. Phone goldens render from the tier's settings,
such as per-face shadows.

## 4. What still scales with content, and how the design bounds each item

1. **Placements a ray enters before its first hit.** Bounded by the
   rasterizer's depth test on raster and fragment arms, and on the compute arm
   by tile lists sorted by entry depth with early-out, brick-cluster leaves,
   Hi-Z culling, and a far-field HLOD rule. B3 reports a histogram of entries
   per ray.
2. **Steps per ray, worst at grazing angles.** Bounded by wide masks, uniform
   collapse, optional empty-box distances, LOD at about 1.4 px, and the
   near-field raster. The cap is a counted diagnostic that goldens assert is
   never hit.
3. **Near-field raster faces.** The first visible layer is at most pixels/N^2.
   Depth complexity inside the shell is not bounded, so per-brick culling is
   needed.
4. **Cross-fade band pixels.** Pay two walks. Bounded by the band width,
   measured as a pixel share in B6.
5. **Transparent layers per pixel.** Bounded by the cells crossed [92] and the
   mirrored termination rule. Measured as a p95 count. Raster arms add a depth
   store in glass frames.
6. **Shadow ray length and objects stacked along a light column.** Bounded by
   the light-space grid, the unordered zero-throughput early-out, glTF range
   for point and spot lights, and an optional mirrored maximum distance.
   Receiver-level shadow walks let near receivers walk far casters at full
   detail, measured in B5.
7. **Lights.** Memo cost is shadowed lights x faces. A mirrored cap on shadowed
   lights per face, with a named fallback, is reserved if needed.
8. **Memo fills.** On the compute arm, bounded by newly visible faces, so by
   pixels. On raster arms, bounded by bricks entering the draw set, including
   their hidden faces. Spikes on teleports and snap turns hide behind fades,
   and over-budget frames fall back to per-face. The spike shows as time, as
   the fade, or as a counted provisional frame, never as an uncounted wrong
   pixel.
9. **Light commits.** A light change shows its new shadows after the fill
   completes. The image is late, never mixed. Continuously moving lights take
   the VR tier rule.
10. **Edge pixels needing sub-rays.** At most 4x near and 2x far, kept near the
    silhouette share by analytic coverage and the mirrored per-tile silhouette
    budget.
11. **Unique voxel memory.** Bounded by the brick pool, distance-driven
    residency, and leaf dedup. The PSS limit is the ceiling.
12. **TLAS build and binning.** Linear in placements or leaves, about 0.5 ms of
    CPU at 10^4 placements for a wgpu hardware TLAS [99]. Binning is measured
    at 10^3 to 10^5 placements.
13. **Edit relight reach.** Directional shadows run to infinity, so an edit
    invalidates every entry downstream in light space. Those are misses traced
    in the frame, or the over-budget fallback.
14. **CPU draw submission** on raster arms without indirect draws. Linear in
    brick clusters drawn, measured in B3.
15. **Moving placements.** Every visible face of a moving placement and every
    receiver in its light-space footprint refills each frame. Bounded by the
    moving placements' visible faces and footprints, not by pixels, with the
    over-budget fallback to per-face. Measured on S6 in B5.

## 5. Contract mapping

**Matched exactly:**

1. The traversal: the integer DDA on shared quantized inputs, with its tie
   order, inset and boundary ownership.
2. The surface rule: material boundaries, entering faces, per-placement walks,
   merge by distance, restart from inside a cell.
3. Pass and front-to-back throughput, ordered across placements.
4. Per-face and per-corner shadows through the memo, which casts the same rays
   from the same points. Colored transmissive shadows.
5. Corner occlusion, an integer rule.
6. Output alpha and background compositing.

**Approximated, and how it is checked:**

1. f32 shading against f64: within tolerance, checked in B1 on every target.
2. The hybrid's one `frag_depth` per pixel: counted in B6.
3. Per-sample transparency over resolved opaque color on raster arms: counted
   in B7.
4. Converged memo state: goldens use converged or unlimited-budget frames.
   Over-budget per-face fallbacks are counted.
5. Bloom: the quarter-resolution pyramid matches once the contract adopts it.
6. Foveated, quad-view, FDM, compositor, timewarp and AppSW output: outside
   goldens. Tiers render goldens with these off.

**What the reference must mirror (new contract rules):**

1. The integer traversal: F, DB, S, quantization, the 2^-10 inset, boundary
   ownership, and integer tie-breaks between axes and between placements.
2. A stereo view with two XrFovf frusta and a head pose.
3. `antialias: none | msaa4` with standard positions, per-surface shading at a
   clamped point, per-sample transparency, the far-field shared sub-ray, and
   the silhouette budget fallback if one exists.
4. `lod: brick(bias, band)`: the head-center LOD eye, 2:1 balance, OR occupancy
   with mode material and lowest-id ties, the mid-walk rule, the
   distance-weighted band, occlusion per level, roughness widening, and shadow
   rays at the receiver's level.
5. The far-field rule: HLOD proxy objects past footprint F.
6. Transparency termination: an epsilon or a layer cap.
7. Shadow rules: per-face at coarse LOD, an optional maximum distance, the VR
   rule for continuously moving lights, and an optional cap on shadowed lights.
8. Bloom at quarter resolution with the GPU's pyramid kernel, and an angular
   radius for VR.
9. Auxiliary outputs: which hit supplies depth and motion vectors.
10. Any foveation pattern a tier wants matched in goldens.

## 6. Risks, ranked

1. **Quest primary throughput.** Hypothesis: no front end fits the 3.5 ms
   primary line at default resolution. Nothing has been measured on the device.
   The rate, ALU and Claybook estimates all point to a shortfall of 3x or more.
   Shows in B2 and B3 as rates below 1.7 Grays/s, forcing the hybrid, raster,
   or a lower render scale.
2. **Per-step memory latency on Adreno.** Steps may cost far more than the ALU
   estimate if wave64 and wave128 occupancy cannot hide 34-cycle L1 and
   132-cycle cache hits [54][91]. Shows in B2 as ms per step well above the ALU
   figure.
3. **Post and bandwidth.** If the fused path fails, primaries get about 2 ms.
   Shows in B4.
4. **Integer traversal cost on Adreno.** The unexplained INT32 add result on
   Adreno X1 [91] could make the integer step slow. Shows in B1.
5. **Early-Z and LRZ on the proprietary driver.** Fragment proxies pay the full
   march for hidden fragments if LRZ is lost [58]. Shows as cost growing with
   proxy overdraw.
6. **Memo spikes.** Teleports, snap turns, light steps and edits trace 6 to 80
   ms of rays cold. Shows as over-budget frames and slow light commits in B5.
7. **Edge AA cost on fine voxels, foliage and the LOD floor.** 61% of pixels
   disagree at 2 px voxels and 78% at 1.4 px. Shows as AA time over 1.0 ms in
   B6.
8. **LOD seams and popping without history.** Ledges, internal faces and brick
   flips. Shows in headset review and per-pixel frame-to-frame variance on the
   scripted path.
9. **Transparency on raster arms.** The depth store costs about 0.9 to 1.85 ms
   in glass frames. Shows as an S3 surcharge over budget in B7.
10. **Preemption and compositor misses.** Long dispatches or untiled passes
    miss the compositor's deadline. Shows as stale frames that do not track app
    GPU time.
11. **Experimental wgpu features.** Ray-query bugs (#9945, #10287), FDM absent
    from wgpu, the indirect-draw report. Shows as wrong pixels, crashes, or a
    fork to maintain.
12. **OpenXR interop soundness and queue contention.** Shows as intermittent
    device loss or judder.
13. **Clocks and thermals.** Sustained level 5 needs dynamic resolution, and MR
    caps at 456 MHz [49]. Shows as drops after minutes of play.
14. **Memory.** The pool and caches compete with the app inside 5.75 GiB [144].
    Shows as lmkd kills or residency churn.
15. **Phone floor.** A floor phone has about half Quest's per-pixel budget and
    fewer features. Shows in B12.
16. **Reference scaling.** Giant-scene goldens become slow without culling in
    the reference. Shows as golden generation time.

## 7. Next phase: benchmarks and prototypes, in order

**Common setup.**

- **Scenes.** S1 is a voxel-art room at 1 to 2 cm voxels seen at arm's length.
  S2 is a forest or city with 10^3 to 10^4 placements, some rotated or scaled,
  at mixed voxel sizes. S3 is a glass-heavy greenhouse with overlapping glass
  placements. S4 is foliage and fences, the silhouette worst case. S5 is a
  large grazing floor plane. S6 puts eight walking, turning figures of 15
  rigid parts each in S1's room under a sun. The figures overlap at their
  joints. A flame cycles 8 objects at 12 Hz. A material pulses its emission.
  The bench scripts the motion.
- **Head paths.** One deterministic scripted path, a smooth turn at 200
  degrees/s, a 45-degree snap turn, and a teleport with fade.
- **Timing.** wgpu timestamps at pass boundaries only, since inside-pass
  timestamps are meaningless on tilers [66], confirmed with ovrgpuprofiler,
  whose detailed mode adds about 10% [157]. 300 warm-up frames, then 600
  measured frames, reporting p50 and p99. Fixed GPU level, temperature
  recorded.
- **Budget.** Each benchmark is judged against its line in 3.1. B10 judges the
  sum.

**B0. Device capability probe (Quest 3, retail Horizon OS).**

- **Question.** What do the shipping runtime and driver expose?
- **Setup.**
  - wgpu adapter features and limits, and Vulkan feature bits:
    shaderSharedInt64Atomics, shaderFloat16, shaderInt64
  - view configurations (quad views), XR_KHR_visibility_mask with its masked
    fraction, the recommended image rect, and XrSystemSpaceWarpPropertiesFB's
    recommended motion-vector size
  - swapchain creation with STORAGE and MUTABLE_FORMAT through
    XR_META_vulkan_swapchain_create_info, on color, D32F depth and RGBA16F
    motion vectors
  - XR_EXT_frame_synthesis
  - indirect draw and indirect dispatch
  - compositor GPU time
- **Threshold.** None. This is a gate.
- **Outcomes.** Each result prunes later steps. Without ray query, drop arm
  (e). If indirect draws fail, raster arms use direct draws. If STORAGE
  swapchains work, the compute arm writes the swapchain directly. If quad views
  exist, B9 includes them. The compositor time replaces the 1.3 ms placeholder
  in 3.1.

**B1. Integer traversal (reference, desktop GPUs, Quest 3, one phone).**

- **Question.** Does the integer DDA give the reference's cell and face ids on
  every target, and what does it cost on Adreno?
- **Setup.**
  - implement the DDA and quantization in the reference and in WGSL
  - render all goldens, fitted orthographic views of S1 and S5, and the
    45-degree-light corner shadow scenes from the simulation [9]
  - output an id buffer beside color
  - run on NVIDIA, AMD and Apple Metal, on Quest 3, and on a Galaxy S23-class
    phone
  - measure int32 add and compare throughput on Adreno 740
  - for the record, count f32 DDA flips on the same scenes
- **Threshold.** Id mismatches on at most 0.1% of samples per scene on every
  target, and color within tolerance on every other pixel. The integer step
  costs at most 1.25x a float DDA step on Quest.
- **Outcomes.** Pass: adopt it, change the reference, regenerate goldens. A
  share over the line is a bug, found and fixed before B3. If the step is too
  slow, reduce F or DB, or move the coarse loop to fewer bits. If it is still
  over 1.25x, Quest keeps a float DDA and its goldens tolerate the float
  walk's counted flips (Priority).

**B2. Ray rate and step cost (Quest 3).**

- **Question.** What coherent primary ray rate do our walk and hardware ray
  query reach on Adreno 740 through wgpu, and what does one step cost?
- **Setup.**
  - a fixed-N-step loop over a storage buffer, a sampled 3D texture, a 2D
    array, and u32-pair masks, regressing ms against N
  - the full 64-tree walk on S1 and S5 at 1680x1760x2
  - hardware ray query over brick AABBs on the same scenes
  - trusted shader modules, GPU level 4, wave size read from subgroup builtins
- **Threshold.** The rate R sets the largest marched pixel share the 3.5 ms
  line allows:
  - a full-frame march is viable at R of at least 1.7 Grays/s
  - a hybrid far field covering 30% of pixels is viable at R of at least 0.5
    Grays/s
  - below 0.25 Grays/s, the march serves only glass and shadows
- **Outcomes.** R picks which B3 arms are worth building. The 500 Mrays/s bar
  suggested in the gap research is the hybrid line here.

**B3. Primary visibility shootout (Quest 3; the core).**

- **Question.** Which front end delivers opaque primary visibility for both
  eyes within the 3.5 ms line?
- **Arms.** All share one per-object structure, the 64-tree with uniform
  collapse, plus an empty-box-distance variant:
  - (a) compute march with tile binning, in tile dispatches
  - (b) fragment march over front-face proxies with early depth and multiview
  - (c) the hybrid at N = 4 and N = 8, with the handoff in 3.2
  - (d) greedy mesh with 4x MSAA and multiview, with and without brick LOD
    meshes
  - (e) ray query TLAS with AABB brick BLAS
- **Equal shading load.** Every arm outputs the final opaque image: one
  directional light with per-corner values read from a prefilled static memo,
  hemisphere ambient, corner occlusion, GGX, emissive, the fused bloom add and
  tonemap, and one output write. Memo fill is excluded here and measured in B5.
- **Scenes.** S1 and S5 decide. S2 runs with LOD off as a stress test. S3 and
  S4 run as provisional until the B6 and B7 rules exist, then rerun as B3b.
  S6 reports its time with per-frame TLAS and binning included.
- **Setup.** 1680x1760x2 and 1344x1408x2. GPU level 4 fixed, and level 5 with
  dynamic resolution. TimeWarp on. If MR is a goal, a rerun at 456 MHz records
  the render scale that meets the line.
- **Measures.** ms p50 and p99, iterations per ray, cap hits, placements
  entered per ray, marched pixel share, LRZ state, DRAM read counters, triangle
  and draw counts for (c) and (d), CPU submission time, and compositor misses.
- **Threshold.** 3.5 ms or less at p99 on S1 and S5 at default resolution and
  level 4, with no compositor misses caused by the app.
- **Outcomes.**
  - A full march arm passes: it becomes the Quest path.
  - Only (c) passes: the hybrid is the Quest design.
  - Only (d) passes: raster primaries with traced memo shadows, the SVDAG
    pattern [16]. The march serves glass, desktop and flagship phones.
  - Passing only with more time: take it from the bloom and shadow lines first,
    then adopt a 0.8 render scale plus dynamic resolution (Priority).

**B4. Post (Quest 3).**

- **Question.** Does the fused path fit its 0.6 ms of lines, and how far off is
  a conventional chain?
- **Setup.**
  - a full-screen read-and-write pass, Rg11b10Ufloat against Rgba16Float into
    sRGB, fragment against compute
  - bloom pyramids, dual filter against 13-tap, from half and quarter
    resolution, as fragment passes and as one dispatch
  - the fused path: quarter-resolution pre-pass plus in-shader add and tonemap
  - motion vectors and depth from a low-resolution pass against full-resolution
    outputs
  - default and panel resolution, GPU level 5 and level 4
- **Threshold.** Fused post, pre-pass plus pyramid plus the in-shader overhead,
  at 0.6 ms or less. Bloom under the quarter-resolution rule within tolerance
  of the reference.
- **Outcomes.** Pass: the fused path and its bloom rules enter the contract.
  Fail: the primary line drops to about 2 ms and B3 is rejudged against it.

**B5. Shadow memo (Quest 3).**

- **Question.** What do fills, misses and corner rays cost under each
  mechanism?
- **Setup.** The per-face inline fill (compute arm) and the per-brick prefill
  (raster arms). A sun plus one point light, opaque scenes against S3. Every
  head path, plus a sun step, a moving caster, S6, an edit, a head-relative
  light, and a hand-held spot light swept by a controller. Insert cost with
  32-bit atomics.
- **Measures.** Distinct visible faces, new faces per frame, faces S6's motion
  invalidates per frame, fallback frames on S6, corner rays per second, the
  per-brick superset ratio, frames to converge after a teleport and after a
  light step, provisional frame counts.
- **Threshold.** Steady state and smooth turns at 1.0 ms or less. Teleports and
  snap turns converge within the fade with no frame over budget. A sun step
  commits within 30 frames.
- **Outcomes.** Pass: per-corner is the Quest default. Steady-state fail:
  per-face becomes the Quest tier default, mirrored. Cold fail: a longer fade,
  or per-face fallback as the documented behavior. The hand-held light's cost
  sets its tier rule (3.8).

**B6. AA and LOD (desktop prototype first, then Quest).**

- **Question.** How many pixels are edges, what does msaa4 cost with its
  far-field variant, and do the LOD rules hide seams and pops in the headset?
- **Setup.**
  - classify edge pixels as coplanar, crease or silhouette at 2 mm, 1 cm and 5
    cm voxels at LOD 0, and at the LOD floor with 1.4 px and 2 px cells, on S1,
    S2 and S4
  - time msaa4 against brute-force 4x, with and without the far-field variant,
    detecting discontinuities inside the dispatch
  - count hybrid pixels where one `frag_depth` changes the result
  - compare id and filtered mips, and band widths from 2 cells to n/10,
    measuring band pixel share and per-pixel frame-to-frame variance on the
    scripted path
  - headset review at biases 1.0 and 1.4 px of a large uniform wall under slow
    head translation, one-voxel fences, and mid-periphery content
- **Threshold.** AA at 1.0 ms or less at default resolution. Silhouette
  sub-rays on 10% of pixels or fewer on S1 to S3. Hybrid depth mismatches on
  under 0.1% of pixels. No visible pop, ledge or seam at the chosen bias and
  band.
- **Outcomes.** Fix the AA and LOD rules in the contract and render the
  goldens. If S4 fails, add the mirrored silhouette budget or raise N so the
  hybrid's raster MSAA covers more of the scene. If the far-field variant
  crawls in the headset, drop it and pay for full msaa4 with a coarser LOD
  bias. Where shimmer and sharpness pull the bias apart, the coarser bias wins
  (Priority).

**B7. Transparency (Quest 3).**

- **Question.** What do transparent layers cost per arm?
- **Setup.** S3: layer histograms and cost per layer under the epsilon and cap
  rules. The compute arm inline, and the raster arms with the depth store at
  D16 and D32. Count glass-silhouette pixels where per-sample transparency over
  the resolved color differs from the reference.
- **Threshold.** S3 surcharge at 1.0 ms or less, or 1.9 ms with the
  contingency.
- **Outcomes.** Choose the termination rule and value, and add an
  overlapping-glass golden. If the raster arms fail only on the depth store,
  glass frames switch to the compute walk's own opaque pass.

**B8. Beam pre-pass (winner of B3, if it marches).**

- **Question.** Does a conservative start-depth pass pay for itself once
  traversal is shallow?
- **Setup.** No beam, a per-eye 8x8 beam, and one shared beam enclosing both
  eyes, on S1 and S2.
- **Threshold.** Net primary time falls by 10% or more, the pass included.
  Prior results: 0.2 ms for Claybook's cone pass [5], 1.33x on an iGPU [33], +2
  to +25% on ESVO [90].
- **Outcomes.** Keep the best variant or drop the beam.

**B9. Foveation routes (Quest 3).**

- **Question.** Which fixed-foveation route saves the most without visible
  artifacts?
- **Setup.** Quad views if B0 found them, in-app radial density with the
  deterministic fill, and FDM through a throwaway hal patch. Savings against
  full density, plus a headset review of seams and peripheral shimmer.
- **Threshold.** Primary time falls by 20% or more with no visible seam or
  crawl.
- **Outcomes.** Adopt the best route. If FDM wins, propose it upstream to wgpu.

**B10. Integrated sustained run (Quest 3).**

- **Question.** Does the full frame fit 9.1 ms, and does it hold thermally?
- **Setup.** The chosen configuration on S1 to S5 with every line active, then
  30 minutes on S1 and S2 in VR only, plus passthrough at level 2 if MR is a
  goal.
- **Threshold.** The sum at 9.1 ms or less at p99, fewer than 1 stale frame per
  minute, and no level drop.
- **Outcomes.** Set the shipping GPU level, the adaptive ladder, and the engine
  reserve.

**B11. Desktop and PCVR.**

- **Question.** Does hardware ray query beat a software top level on desktop,
  and are per-pixel shadows affordable?
- **Setup.** The hardware ray query TLAS against the software top level, and
  per-pixel shadows with and without the per-face beam test. RTX 3060-class and
  RDNA GPUs at 3840x2160 and at PCVR 2x2064x2208.
- **Threshold.** Primary time at 4 ms or less at 4K on an RTX 3060-class GPU,
  and per-pixel shadows adding 3 ms or less.
- **Outcomes.** Pick the desktop top level and whether per-pixel shadows are
  the desktop default.

**B12. Phones.**

- **Question.** Do the floor and flagship tiers hold 60 Hz under throttling?
- **Setup.** An Adreno 610-class and a mid-range Mali phone on the raster tier,
  and a Galaxy S23-class phone on the hybrid, at native resolution on S1 and
  S2. Ten-minute sustained runs, temperature recorded.
- **Threshold.** The floor sustains 60 Hz at a render scale of 0.7 or more. The
  flagship sustains the hybrid at 60 Hz.
- **Outcomes.** Fix each phone tier's settings and goldens, or drop the
  flagship hybrid in favor of raster.

**Prerequisite prototypes in voxrender (alongside B0 to B2).** Every tier
judged after B3 needs these in the reference first:

1. the integer traversal and its quantization
2. the stereo view projection
3. msaa4 with the far-field variant
4. brick LOD with OR occupancy, the mid-walk rule and the band
5. transparency termination
6. quarter-resolution bloom with the pyramid kernel
7. culling over placements

## 8. Sources

1. Meta. Render Scale.
   https://developers.meta.com/vr/documentation/native/android/os-render-scale/
2. Biswas, Bourd (Qualcomm). Linear Ray Tracing (HPG poster), 2025.
   https://highperformancegraphics.org/2025/publications/posters/Linear%20Ray%20Tracing%20-%20HPG2025.pdf
3. Frolov et al. Cross-RT: cross-platform hardware-accelerated ray tracing,
   2024. https://arxiv.org/abs/2409.12617
4. dubiousconst282. VoxelRT benchmarks, 2024-2025.
   https://github.com/dubiousconst282/VoxelRT
5. Aaltonen. GPU-based clay simulation and ray-tracing tech in Claybook, GDC
   2018.
   https://media.gdcvault.com/gdc2018/presentations/Aaltonen_Sebastian_GPU_Based_Clay.pdf
6. Vibrant Visuals, Minecraft Wiki, 2025.
   https://minecraft.wiki/w/Vibrant_Visuals
7. QuestCraft. https://github.com/QuestCraftPlusPlus/QuestCraft
8. VivecraftMod. https://github.com/Vivecraft/VivecraftMod
9. Precision simulations from this research, run with python3 from their
   folder: tie.py, flips.py, primary2.py, intdda.py, entry32.py, 2026.
   [precision-sim](precision-sim/)
10. W3C. WGSL specification, CRD 21 Sep 2026. https://www.w3.org/TR/WGSL/
11. wgpu repository at commit 1468594 (naga back/spv, back/msl, back/hlsl;
    wgpu-hal metal device.rs), 2026. https://github.com/gfx-rs/wgpu
12. Meta (Palandri). Loads, Stores, Passes, and Advanced GPU Pipelines, 2020.
    https://developers.meta.com/horizon/blog/loads-stores-passes-and-advanced-gpu-pipelines/
13. Godot PR 110077: Overhaul and optimize Glow in the mobile renderer.
    https://github.com/godotengine/godot/pull/110077
14. dubiousconst282. A guide to fast voxel ray tracing using sparse 64-trees,
    2024. https://dubiousconst282.github.io/2024/10/03/voxel-ray-tracing/
15. Ulschmid et al. NAADF: Globally Illuminated Voxel Worlds Accelerated with
    Nested Axis-Aligned Distance Fields, 2026.
    https://diglib.eg.org/handle/10.1111/cgf70413
16. Kampe, Sintorn, Assarsson. High Resolution Sparse Voxel DAGs, 2013.
    https://www.cse.chalmers.se/~uffe/HighResolutionSparseVoxelDAGs.pdf
17. Molenaar, Eisemann. Transform-Aware Sparse Voxel DAGs, 2025.
    https://dl.acm.org/doi/10.1145/3728301
18. Fang, Wang, Wang. Aokana: A GPU-Driven Voxel Rendering Framework for Open
    World Games, 2025. https://arxiv.org/abs/2505.02017
19. Wright, Narkowicz, Kelly. Lumen: Real-time Global Illumination in Unreal
    Engine 5, 2022.
    https://advances.realtimerendering.com/s2022/SIGGRAPH2022-Advances-Lumen-Wright%20et%20al.pdf
20. zonkypop. Quest 3 wgpu-info output, 2026.
    https://gist.github.com/zonkypop/bc6adcb01e07f41c37fb4db671ae2ed1
21. Giegl, Wimmer. Unpopping: Solving the Image-Space Blend Problem for Smooth
    Discrete LOD Transitions, 2007.
    https://www.cg.tuwien.ac.at/research/publications/2007/GIEGL-2007-UNP/GIEGL-2007-UNP-Preprint.pdf
22. Vlachos. Advanced VR Rendering, GDC 2015.
    https://media.steampowered.com/apps/valve/2015/Alex_Vlachos_Advanced_VR_Rendering_GDC2015.pdf
23. Krajancich, Kellnhofer, Wetzstein. A Perceptual Model for
    Eccentricity-dependent Spatio-temporal Flicker Fusion, 2021.
    https://arxiv.org/abs/2104.13514
24. Khronos. OpenXR-Inventory, Meta Quest 3 runtime.
    https://raw.githubusercontent.com/KhronosGroup/OpenXR-Inventory/main/runtimes/meta_quest_3_mobile.json
25. Meta. Introducing Application SpaceWarp, 2021.
    https://developers.meta.com/vr/blog/introducing-application-spacewarp/
26. Meta. Application SpaceWarp Developer Guide (native).
    https://developers.meta.com/horizon/documentation/native/android/mobile-asw/
27. Wittens. Teardown Frame Teardown, 2023.
    https://acko.net/blog/teardown-frame-teardown/
28. Montoya. Teardown Teardown.
    https://juandiegomontoya.github.io/teardown_breakdown.html
29. Rundlett, Gustafsson. Raytracing Voxels in Teardown and Beyond (GPC
    slides), 2025.
    https://static.graphicsprogrammingconference.com/public/2025/talks/raytracing-voxels-in-teardown-and-beyond/Rundlett-Gustafsson-raytracing-voxels-in-teardown-and-beyond.pdf
30. Gustafsson. Voxagon year summary, 2024.
    https://blog.voxagon.se/2024/12/29/year-summary.html
31. Dwyer. Parallax ray marching, Voxel Devlog 4, 2022.
    https://www.youtube.com/watch?v=h81I8hR56vQ
32. Dwyer. Adding ray tracing (back), Voxel Devlog 17, 2024.
    https://www.youtube.com/watch?v=aY4Zet_C9Zs
33. Dwyer. Doubling the speed of my game's graphics, Voxel Devlog 18, 2024.
    https://www.youtube.com/watch?v=P2bGF6GPmfc
34. Evans. Learning from Failure (Dreams renderers), 2015.
    http://advances.realtimerendering.com/s2015/AlexEvans_SIGGRAPH-2015-sml.pdf
35. Dreams (video game), Wikipedia.
    https://en.wikipedia.org/wiki/Dreams_(video_game)
36. Large Scale Animated Foliage in The Witcher 4 UE5 Tech Demo, 2025.
    https://www.youtube.com/watch?v=EdNkm0ezP0o
37. Epic. Nanite Foliage documentation, 2025.
    https://dev.epicgames.com/documentation/unreal-engine/nanite-foliage
38. Epic. Nanite Virtualized Geometry (UE 5.8 docs).
    https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine
39. Minecraft with RTX, Wikipedia.
    https://en.wikipedia.org/wiki/Minecraft_with_RTX
40. Veloren greedy mesher.
    https://gitlab.com/veloren/veloren/-/raw/master/voxygen/src/mesh/greedy.rs
41. Roblox creator-docs, Technology enum.
    https://github.com/Roblox/creator-docs/blob/main/content/en-us/reference/engine/enums/Technology.yaml
42. Aaltonen. Modern Mobile Rendering @ HypeHype, REAC 2023.
    https://enginearchitecture.org/downloads/reac2023_modern_mobile_rendering_at_hypehype.pdf
43. Lempiainen. Stochastic Tile-Based Lighting in HypeHype, Advances 2025.
    https://advances.realtimerendering.com/s2025/content/s2025_stb_lighting_v1.1_notes.pdf
44. Realtime Global Illumination in Enshrouded, GPC 2024.
    https://www.youtube.com/watch?v=57F1ezwH7Mk
45. Rundlett. gvox_engine, 2026. https://github.com/GabeRundlett/gvox_engine
46. jms55. Realtime Raytracing in Bevy 0.17 (Solari), 2025.
    https://jms55.github.io/posts/2025-09-20-solari-bevy-0-17/
47. Schutz, Kerbl, Wimmer. Software Rasterization of 2 Billion Points in Real
    Time, 2022. https://arxiv.org/abs/2204.01287
48. Meta. VrApi logcat stats.
    https://developers.meta.com/horizon/documentation/native/android/ts-logcat-stats/
49. Meta. CPU and GPU levels.
    https://developers.meta.com/horizon/documentation/unity/os-cpu-gpu-levels/
50. azhirnov. cpu-gpu-arch, Adreno-700 notes.
    https://github.com/azhirnov/cpu-gpu-arch/blob/main/gpu/Adreno-700.md
51. Qualcomm. Snapdragon XR2 Gen 2 Platform Product Brief.
    https://docs.qualcomm.com/doc/87-73689-1/87-73689-1_REV_A_Snapdragon_XR2_Gen_2_Platform_Product_Brief.pdf
52. Meta Quest 3, Wikipedia (memory figures conflict with Qualcomm's brief).
    https://en.wikipedia.org/wiki/Meta_Quest_3
53. wgpu issue 10148: RenderPipeline input attachments, 2026.
    https://github.com/gfx-rs/wgpu/issues/10148
54. Chips and Cheese. Inside Snapdragon 8+ Gen 1's iGPU: Adreno Gets Big, 2024.
    https://chipsandcheese.com/p/inside-snapdragon-8-gen-1s-igpu-adreno-gets-big
55. Qualcomm. Adreno GPU Game Developer Guide, shader best practices
    (archived).
    https://developer.qualcomm.com/sites/default/files/docs/adreno-gpu/developer-guide/gpu/best_practices_shaders.html
56. Khronos. VK_EXT_fragment_density_map reference page.
    https://docs.vulkan.org/refpages/latest/refpages/source/VK_EXT_fragment_density_map.html
57. Mesa. Fragment Density Map in turnip.
    https://docs.mesa3d.org/drivers/freedreno/fdm.html
58. Mesa. turnip tu_lrz.cc, 2026.
    https://gitlab.freedesktop.org/mesa/mesa/-/raw/main/src/freedreno/vulkan/tu_lrz.cc
59. Mesa. freedreno_devices.py and turnip tu_image.cc, 2026.
    https://gitlab.freedesktop.org/mesa/mesa/-/raw/main/src/freedreno/common/freedreno_devices.py
    and
    https://gitlab.freedesktop.org/mesa/mesa/-/raw/main/src/freedreno/vulkan/tu_image.cc
60. Mesa. turnip tu_autotune.cc and tu_cmd_buffer.cc, 2026.
    https://gitlab.freedesktop.org/mesa/mesa/-/raw/main/src/freedreno/vulkan/tu_autotune.cc
    and
    https://gitlab.freedesktop.org/mesa/mesa/-/raw/main/src/freedreno/vulkan/tu_cmd_buffer.cc
61. Khronos. OpenXR 1.1 specification, 2026.
    https://registry.khronos.org/OpenXR/specs/1.1/html/xrspec.html
62. Mesa. Freedreno driver docs (direct and tiled modes, autotune, preemption).
    https://docs.mesa3d.org/drivers/freedreno.html
63. Meta. Performance targets.
    https://developers.meta.com/horizon/documentation/unity/unity-perf/
64. Meta. Developer Insights: Vulkan for Mobile VR Rendering, 2019.
    https://developers.meta.com/horizon/blog/vulkan-for-mobile-vr-rendering/
65. tcoppex. Meta Quest 3 Vulkan extensions (driver 0.837.7), 2025-2026.
    https://gist.github.com/tcoppex/6ef9f5f60ddae41d6198d2bdf4a40beb
66. wgpu CHANGELOG, 2026.
    https://raw.githubusercontent.com/gfx-rs/wgpu/trunk/CHANGELOG.md
67. Abbott. Ray Tracing for Adreno GPUs on Turnip, XDC 2024.
    https://indico.freedesktop.org/event/6/contributions/302/attachments/225/305/Ray%20Tracing%20for%20Adreno%20GPUs%20on%20Turnip%20(XDC%202024).pdf
68. Wald et al. RTX Beyond Ray Tracing, 2019.
    https://www.sci.utah.edu/~wald/Publications/2019/rtxPointQueries/rtxPointQueries.pdf
69. Vulkan Hardware Database, report 48433 (Oculus Quest 3, driver 512.837.7).
    https://vulkan.gpuinfo.org/displayreport.php?id=48433
70. wgpu-hal Vulkan adapter.rs, 2026.
    https://raw.githubusercontent.com/gfx-rs/wgpu/trunk/wgpu-hal/src/vulkan/adapter.rs
71. Majercik et al. A Ray-Box Intersection Algorithm and Efficient Dynamic
    Voxel Rendering, 2018. https://jcgt.org/published/0007/03/04/paper.pdf
72. Cohen. Voxel Traversal along a 3D Line, Graphics Gems IV, 1994 (code).
    https://github.com/erich666/GraphicsGems/blob/master/gemsiv/vox_traverse.c
73. Amanatides, Woo. A Fast Voxel Traversal Algorithm for Ray Tracing, 1987.
    http://www.cse.yorku.ca/~amana/research/grid.pdf
74. voxrender reference ray walk, 2026.
    [render_ray_walk.rs](../../../../../projects/utilities/voxrender/src/render_ray_walk.rs)
75. Vulkan Hardware Database, report 51086 (Samsung SM-S911U1, Adreno 740,
    driver 512.676.1). https://vulkan.gpuinfo.org/displayreport.php?id=51086
76. voxrender render.rs: shadow_factor, BIAS, INSET, 2026.
    [render.rs](../../../../../projects/utilities/voxrender/src/render.rs)
77. Khronos. Vulkan specification, Precision and Operation of SPIR-V
    Instructions.
    https://registry.khronos.org/vulkan/specs/latest/html/vkspec.html#spirvenv-precision-operation
78. Khronos. SPIR-V Unified Specification.
    https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html
79. Apple. Metal Shading Language Specification 4.1, 2026.
    https://developer.apple.com/metal/Metal-Shading-Language-Specification.pdf
80. Apple. MTLCompileOptions.mathMode.
    https://developer.apple.com/documentation/metal/mtlcompileoptions/mathmode
81. Microsoft. D3D11.3 Functional Specification, section 3.1.
    https://microsoft.github.io/DirectX-Specs/d3d/archive/D3D11_3_FunctionalSpec.htm
82. Mesa. ac_nir.c (RADV NIR options).
    https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/amd/common/nir/ac_nir.c
83. Mesa. freedreno ir3_compiler.c and ir3_compiler_nir.c.
    https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/freedreno/ir3/ir3_compiler.c
84. Mesa. Asahi agx_compile.c (libagx_frcp).
    https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/asahi/compiler/agx_compile.c
85. NVIDIA. PTX ISA (add, div, rcp).
    https://docs.nvidia.com/cuda/parallel-thread-execution/index.html
86. Mesa. nir_opcodes.py and nir_opt_algebraic.py.
    https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/compiler/nir/nir_opcodes.py
87. Woop, Benthin, Wald. Watertight Ray/Triangle Intersection, 2013.
    https://jcgt.org/published/0002/01/05/
88. Ize. Robust BVH Ray Traversal, 2013. https://jcgt.org/published/0002/02/02/
89. Shewchuk. Adaptive Precision Floating-Point Arithmetic and Fast Robust
    Geometric Predicates. https://www.cs.cmu.edu/~quake/robust.html
90. Laine, Karras. Efficient Sparse Voxel Octrees: Analysis, Extensions, and
    Implementation (tech report), 2010.
    https://research.nvidia.com/sites/default/files/pubs/2010-02_Efficient-Sparse-Voxel/laine2010tr1_paper.pdf
91. Chips and Cheese. The Snapdragon X Elite's Adreno iGPU, 2024.
    https://chipsandcheese.com/p/the-snapdragon-x-elites-adreno-igpu
92. The render contract, 2026.
    [contract.md](../../../../ref/render/contract.md)
93. Arbore et al. Hybrid Voxel Formats for Efficient Ray Tracing, 2024.
    https://arxiv.org/abs/2410.14128
94. Molenaar, Eisemann. Editing Compact Voxel Representations on the GPU, 2024.
    https://diglib.eg.org/bitstream/handle/10.2312/pg20241310/pg20241310.pdf
95. Villanueva, Marton, Gobbetti. Symmetry-aware Sparse Voxel DAGs, 2017.
    https://jcgt.org/published/0006/02/01/paper.pdf
96. Soderlund, Evans, Akenine-Moller. Ray Tracing of Signed Distance Function
    Grids, 2022. https://jcgt.org/published/0011/03/06/paper-lowres.pdf
97. Benthin, Woop, Wald, Afra. Improved Two-Level BVHs using Partial
    Re-Braiding, 2017. https://www.embree.org/papers/2017-HPG-openmerge.pdf
98. Benthin et al. H-PLOC, 2024. https://gpuopen.com/download/HPLOC.pdf
99. wgpu PR 9877: build TLAS from instances buffer, 2026.
    https://github.com/gfx-rs/wgpu/pull/9877
100. Losasso, Hoppe. Geometry Clipmaps, 2004.
     https://hhoppe.com/geomclipmap.pdf
101. Asirvatham, Hoppe. Terrain Rendering Using GPU-Based Geometry Clipmaps,
     GPU Gems 2 ch. 2.
     https://developer.nvidia.com/gpugems/gpugems2/part-i-geometric-complexity/chapter-2-terrain-rendering-using-gpu-based-geometry
102. Karis, Stubbe, Wihlidal. Nanite: A Deep Dive, 2021.
     https://advances.realtimerendering.com/s2021/Karis_Nanite_SIGGRAPH_Advances_2021_final.pdf
103. Laine, Karras. Efficient Sparse Voxel Octrees, I3D 2010.
     https://users.aalto.fi/~laines9/publications/laine2010i3d_paper.pdf
104. Distant Horizons wiki. Problems and Solutions.
     https://gitlab.com/distant-horizons-team/distant-horizons/-/wikis/1-user-guide/1-frequently-asked-questions/2-problems-and-solutions/Problems-and-Solutions
105. Strugar. Continuous Distance-Dependent Level of Detail for Rendering
     Heightmaps.
     https://raw.githubusercontent.com/fstrugar/CDLOD/master/cdlod_paper_latest.pdf
106. Crassin et al. GigaVoxels: Ray-Guided Streaming, 2009.
     https://maverick.inria.fr/Publications/2009/CNLE09/CNLE09.pdf
107. Unity. URP Asset reference (LOD Cross Fade Dithering Type).
     https://docs.unity3d.com/6000.0/Documentation/Manual/urp/universalrp-asset.html
108. Epic. Material Properties (Dithered LOD Transition).
     https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-engine-material-properties
109. Patney et al. Towards Foveated Rendering for Gaze-Tracked Virtual Reality,
     2016.
     https://research.nvidia.com/publication/2016-12_towards-foveated-rendering-gaze-tracked-virtual-reality
110. Crassin et al. Interactive Indirect Illumination Using Voxel Cone Tracing,
     2011.
     https://research.nvidia.com/sites/default/files/publications/GIVoxels-pg2011-authors.pdf
111. Akenine-Moller et al. Texture Level of Detail Strategies for Real-Time Ray
     Tracing (Ray Tracing Gems ch. 20), 2019.
     https://media.contentapi.ea.com/content/dam/ea/seed/presentations/2019-ray-tracing-gems-chapter-20-akenine-moller-et-al.pdf
112. Filtering Approaches for Real-Time Anti-Aliasing course, incl. Persson
     GBAA, 2011. https://www.iryoku.com/aacourse/
113. W3C. WebGPU specification, 2026. https://www.w3.org/TR/webgpu/
114. Meta. Multisample Anti-Aliasing Analysis for Meta Quest.
     https://developers.meta.com/vr/documentation/native/android/mobile-msaa-analysis/
115. voxsurface corner_occlusion.rs, 2026.
     [corner_occlusion.rs](../../../../../projects/utilities/voxsurface/src/corner_occlusion.rs)
116. Intel. Conservative Morphological Anti-Aliasing 2.0, 2018.
     https://www.intel.com/content/www/us/en/developer/articles/technical/conservative-morphological-anti-aliasing-20.html
117. Epic. Virtual Shadow Maps documentation, 2024.
     https://dev.epicgames.com/documentation/en-us/unreal-engine/virtual-shadow-maps-in-unreal-engine
118. Hillesland, Yang. Texel Shading, 2016.
     https://gpuopen.com/learn/texel-shading/
119. Mueller et al. Shading Atlas Streaming, 2018.
     https://www.tugraz.at/fileadmin/user_upload/Institute/ICG/Images/team_steinberger/SAS/shading_atlas_streaming.pdf
120. Mueller et al. Temporally Adaptive Shading Reuse for Real-Time Rendering
     and Virtual Reality, 2021. https://doi.org/10.1145/3446790
121. Advances in Real-Time Rendering in Games 2026 (ORCA), 2026.
     https://advances.realtimerendering.com/s2026/index.html
122. Xu et al. Real-Time Neural Materials on Mobile VR, 2026.
     https://diglib.eg.org/handle/10.1111/cgf70318
123. Baker, Jarzynski. Generalized Decoupled and Object Space Shading System,
     2022.
     https://www.oxidegames.com/wp-content/uploads/2022/07/Generalized-Decoupled-and-Object-Space-Shading-System.pdf
124. Boisse et al. GI-1.0: A Fast and Scalable Two-level Radiance Caching
     Scheme, 2022. https://arxiv.org/abs/2310.19855
125. Wissmann et al. Accelerated Stereo Rendering with Hybrid
     Reprojection-Based Rasterization and Adaptive Ray-Tracing, 2020.
     https://downloads.hci.informatik.uni-wuerzburg.de/2020-ieeevr-stereo-rendering-preprint.pdf
126. Meta. Introducing Stereo Shading Reprojection for Unity, 2017.
     https://developers.meta.com/horizon/blog/introducing-stereo-shading-reprojection-for-unity/
127. Gribble et al. Multi-Hit Ray Traversal, 2014.
     http://jcgt.org/published/0003/01/01
128. Meta. PC Rendering Techniques to Avoid when Developing for Mobile VR.
     https://developers.meta.com/horizon/blog/pc-rendering-techniques-to-avoid-when-developing-for-mobile-vr/
129. Godot PR 113853: skip subpass and tonemap directly on fragment on mobile.
     https://github.com/godotengine/godot/pull/113853
130. Bjorge (Arm). Bandwidth-Efficient Rendering, SIGGRAPH 2015.
     https://community.arm.com/cfs-file/__key/communityserver-blogs-components-weblogfiles/00-00-00-20-66/siggraph2015_2D00_mmg_2D00_marius_2D00_slides.pdf
131. Unity URP. OnTilePostProcessPass.cs and OnTilePostProcessFeature.cs.
     https://github.com/Unity-Technologies/Graphics/tree/master/Packages/com.unity.render-pipelines.universal/Runtime/RendererFeatures
132. Meta. Forward Shading Renderer (Unreal).
     https://developers.meta.com/vr/documentation/unreal/unreal-forward-renderer/
133. Godot issue 113778: glow and foveation on standalone XR.
     https://github.com/godotengine/godot/issues/113778
134. Meta. Application SpaceWarp (Unity).
     https://developers.meta.com/horizon/documentation/unity/unity-asw/
135. Vlachos. Advanced VR Rendering Performance, GDC 2016.
     http://alex.vlachos.com/graphics/Alex_Vlachos_Advanced_VR_Rendering_Performance_GDC2016.pdf
136. Meta. Fixed foveated rendering.
     https://developers.meta.com/horizon/documentation/native/android/os-fixed-foveated-rendering/
137. Godot issue 102780: foveated rendering with fragment density maps, 2025.
     https://github.com/godotengine/godot/issues/102780
138. Godot PR 116220: Vulkan subsampled images with foveated rendering, 2026.
     https://github.com/godotengine/godot/pull/116220
139. Meta. Stereo with foveated inset, 2026.
     https://developers.meta.com/vr/documentation/native/android/os-stereo-with-foveated-inset/
140. Weier et al. Foveated Real-Time Ray Tracing for Head-Mounted Displays,
     2016. https://people.brunel.ac.uk/~csstyyl/papers/cgf2016.pdf
141. Meng et al. Kernel Foveated Rendering, 2018.
     https://duruofei.com/papers/Meng_KernelFoveatedRendering_I3D2018.pdf
142. Koskela et al. Foveated Real-Time Path Tracing in Visual-Polar Space,
     2019.
     https://researchportal.tuni.fi/en/publications/foveated-real-time-path-tracing-in-visual-polar-space
143. Zhang, Gai, Li. Visual Acuity Consistent Foveated Rendering towards
     Retinal Resolution, 2025. https://arxiv.org/abs/2503.23410
144. Meta. Memory and RAM limits.
     https://developers.meta.com/horizon/documentation/unity/po-memory-ram/
145. Careil, Billeter, Eisemann. HashDAG code, 2020.
     https://github.com/Phyronnaz/HashDAG
146. bevy_oxr (bevy_mod_openxr), 2026. https://github.com/awtterpip/bevy_oxr
147. indite: OpenXR with wgpu, 2026. https://github.com/celphase/indite
148. wgpu Adapter::request_device (30.0.1).
     https://docs.rs/wgpu/latest/wgpu/struct.Adapter.html
149. Khronos Vulkan-Samples (Arm). Using async compute to saturate GPU.
     https://github.com/KhronosGroup/Vulkan-Samples/tree/main/samples/performance/async_compute
150. wgpu PR 7080: bounded loops in SPIR-V backend, 2025.
     https://github.com/gfx-rs/wgpu/pull/7080
151. wgpu issue 8801: indirect draws on Quest 3, 2026.
     https://github.com/gfx-rs/wgpu/issues/8801
152. Khronos. VK_QCOM_tile_shading reference page.
     https://docs.vulkan.org/refpages/latest/refpages/source/VK_QCOM_tile_shading.html
153. wgpu PR 10287: Metal ray query semantics and BLAS transforms, 2026.
     https://github.com/gfx-rs/wgpu/pull/10287
154. Vulkan Hardware Database, extension coverage.
     https://vulkan.gpuinfo.org/listextensions.php
155. Khronos Vulkan-Samples (Arm). AFBC sample.
     https://github.com/KhronosGroup/Vulkan-Samples/tree/main/samples/performance/afbc
156. Godot docs. Overview of renderers.
     https://docs.godotengine.org/en/stable/tutorials/rendering/renderers.html
157. Meta. ovrgpuprofiler.
     https://developers.meta.com/horizon/documentation/unity/ts-ovrgpuprofiler/
