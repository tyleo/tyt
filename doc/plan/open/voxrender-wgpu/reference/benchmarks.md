# voxrender-wgpu benchmarks

_Part of the [voxrender-wgpu plan](../README.md)._ Each run gets a section with
its results and its device, OS and driver versions, GPU level, and
temperature. The [research](research.md)'s section 7 sets each benchmark's
threshold.

## B0 Quest 3, 2026-10-18

What the shipping runtime and driver expose. The app draws no scene: one
multiview pass clears both eyes. Each frame submits one projection layer
without depth.

- Device: Meta Quest 3, Horizon OS build 52433670048800520 on Android 14,
  OpenXR runtime Oculus 207.297.0
- GPU: Adreno 740, Qualcomm Vulkan driver build 810c66ceb7 dated 06/24/26,
  Vulkan 1.3.295
- wgpu 30.0.1 on Vulkan, openxr 0.22.0
- Command: the run steps in `voxrender-bench-quest`'s README, worn, at 90 Hz
- GPU level: 4 at 545 MHz, pinned through `debug.oculus.gpuLevel` and
  confirmed by VrApi
- Temperature: 30 to 32 C per VrApi

**Runtime.**

1. View configurations: PRIMARY_STEREO alone. The runtime lists no
   XR_VARJO_quad_views
2. Recommended image rect: 1680x1760 per eye, up to 8192, one sample
3. Motion vectors: 420x440 from XR_FB_space_warp and from each view of
   XR_EXT_frame_synthesis
4. XR_KHR_visibility_mask: listed. Every mask type is empty for both views
   once the session has focus
5. Swapchains: R8G8B8A8_SRGB color, D32_SFLOAT depth and R16G16B16A16_SFLOAT
   motion vectors all create three ways: with attachment usage alone, with
   OpenXR's unordered-access and mutable-format usage, and with Vulkan's
   STORAGE and MUTABLE_FORMAT through XR_META_vulkan_swapchain_create_info
6. Refresh rates: 72, 80, 90 and 120 Hz. The runtime starts at 72 Hz and
   takes the request for 90

**GPU.**

1. Vulkan: shaderInt64 and shaderFloat16 true, shaderSharedInt64Atomics
   false, rayQuery true
2. wgpu: EXPERIMENTAL_RAY_QUERY, MULTIVIEW with up to 6 views, SHADER_INT64,
   SUBGROUP and TIMESTAMP_QUERY_INSIDE_PASSES. No SHADER_F16
3. Indirect draws: `draw_indirect` and `draw_indexed_indirect` each turn 0
   of 4096 texels red, where the direct control turns all 4096. Neither
   dropping wgpu's indirect-call validation nor uploading the arguments
   through `write_buffer` changes the count
4. Indirect dispatch: `dispatch_workgroups_indirect` runs all 6 of its
   workgroups

**GPU frame times.** The app reports XR_META_performance_metrics over each
window of 600 frames after a 300-frame warm-up. Window 0 overlaps the launch
transition. VrApi counts three layers and about 1.56 ms of compositor time
during it. From window 1 on, VrApi counts one layer and no stale frames.

| Window | App p50 | App p99 | Compositor p50 | Compositor p99 |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 0.212 | 0.541 | 0.635 | 1.560 |
| 1 | 0.207 | 0.222 | 0.631 | 0.652 |
| 2 | 0.216 | 0.310 | 0.634 | 0.685 |
| 3 | 0.213 | 0.231 | 0.631 | 0.641 |
| 4 | 0.234 | 0.258 | 0.636 | 0.647 |
| 5 | 0.227 | 0.276 | 0.629 | 0.675 |

Times are in ms. An earlier run with the GPU unpinned held level 2 at
456 MHz. That run's compositor p50 was 0.731 ms, and its app p50 was
0.254 ms.

**Outcomes.**

1. B3 keeps arm (e) because ray query exists
2. The raster arms use direct draws over precomputed index ranges because
   indirect draws render nothing. B3 measures their CPU draw cost. The
   compute arm can use indirect dispatch
3. The compute arm writes the swapchain directly because STORAGE swapchains
   work
4. B9 compares in-app radial density with FDM because quad views are absent
5. Timewarp has no masked tiles to skip because the visibility mask is empty
6. The steady compositor p99 of at most 0.69 ms replaces the 1.3 ms
   placeholder in the research's 3.1. The app budget grows from 8.5 to
   9.1 ms. The contingency line takes the extra 0.6 ms. A depth layer or
   SpaceWarp can add compositor work. B10's full frame includes that work

## B1 desktop, 2026-10-18

The integer walk's ids and corner shadow bits on this Mac, against the
reference. The run times nothing, so it has no GPU level or temperature.

- Device: Apple M4 Max, Metal 4, macOS 27.0.1 (26A434)
- wgpu 30.0.1 on Metal. The adapter reports no driver version
- Command: `cargo run --release -p voxrender-bench-desktop -- b1 --size 256`
- Samples: each pixel's placement, cell, and face. On a pixel whose ids
  match, each light's four corner shadow rays, as whether each meets a
  surface
- Float walk: an f32 DDA over the same quantized inputs and tie order

| Scene | Views | Pixels | Shadow samples | Id mismatches | Shadow mismatches | Float id flips | Float shadow flips |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| cube | 1 | 65536 | 55176 | 0 | 0 | 0 | 0 |
| glass | 1 | 65536 | 128188 | 0 | 0 | 1 | 0 |
| glow | 1 | 65536 | 105484 | 0 | 0 | 0 | 0 |
| l-shape | 1 | 65536 | 89796 | 0 | 0 | 1 | 0 |
| room | 1 | 65536 | 133384 | 0 | 0 | 0 | 0 |
| spot-room | 1 | 65536 | 133384 | 0 | 0 | 0 | 0 |
| two-placements | 1 | 65536 | 57504 | 0 | 0 | 0 | 0 |
| precision views | 10 | 655360 | 0 | 0 | 0 | 19 | 0 |
| precision shadows, sparse | 3 | 196608 | 7031760 | 0 | 0 | 2 | 0 |
| precision shadows, dense | 3 | 196608 | 7186560 | 0 | 0 | 1 | 0 |
| precision shadows, stairs | 3 | 196608 | 6026240 | 0 | 0 | 0 | 0 |

The desktop half passes B1's line. The integer walk matches the reference on
every sample, including the shadow starts that two-placements carries into
the other grid in f32. A run at 64 by 64 pixels also matches everywhere.

The float walk flips 24 of 1,572,864 pixels and no shadow sample. The
research's simulation found f32 against f64 disagreeing on 2.3 to 8.0% of
corner samples. It traced that to input rounding, which the shared
quantization now removes for both walks. Every float flip is on a primary
ray.

**Bench scenes, 2026-10-19.** The research's five scenes joined B1 in S7,
run as above.

| Scene | Views | Pixels | Shadow samples | Id mismatches | Shadow mismatches | Float id flips | Float shadow flips |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| S1 art room | 2 | 131072 | 345556 | 0 | 0 | 6 | 0 |
| S2 city | 2 | 131072 | 376508 | 0 | 0 | 1 | 0 |
| S3 greenhouse | 3 | 196608 | 500392 | 0 | 0 | 10 | 0 |
| S4 foliage | 2 | 131072 | 174176 | 0 | 0 | 8 | 0 |
| S5 grazing floor | 2 | 131072 | 389416 | 0 | 0 | 7 | 0 |

The integer walk matches the reference on every sample of the five. The
earlier scenes' counts did not change.

**Tree walk, 2026-10-20.** S7's tree walk joined B1, run as above over all
sixteen scenes. It matches the reference on every id and shadow sample of
each. The integer and float counts did not change. A check build that lands
each skip one cell too far mismatches 0.4 to 4.3% of the pixels in S1 to S4.
S5 walks cell by cell, because every child its rays reach is a mixed leaf.

## B1 Quest 3, 2026-10-18

The integer walk's ids and corner shadow bits on the headset, against the
reference rendered there, and the GPU times of the walks and the chains.

- Device, GPU, wgpu, and openxr: as in B0
- Command: the run steps in `voxrender-bench-quest`'s README. The ids run
  before the session opens. The times run worn. The walks slow the frames
  to about 30 FPS
- GPU level: 4. VrApi showed 640 MHz on every sample of the first timing
  run. The second run showed 640 MHz on 101 of 117 samples and 285 MHz on
  16. The log did not record which samples fell inside the timed window
- Temperature: 40 C per VrApi
- Samples and float walk: as in B1 desktop

**Ids.**

| Scene | Views | Pixels | Shadow samples | Id mismatches | Shadow mismatches | Float id flips | Float shadow flips |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| cube | 1 | 65536 | 55176 | 0 | 0 | 0 | 0 |
| glass | 1 | 65536 | 128188 | 0 | 0 | 1 | 0 |
| glow | 1 | 65536 | 105484 | 0 | 0 | 0 | 0 |
| l-shape | 1 | 65536 | 89796 | 0 | 0 | 1 | 0 |
| room | 1 | 65536 | 133384 | 0 | 0 | 0 | 0 |
| spot-room | 1 | 65536 | 133384 | 0 | 0 | 0 | 0 |
| two-placements | 1 | 65536 | 57504 | 0 | 0 | 0 | 0 |
| precision views | 10 | 655360 | 0 | 0 | 0 | 10 | 0 |
| precision shadows, sparse | 3 | 196608 | 7031760 | 0 | 0 | 0 | 0 |
| precision shadows, dense | 3 | 196608 | 7186560 | 0 | 0 | 0 | 0 |
| precision shadows, stairs | 3 | 196608 | 6026240 | 0 | 0 | 0 | 0 |

The integer walk matches the reference on every sample, as on this Mac. The
float walk flips 12 pixels and no shadow sample, where this Mac's flipped 24.

**Times.**

- Kernels: the integer and float walks over the far wall scene 8 and 16
  cells deep, and the i32 and f32 chains of 64 links
- Each pass times one kernel over two 1680 by 1760 eyes with GPU
  timestamps. 300 passes warm up, and a window holds 600 per kernel
- Step ratio: the integer walk's growth from 8 to 16 cells deep over the
  float walk's
- This Mac: the device of B1 desktop, through
  `cargo run --release -p voxrender-bench-desktop -- b1-times`

| Kernel | Quest p50 | Quest p99 | Mac p50 | Mac p99 |
| --- | ---: | ---: | ---: | ---: |
| integer walk, 8 deep | 11.797 | 12.959 | 1.737 | 1.928 |
| float walk, 8 deep | 13.979 | 15.984 | 1.820 | 2.041 |
| integer walk, 16 deep | 18.121 | 20.119 | 3.236 | 3.467 |
| float walk, 16 deep | 21.870 | 22.829 | 3.172 | 3.391 |
| i32 chain | 17.246 | 19.203 | 0.651 | 0.824 |
| f32 chain | 16.835 | 18.796 | 0.593 | 0.674 |

Times are in ms, from one window on Quest and the second of three on this
Mac. The integer step costs 0.801x the float step on Quest and 1.108x on
this Mac. The i32 chain costs 1.024x the f32 chain on Quest and 1.098x on
this Mac. This Mac's three windows agree to within 0.5%.

The first Quest run timed the walks before their step rewrite. Its integer
walk took 23.8 and 47.9 ms and its float walk 15.2 and 24.0 ms, a step
ratio of 2.75x. Its three windows agreed to within 1%, and its 256-link
chains ran at 1.03x. The old integer step's cost came from indexing
vectors by a computed axis.

**Outcomes.**

1. B1 passes on Quest. The integer walk's ids and shadow bits match the
   reference everywhere
2. The integer step costs 0.80x a float step on Quest, under the 1.25x
   line. Decision 1 keeps the integer walk on Quest, so Quest's goldens
   need no float tolerance
3. On the Adreno 740, i32 add and compare cost about what f32 does. The
   research's doubt about integer throughput on Adreno does not apply to
   this walk
4. B1's NVIDIA, AMD, and phone runs wait for those devices

## B2 Quest 3, 2026-10-22

The step cost of four memory layouts and the primary ray rates of the tree
walk and hardware ray query, on the headset and on this Mac.

- Device, GPU, wgpu, and openxr: as in B0
- Command: the run steps in `voxrender-bench-quest`'s README. B1's ids run
  before the session opens. The times run worn. The slowest trace holds a
  frame for about 160 ms
- GPU level: 4. The run sampled no clock
- Temperature: 35 C on the SoC sensor right after the window
- This Mac: the device of B1 desktop, through
  `cargo run --release -p voxrender-bench-desktop -- b2-times` and
  `cargo run --release -p voxrender-bench-desktop -- b1 --size 256`

**Ids.** B1 gained the ray query's integer and float marches over 4^3 and
16^3 bricks. A TLAS of placements finds the candidate bricks that each march
walks. The float march carries the world ray into the grid through
the placement's transform. The counts are Quest's.

| Scene | Tree id mismatches | Tree shadow mismatches | Integer 4^3 id mismatches | Integer 16^3 id mismatches | Float 4^3 id flips | Float 16^3 id flips |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cube | 0 | 0 | 0 | 0 | 3 | 3 |
| glass | 0 | 0 | 0 | 0 | 5 | 5 |
| glow | 0 | 0 | 0 | 0 | 5 | 5 |
| l-shape | 0 | 0 | 0 | 0 | 8 | 8 |
| room | 0 | 0 | 0 | 0 | 6 | 6 |
| spot-room | 0 | 0 | 0 | 0 | 6 | 6 |
| two-placements | 0 | 0 | 0 | 0 | 1 | 1 |
| precision views | 0 | 0 | 0 | 0 | 176 | 170 |
| precision shadows, sparse | 0 | 0 | 0 | 0 | 55 | 54 |
| precision shadows, dense | 0 | 0 | 0 | 0 | 55 | 55 |
| precision shadows, stairs | 0 | 0 | 0 | 0 | 38 | 38 |
| S1 art room | 0 | 0 | 0 | 0 | 130 | 130 |
| S2 city | 0 | 1 | 0 | 0 | 312 | 312 |
| S3 greenhouse | 0 | 0 | 0 | 0 | 326 | 326 |
| S4 foliage | 0 | 0 | 0 | 0 | 96 | 96 |
| S5 grazing floor | 0 | 0 | 0 | 0 | 715 | 715 |

The integer marches match the reference on every pixel of every scene. The
float marches flip 0.10% of S1's pixels, 0.24% of S2's, 0.17% of S3's, and
0.55% of S5's. This Mac's float counts fall within 1.2% of Quest's. Its
integer and tree counts match Quest's except on S2.

On Quest, one of S2's shadow samples mismatches for the integer, float, and
tree walks alike. The three walks see corner 3 of pixel (111, 114) in view 1
shadowed, where the reference rendered on the headset sees it lit. This Mac's
walks and reference agree there. The share stays far under B1's line.

**Step loops.** Each pass times one layout at one step count over two 1680
by 1760 eyes. 300 passes warm up, and a window holds 120 per kernel. Times
are in ms, from one window on each device.

| Kernel | Quest p50 | Quest p99 | Mac p50 | Mac p99 |
| --- | ---: | ---: | ---: | ---: |
| storage buffer, 8 steps | 4.616 | 6.861 | 0.589 | 0.726 |
| storage buffer, 16 steps | 11.282 | 12.719 | 1.159 | 1.327 |
| storage buffer, 32 steps | 22.372 | 23.471 | 2.299 | 2.555 |
| 3D texture, 8 steps | 6.760 | 7.036 | 0.571 | 0.574 |
| 3D texture, 16 steps | 11.929 | 12.924 | 1.119 | 1.259 |
| 3D texture, 32 steps | 28.090 | 28.923 | 2.214 | 2.489 |
| 2D array, 8 steps | 5.401 | 8.006 | 0.573 | 0.709 |
| 2D array, 16 steps | 14.217 | 15.885 | 1.121 | 1.281 |
| 2D array, 32 steps | 26.068 | 28.595 | 2.219 | 2.363 |
| u32-pair masks, 8 steps | 9.494 | 9.747 | 0.691 | 0.828 |
| u32-pair masks, 16 steps | 16.208 | 18.819 | 1.361 | 1.634 |
| u32-pair masks, 32 steps | 35.889 | 37.789 | 2.708 | 2.867 |

Each layout's least-squares line through its p50s against the step count:

| Layout | Quest ms per step | Quest Gsteps/s | Mac ms per step | Mac Gsteps/s |
| --- | ---: | ---: | ---: | ---: |
| storage buffer | 0.7332 | 8.07 | 0.0712 | 83.01 |
| 3D texture | 0.9061 | 6.53 | 0.0685 | 86.36 |
| 2D array | 0.8439 | 7.01 | 0.0686 | 86.20 |
| u32-pair masks | 1.1184 | 5.29 | 0.0840 | 70.37 |

Each Quest line's intercept falls between -1.3 and -0.3 ms because 8 steps
cost less per step than 16 or 32.

**Traces.** The tree walk and the four ray query arms trace S1 and S5 without
their lights. Both eyes trace the scene's view. The traces share the step
loops' window.

| Trace | Quest p50 | Quest Grays/s | Mac p50 | Mac Grays/s |
| --- | ---: | ---: | ---: | ---: |
| tree walk, S1 | 161.725 | 0.037 | 11.171 | 0.529 |
| ray query, integer march, 4^3, S1 | 83.260 | 0.071 | 8.888 | 0.665 |
| ray query, integer march, 16^3, S1 | 103.242 | 0.057 | 10.287 | 0.575 |
| ray query, float march, 4^3, S1 | 78.567 | 0.075 | 7.156 | 0.826 |
| ray query, float march, 16^3, S1 | 75.778 | 0.078 | 6.755 | 0.875 |
| tree walk, S5 | 19.469 | 0.304 | 1.288 | 4.592 |
| ray query, integer march, 4^3, S5 | 51.982 | 0.114 | 8.366 | 0.707 |
| ray query, integer march, 16^3, S5 | 33.703 | 0.175 | 4.330 | 1.366 |
| ray query, float march, 4^3, S5 | 53.033 | 0.112 | 7.084 | 0.835 |
| ray query, float march, 16^3, S5 | 38.138 | 0.155 | 3.862 | 1.531 |

The tree walk takes 18.6 loop iterations per ray on S1 and 2.7 on S5. That
is 0.68 and 0.83 Giterations/s on Quest, and 9.83 and 12.49 on this Mac. One
iteration costs about 12 storage-buffer steps on Quest and 8.4 on this Mac.
The Quest p99s sit within 6% of the p50s.

**Outcomes.**

1. R falls under 0.25 Grays/s on S1 and between 0.25 and 0.5 on S5. The best
   S1 rate is 0.078 Grays/s, from the ray query's float march over 16^3
   bricks. The best S5 rate is 0.304, from the tree walk. Under B2's
   threshold, the march serves only glass and shadows
2. At those rates, 3.5 ms marches about 4.6% of the default frame's 5.91 M
   pixels on S1 and 18% on S5
3. R picks B3's arms after S9 tries to raise these rates. At the rates
   above, B3 would build only (d), the greedy mesh, and (c), the hybrid,
   because (a), (b), and (e) march every pixel
4. Risk 2 holds. A storage-buffer step costs about 104 lane-cycles at
   545 MHz over 1536 lanes, where the research's ALU figure allowed 100 to
   170 for a whole hierarchical step
5. Bricks stay in storage buffers. They lead the 2D array by 15% and the 3D
   texture by 24% on Quest. A u32-pair mask step costs 1.5x a
   storage-buffer step
6. On Quest the ray query beats the tree walk by 2.1x on S1 and loses by
   1.7x on S5. Both stay candidates for the shadow march B5 measures

## B2 re-baseline, 2026-10-25

S9.2's profile of where B2's time goes. B2 runs on trusted modules with new
step variants and launch kernels, beside the tree walk with the checks.

- Device: Meta Quest 3, Horizon OS build 52641960028000520 on Android 14,
  OpenXR runtime Oculus 209.147.0. GPU, driver, wgpu, and openxr as in B0
- Command: the run steps in `voxrender-bench-quest`'s README, worn. Two
  windows ran at 07:22 and 07:24 and one at 13:35. The tables hold the 13:35
  window
- GPU level: 4. The 13:35 window ran 89% of its time at 640 MHz, 7% at 599,
  and 4% at 545, a mean of 633 MHz. The morning windows logged 545 to 640 MHz
  without the shares
- Temperature: 33 C per VrApi two minutes after the 13:35 window
- This Mac: the device of B1 desktop, through
  `cargo run --release -p voxrender-bench-desktop -- b2-times`
- Ids: B1 passes on both devices. Quest's counts match B2 2026-10-22's
  table

**Clock.** Level 4 moves between 545, 599, and 640 MHz every few seconds, so
every window blends the three. The three windows' trace medians agree
within 2%. Kernels under 15 ms agree within 10%. B2 2026-10-22 sampled no
clock. Its checked tree walk on S1 took 161.7 ms, against 135.6 ms now, and
that 1.19x falls near the 1.16x between a mean of 633 MHz and level 4's
floor of 545.

**Probes.**

1. Subgroups: an 8x8 dispatch runs as one 64-wide subgroup on Quest and two
   32-wide ones on this Mac
2. Quad views: with an OpenXR 1.1 instance and the manifest tag, the runtime
   offers PRIMARY_STEREO_WITH_FOVEATED_INSET. Its four views each recommend
   840x880, 2.96 M pixels in all, where PRIMARY_STEREO's two recommend
   5.91 M
3. Focal lengths: the left eye spans -54 to 40 degrees across and -55 to 44
   down to up, and the right eye mirrors it. At 1680x1760, f is 758.3 px
   across and 735.2 px up

**Step loops.** Each pass times one loop at one step count over two 1680 by
1760 eyes. Times are p50s in ms. The rate comes from each loop's
least-squares line through its three p50s.

| Loop | Quest 8 | Quest 16 | Quest 32 | Quest Gsteps/s | Mac 8 | Mac 16 | Mac 32 | Mac Gsteps/s |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| storage buffer | 5.176 | 10.138 | 19.599 | 9.86 | 0.254 | 0.470 | 0.901 | 219.25 |
| 3D texture | 4.802 | 10.518 | 20.510 | 9.09 | 0.243 | 0.444 | 0.846 | 235.13 |
| 2D array | 4.802 | 10.506 | 20.373 | 9.18 | 0.250 | 0.461 | 0.879 | 225.88 |
| u32-pair masks | 6.321 | 13.345 | 26.865 | 6.92 | 0.359 | 0.688 | 1.355 | 142.46 |
| buffer, start hoisted | 3.201 | 5.789 | 10.089 | 20.80 | 0.232 | 0.428 | 0.813 | 244.72 |
| buffer, two chains | 2.877 | 5.390 | 11.889 | 31.13 | 0.281 | 0.534 | 1.024 | 382.86 |
| buffer, vec4 loads | 3.108 | 6.598 | 13.752 | 13.32 | 0.258 | 0.478 | 0.916 | 215.54 |
| buffer, uniform address | 2.592 | 4.729 | 10.622 | 17.42 | 0.234 | 0.427 | 0.813 | 245.07 |

The trusted storage-buffer step runs 1.22x B2 2026-10-22's on Quest and
2.6x on this Mac. Hoisting the start doubles the Quest rate again. Two chains
run 1.50x one chain on Quest and 1.56x on this Mac. On Quest, vec4 loads cost
0.64x the hoisted rate and a uniform address 0.84x.

**Launches.** The compute launch runs the hoisted loop at 0 to 8 steps. The
fragment launch draws both eyes with one triangle each. Times are p50s in
ms.

| Launch | Quest | Mac |
| --- | ---: | ---: |
| compute, 0 steps | 2.318 | 0.106 |
| compute, 1 step | 2.319 | 0.159 |
| compute, 2 steps | 2.321 | 0.171 |
| compute, 4 steps | 2.323 | 0.172 |
| compute, 8 steps | 2.793 | 0.232 |
| fragment | 0.228 | 0.056 |

On Quest, a compute pass over both eyes takes at least 2.32 ms, 10x the
fragment pass over the same pixels. Up to 4 steps hide under that floor. A
pass takes the floor or its work, whichever is longer. The step loops' lines
start at 8 steps, above the floor, so their intercepts hold no launch cost.

The hoisted loop at 8 steps takes 3.201 ms in the step loops' rows and
2.793 ms as the compute launch, the same pipeline in the same window. The
morning windows show the same gap. The two differ only in the pass before
them: the 32-step masks loop, and the 4-step launch. On this Mac both take
0.232 ms. The pass before can move a short Quest kernel's time by up to
15%.

**Traces.** The tree walk, with and without the checks, and the four ray
query arms trace S1 and S5 without their lights.

| Trace | Quest p50 | Quest Grays/s | Mac p50 | Mac Grays/s |
| --- | ---: | ---: | ---: | ---: |
| tree walk, S1 | 108.036 | 0.055 | 8.505 | 0.695 |
| tree walk, checked, S1 | 135.578 | 0.044 | 11.180 | 0.529 |
| ray query, integer march, 4^3, S1 | 67.187 | 0.088 | 7.742 | 0.764 |
| ray query, integer march, 16^3, S1 | 75.620 | 0.078 | 8.615 | 0.686 |
| ray query, float march, 4^3, S1 | 44.752 | 0.132 | 6.216 | 0.951 |
| ray query, float march, 16^3, S1 | 42.175 | 0.140 | 5.700 | 1.037 |
| tree walk, S5 | 13.392 | 0.442 | 1.064 | 5.559 |
| tree walk, checked, S5 | 16.949 | 0.349 | 1.290 | 4.583 |
| ray query, integer march, 4^3, S5 | 42.555 | 0.139 | 7.320 | 0.808 |
| ray query, integer march, 16^3, S5 | 27.298 | 0.217 | 3.730 | 1.586 |
| ray query, float march, 4^3, S5 | 30.350 | 0.195 | 6.187 | 0.956 |
| ray query, float march, 16^3, S5 | 21.691 | 0.273 | 3.282 | 1.802 |

The checks cost the tree walk 1.25x on S1 and 1.27x on S5 on Quest, and
1.31x and 1.21x on this Mac.

**Iterations.** A counted walk splits each ray's loop iterations by the side
of the region each one leaves. The counts match on both devices.

| Scene | Iterations per ray | Cell steps | 4^3 skips | 16^3 skips | Quest Giterations/s | Mac Giterations/s |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| S1 | 18.6 | 3.8 | 5.2 | 9.5 | 1.02 | 12.91 |
| S5 | 2.7 | 2.7 | 0.0 | 0.0 | 1.20 | 15.12 |

On S1, 79% of the iterations skip a region. One S1 iteration costs about 20
hoisted steps on Quest and 19 on this Mac.

**Shader lines.** One Xcode Metal capture of the S1 tree walk on this Mac,
on 2026-10-24, costs each line of the generated Metal. The walk compiles to
80 registers. Shares are of the whole shader.

| Code | Share |
| --- | ---: |
| `tree_walk` | 88.75% |
| `leave()`, crossing a region | 29.74% |
| `leave()`'s f32 guess, `(high * ONE - origin) / direction` | 6.35% |
| `leave()`'s crossing on another axis | 5.29% |
| `leave()`'s floor and clamp of that crossing | 3.49% |
| `leave()`'s correction loops | about 2% |
| `enter()`, no line over 3.03% | 22.27% |
| `at % size`, an i32 modulo by a runtime 4 or 16 | 13.11% |
| `region_at` | 6.54% |
| the single-cell step | about 0.8% |

**Counters.** `ovrgpuprofiler -r` sampled each profiled kernel for 7 seconds
on Quest at 640 MHz, on 2026-10-25. The profile runs each kernel alone.

| Counter | Tree walk, S1 | Tree walk, S5 | Ray query, float, 16^3, S1 | Hoisted, 16 steps | Compute launch | Fragment launch |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| % Shaders Busy | 94.9 | 93.2 | 94.3 | 93.1 | 27.6 | 5.6 |
| % Shaders Stalled | 8.6 | 15.8 | 4.5 | 6.0 | 22.3 | 0.2 |
| % Shader ALU Capacity Utilized | 56.4 | 39.8 | 51.6 | 57.7 | 11.1 | 68.4 |
| % Time ALUs Working | 59.7 | 44.5 | 64.3 | 59.9 | 11.5 | 71.0 |
| % Time EFUs Working | 6.3 | 1.9 | 7.8 | 1.2 | 2.5 | 15.2 |
| % Wave Context Occupancy | 51.6 | 51.9 | 51.7 | 98.8 | 20.5 | 95.0 |
| % L1 Hit | 89.4 | 81.5 | 91.5 | 85.9 | 62.4 | 62.5 |
| % Global Buffer Read L2 Hit | 78.4 | 80.2 | 78.6 | 75.5 | 75.1 | 75.0 |
| % Stalled on System Memory | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 |
| % RTU Busy | 0.0 | 0.0 | 31.2 | 0.0 | 0.0 | 0.0 |
| % BVH Fetch Stall | 0.0 | 0.0 | 0.5 | 0.0 | 0.0 | 0.0 |
| % Time Compute | 95.4 | 94.8 | 95.3 | 95.0 | 30.5 | 0.0 |
| % Kernel Load Cycles | 5.1 | 6.4 | 5.6 | 6.6 | 70.4 | 92.1 |

The compute launch spends 67% of its time shading fragments, so the eye
clear and the compositor fill its column. The launch columns do not isolate
the launches.

**Registers.** On 2026-10-25, RenderDoc Meta Fork v207.0 captured one frame
of each kernel while the profile ran on Quest. Its
KHR_pipeline_executable_properties view shows the driver's statistics. A
register holds one vec4. A scratch slot holds 16 bytes.

| Kernel | Registers | Scratch slots | ALU fiber occupancy | Instructions |
| --- | ---: | ---: | ---: | ---: |
| tree walk | 20 | 0 | 50% | 4563 |
| ray query, float march, 16^3 | 23 | 168 | 50% | 1912 |
| buffer, start hoisted, 16 steps | 3 | 0 | 100% | 168 |

The driver flags any scratch use as slow.

**Outcomes.**

1. R stays under 0.25 Grays/s on S1 and between 0.25 and 0.5 on S5. The best
   S1 rate is 0.140, from the ray query's float march over 16^3 bricks. The
   best S5 rate is 0.442, from the tree walk
2. Trusted modules gain 1.25x on the tree walk. The rest of the gain over
   B2 2026-10-22 falls within the clock's 1.16x
3. The walk computes more than it waits. Its ALUs work 60% of the time and
   stall 8.6%, its L1 hits 89%, and system memory stalls stay at 0.1%. Two
   chains gain 1.5x, short of the 2x that would mark a latency-bound loop.
   Rung 3 stays ahead of rung 4
4. The walk holds 20 registers and no scratch. Rung 1 moves rung 5's trims
   up only past 24 registers or a spill, so the trims stay at rung 5. The
   counters' 52% matches the driver's cap of 50% occupancy at 20 registers.
   The step loop holds 3 registers and runs at 100% occupancy
5. Region skips dominate. `leave()`, `enter()`, and `at % size` take 65% of
   the S1 walk, and the single-cell step under 1%. `at % size` divides by a
   power of two, which a mask could replace
6. No compute pass over both eyes runs under 2.32 ms on Quest, 21% of a
   90 Hz frame. A fragment pass over the same pixels takes 0.23 ms
7. Quad views cut the recommended pixels to 2.96 M, half of stereo's
8. Wave64 on Quest drops rung 5's 16x8 groups, which waited on wave128
9. Each invocation of the ray query march uses 2.7 KB of scratch

## S9.3 raster, 2026-10-25

A bare greedy-mesh raster of S1 and S5 gives the march a rate to beat. B2
times it beside the traces, and B1 counts its id flips.

- Device: Meta Quest 3 as in B2 re-baseline, run off the face through the
  steps in `voxrender-bench-quest`'s README. Seven windows ran from 19:06 to
  19:22. The tables hold the last
- GPU level: 4. The window ran 83% of its time at 640 MHz, 12% at 599, and
  5% at 545
- This Mac: the device of B1 desktop, through
  `cargo run --release -p voxrender-bench-desktop -- b2-times`

**Raster.** voxsurface meshes each object once, merging quads by occupancy
alone, and each placement draws its object. Depth runs reversed in f32. Each
fragment finds the cell under the hit point and writes the hit as the
march's ids, packed into an `Rgba8Unorm` texel, the 4 bytes of a shaded
pixel. The march writes 16. S1 draws 1988 quads over 8 placements, and S5
draws 803,082. Three arms run:

1. Two passes, one per eye
2. One multiview pass
3. One multiview pass with 4x MSAA, resolved

**Times.** Both eyes draw each scene's view at 1680 by 1760. A pixel counts
as a ray. Times are p50s in ms.

| Kernel | Quest p50 | Quest Grays/s | Mac p50 | Mac Grays/s |
| --- | ---: | ---: | ---: | ---: |
| tree walk, S1 | 106.517 | 0.056 | 8.981 | 0.658 |
| ray query, float march, 16^3, S1 | 42.183 | 0.140 | 6.034 | 0.980 |
| raster, two passes, S1 | 1.073 | 5.512 | 0.058 | 101.158 |
| raster, multiview, S1 | 1.112 | 5.319 | 0.061 | 97.343 |
| raster, multiview, 4x MSAA, S1 | 1.129 | 5.240 | 0.077 | 77.218 |
| tree walk, S5 | 13.601 | 0.435 | 1.130 | 5.235 |
| raster, two passes, S5 | 9.664 | 0.612 | 0.756 | 7.822 |
| raster, multiview, S5 | 8.928 | 0.662 | 0.955 | 6.192 |
| raster, multiview, 4x MSAA, S5 | 11.036 | 0.536 | 1.194 | 4.951 |

The seven windows' raster p50s agree within 7%, apart from the fifth, which
ran up to 1.16x slower.

**Ids.** B1 passes on both devices. The raster's flips count for the record.
On Quest at 256 by 256, they reach 0.12% of S1's pixels, 0.32% of S2's, and
0.68% of S5's. The reference walks quantized rays and the raster samples the
exact projection, so a pixel near a cell edge can land on either side. A few
pixels fall through cracks where a greedy quad's edge meets another's
corner. S3's hits need 34 bits, so B1 leaves it out.

**Outcomes.**

1. On S1 the raster runs 99x the tree walk and 39x the best march. On S5 it
   runs 1.5x the tree walk
2. S5's raster takes 8.9 ms, 80% of a 90 Hz frame, for 1.6 M triangles an
   eye. S1's raster takes 1.07 ms for 4000 triangles an eye, 4.7x the
   fragment launch
3. Multiview saves 8% on S5 and costs 4% on S1 on Quest. On this Mac it
   costs 1.26x on S5
4. 4x MSAA costs 1.24x on S5 and 1.02x on S1 on Quest, against the
   multiview pass
5. The march is behind on both scenes. The gap is smallest on S5, whose
   grazing floor holds 0.8 M quads

## S9.4 rung 2, 2026-10-27

Rung 2 makes three changes in turn and times each on this Mac through
`b2-times`. Each change stays only when it wins. B1 then gives the same ids
and shadows as before rung 2 on every scene. A Quest run confirms the batch.

- This Mac: the device of B1 desktop. Spotlight and media analysis loaded
  it through most runs and moved its times by up to 2.6x. Each comparison
  alternates its builds, three windows a run, and compares builds within a
  round
- Quest: the device of B2 re-baseline at GPU level 4, run off the face on
  2026-10-31. Each window ran 75% to 81% of its time at 640 MHz
- Times are each build's median p50 in ms

**Masks.** Each region's cube aligns to its power-of-two size. In the tree
walk and the float march, `& ~(size - 1)` replaces the runtime `% size`.

| Kernel | Before | Masks | Gain |
| --- | ---: | ---: | ---: |
| tree walk, S1 | 9.072 | 8.437 | 1.08x |
| tree walk, S5 | 1.133 | 1.136 | 1.00x |
| ray query, float march, 16^3, S1 | 6.071 | 5.631 | 1.08x |
| ray query, float march, 16^3, S5 | 3.498 | 3.304 | 1.06x |

**f32 jumps.** A region larger than a cell ends in one f32 jump in place of
`leave()`'s guess and correction loops. The jump takes each axis's exit time
from the exact integer distance to its face times the ray's reciprocal. It
then floors the ray's point at the soonest exit, clamped into the region. A
single cell keeps the integer step. Four variants ran:

| Variant | S1 gain | S5 gain | B1 |
| --- | ---: | ---: | --- |
| Every step f32, exits from per-ray reciprocals | 1.41x | 0.93x | fails: 0.11% to 0.24% of the precision scenes' shadow samples flip |
| Every step f32, exits from exact integer distances | 1.38x | 0.91x | passes: up to 0.004% of a scene's ids flip |
| Integer cell steps, f32 jumps set up once per ray | 1.39x | 0.97x | exact |
| Integer cell steps, f32 jumps set up in the jump | 1.37x | 1.00x | exact |

The last variant stays. It also gains 1.16x on the integer ray-query march's
S1 and 1.01x on its S5, since that march runs the tree walk.

**Placement cull.** Three parts ran in turn, against the f32 jumps:

1. Each walk ends one cell past the nearest hit so far: 0.96x on S1. S1
   lists its room first, so the room's walls set the nearest hit and no
   walk ends sooner
2. The walks also run last placement first: 1.02x on S1, 0.97x on S5. S1's
   iterations per ray fall from 18.6 to 16.7
3. An f32 slab test against the grid padded by a cell also skips the
   integer entry of a ray that misses: 1.30x to 1.35x on S1, 0.94x to 1.02x
   on S5. In forward order, the slab test gains 1.17x on S1

All three stay.

**Rung 2.** Three alternating runs of the build before rung 2 against the
build after it:

| Kernel | Gain per run |
| --- | ---: |
| tree walk, S1 | 1.94x, 2.06x, 1.91x |
| tree walk, S5 | 0.88x, 1.03x, 0.90x |
| ray query, integer march, 16^3, S1 | 1.17x, 1.26x, 1.16x |
| ray query, integer march, 16^3, S5 | 0.99x, 1.01x, 0.97x |
| ray query, float march, 16^3, S1 | 1.05x, 1.10x, 1.03x |
| ray query, float march, 16^3, S5 | 1.02x, 1.10x, 1.02x |

**Quest.** The build after rung 2 ran one window, then the build before it
ran three, then the build after it ran three more. Each kernel's p50s agree
within 1% across a build's windows. The table compares the three-window
runs:

| Kernel | Before | After | Gain |
| --- | ---: | ---: | ---: |
| tree walk, S1 | 107.187 | 80.683 | 1.33x |
| tree walk, checked, S1 | 135.617 | 71.912 | 1.89x |
| tree walk, S5 | 13.715 | 16.456 | 0.83x |
| tree walk, checked, S5 | 17.175 | 17.585 | 0.98x |
| ray query, integer march, 16^3, S1 | 75.684 | 58.694 | 1.29x |
| ray query, integer march, 16^3, S5 | 28.250 | 28.237 | 1.00x |
| ray query, float march, 16^3, S1 | 42.034 | 37.331 | 1.13x |
| ray query, float march, 16^3, S5 | 22.245 | 20.519 | 1.08x |
| raster, two passes, S5 | 10.155 | 13.662 | 0.74x |
| raster, multiview, S5 | 9.235 | 11.708 | 0.79x |
| raster, multiview, 4x MSAA, S5 | 11.036 | 11.201 | 0.99x |

S1's iterations per ray fall from 18.6 to 16.7, as on this Mac. Rung 2
leaves the raster's code alone. Yet in both runs after rung 2, the S5
raster's two-pass and multiview arms run slower. In rung 3, a third run of
the same build ran those arms 1.16x and 1.09x faster than the second. The S1
raster stays within 8%.

**Outcomes.**

1. The tree walk runs about 2x on this Mac's S1. The research expected 3.3x
   on S1 and 1.5x on S5 from the f32 walk and the cull
2. S5 runs 0.88x to 1.03x on this Mac. Its rays leave 2.7 regions, all
   single cells, so the f32 jumps never run there. The slab test and the end
   add their cost to each of its walks
3. On both devices, the tree walk gives the reference's ids and shadows on
   every B1 scene. The research's switch to the exact per-level exit does
   not fire
4. Quest confirms the batch on S1: 1.33x for the tree walk and 1.29x for
   the integer march, below the Mac's 2x. S5 loses more on Quest, at 0.83x
5. With its checks, the tree walk runs 1.89x on Quest's S1 and outruns the
   walk without them. Rung 4's footprint reads can show how the checks
   change the compiled walk
6. The S5 raster's two-pass and multiview arms move by up to 1.16x between
   runs of one build. The S9.3 bar holds

## S9.4 rung 3, 2026-10-31

Rung 3 adds boxes over the empty space the walk crosses. As in rung 2, this
Mac times each change against the build before it in three alternating rounds
of three windows. B1 gives the same tables as before rung 3.

**Tile boxes.** Each empty tile in the root stores the box of empty tiles
around it, up to 15 tiles a side. The f32 jump crosses the box in one step.

| Kernel | Before | Tile boxes | Gain per round |
| --- | ---: | ---: | ---: |
| tree walk, S1 | 4.181 | 3.218 | 1.30x, 1.30x, 1.26x |
| tree walk, checked, S1 | 4.644 | 3.585 | 1.30x, 1.30x, 1.23x |
| tree walk, S5 | 1.048 | 1.085 | 0.97x, 0.97x, 0.92x |
| ray query, integer march, 16^3, S1 | 7.010 | 7.312 | 0.96x, 0.97x, 0.92x |
| ray query, integer march, 16^3, S5 | 3.681 | 3.776 | 0.97x, 0.99x, 0.94x |
| ray query, float march, 16^3, S1 | 5.221 | 5.340 | 0.98x, 0.99x, 0.94x |
| ray query, float march, 16^3, S5 | 3.069 | 3.114 | 0.99x, 0.99x, 0.94x |

S1's iterations per ray fall from 16.7 to 8.6. Its 16^3 skips fall from 9.2
to 1.1, and its 3.1 cell steps and 4.4 4^3 skips stay. The integer march
cuts each box down to its brick. The float march leaves the boxes out. Both
marches pay for the box check in each region lookup.

**Quest.** Tile boxes ran three windows on Quest, off the face at GPU level
4 as in rung 2. The build before them then ran three more. Those runs land
within 1% of rung 2's on every walk.

| Kernel | Before | Tile boxes | Gain |
| --- | ---: | ---: | ---: |
| tree walk, S1 | 80.297 | 72.110 | 1.11x |
| tree walk, checked, S1 | 71.890 | 56.002 | 1.28x |
| tree walk, S5 | 16.413 | 16.026 | 1.02x |
| tree walk, checked, S5 | 17.345 | 14.308 | 1.21x |
| ray query, integer march, 16^3, S1 | 58.945 | 60.775 | 0.97x |
| ray query, integer march, 16^3, S5 | 28.103 | 28.998 | 0.97x |
| ray query, float march, 16^3, S1 | 37.306 | 38.168 | 0.98x |
| ray query, float march, 16^3, S5 | 20.295 | 21.147 | 0.96x |

The iteration counts match this Mac's. B1 passes on Quest.

**Child boxes.** The walk grows a box around an empty child from the node's
mask words. It adds a layer at a time along each axis the ray moves on. Two
variants ran against tile boxes:

| Variant | S1 iterations | S1 gain per round | S5 gain per round |
| --- | ---: | ---: | ---: |
| Up to 3 rounds of growth | 5.6 | 0.84x, 0.84x, 0.84x | 0.86x, 0.86x, 0.86x |
| 1 round of growth | 6.2 | 0.97x, 0.90x, 0.90x | 0.92x, 0.86x, 0.86x |

The float march leaves the growth off, yet it ran 0.92x with the 3-round
variant.

**Outcomes.**

1. Tile boxes halve S1's iterations. The research's model predicted 18.2
   to 9.75. The walk gains 1.30x in time, under the research's 1.6x to 1.9x
   and over the stop line at half the 1.94x iteration ratio. Tile boxes
   stay
2. Child boxes cut S1's 4^3 skips from 4.4 to 1.4 and lose on every
   kernel. S5's walks rarely grow a box and still lose 14%, so the growth
   costs the walk even where it does not run. The stop rule fires, and
   child boxes drop. The research profiles before adding more boxes
3. 2^3 coalescing needs to know which cells of a leaf are empty without
   reading their indices. It waits for rung 4's leaf occupancy masks
4. Quest gains 1.11x on S1 from half the iterations, so each iteration
   costs 1.73x as much there. Iterations no longer set the walk's time on
   Quest
5. The walk with the checks gains more on Quest: 1.28x on S1 and 1.21x on
   S5. It runs S1 in 56.0 ms against the plain walk's 72.1

## S9.4 raster then march, 2026-11-11

The first of S9.4's new approaches: a raster of boxes with a march in each
fragment. B1 checks its ids, and B2 times it beside the tree walk and S9.3's
raster.

- Device: Meta Quest 3 as in B2 re-baseline, run off the face at GPU level
  4. Each table gives its window's clock
- This Mac: the device of B1 desktop, through
  `cargo run --release -p voxrender-bench-desktop -- b2-times`
- Times are p10s in ms. The Mac's are the median of three windows

**Timer.** Two copies of one kernel read 3.2 and 4.0 ms at p50 in one Quest
window. A pass runs about 0.75 ms longer when the compositor interrupts it,
and which passes it interrupts follows the kernels around them. Two changes
answer it:

1. Each kernel takes turns of 20 passes in a row, and the first 4 of a turn
   go untimed
2. A window reports each kernel's least and p10 times beside its p50 and
   p99. The copies agree within 1% at p10

The sections before this one hold p50s from single-pass turns.

**Box raster.** Each tile of each object gives the box around its live
cells. One instanced draw puts each placement's boxes on screen, and each
fragment marches its pixel's ray through its box. The fragment writes its
hit as S9.3's raster packs it. Five arms run:

1. The ray query's integer march over the 64-tree along the pixel's
   quantized ray. Back faces draw, and each fragment writes its hit's depth
2. The ray query's float march, drawn the same way
3. A mask march, hit depth. Each tile holds a bit per cell. A fragment
   starts at the point the rasterizer gives it on its box and steps a cell
   at a time through the bits. It writes its hit's depth
4. The mask march, face depth. A fragment discards a miss and keeps its
   face's depth. The boxes draw front faces, nearest box first. A box that
   crosses another placement's box draws as arm 3 does
5. Arm 4 with solid boxes. A tile first gives up to four slabs of live
   cells alone, which draw as plain faces with no march

**Ids.** B1 passes on both devices. The integer march gives the reference's
ids on every scene, so the boxes' coverage and depth add no mismatch. The
other arms' flips count for the record. On Quest at 256 by 256:

| Scene | Raster | Float march | Mask, hit depth | Mask, face depth | Solid and mask |
| --- | ---: | ---: | ---: | ---: | ---: |
| precision views | 0.043% | 0.026% | 0.027% | 0.026% | 0.026% |
| S1 art room | 0.120% | 0.100% | 0.122% | 0.150% | 0.153% |
| S2 city | 0.320% | 0.238% | 0.315% | 0.315% | 0.315% |
| S4 foliage | 0.114% | 0.073% | 0.107% | 0.111% | 0.111% |
| S5 grazing floor | 0.680% | 0.546% | 0.590% | 0.631% | 0.631% |

**Depth probe.** The first box rasters drew front faces under a declared
depth bound, `@early_depth_test(less_equal)`, and gained nothing. A probe
then drew four costly layers over both eyes, each behind the last, in the
open and behind a near layer. Quest at 545 MHz:

| A costly fragment | Open | Behind a near layer |
| --- | ---: | ---: |
| writes color alone | 1.615 | 0.519 |
| can discard | 1.667 | 0.520 |
| writes depth | 7.079 | 7.445 |
| writes depth under the bound | 6.951 | 7.430 |

On Quest a fragment that writes depth always runs, with or without the
bound. A fragment that can discard is rejected like a plain one. On this
Mac a discarding layer hides behind a near layer and behind no layer of its
own kind. A depth-writing layer never hides.

**Times.** One Quest window ran every trace at 640 MHz for 87% of its time,
599 for 3%, and 545 for 10%.

| Kernel | Quest | Quest Grays/s | Mac |
| --- | ---: | ---: | ---: |
| tree walk, S1 | 61.715 | 0.096 | 3.215 |
| ray query, float march, 16^3, S1 | 37.685 | 0.157 | 5.331 |
| raster, two passes, S1 | 0.916 | 6.456 | 0.053 |
| box raster, integer march, S1 | 19.373 | 0.305 | 1.776 |
| box raster, float march, S1 | 10.661 | 0.555 | 1.011 |
| box raster, mask march, hit depth, S1 | 3.266 | 1.811 | 0.418 |
| box raster, mask march, face depth, S1 | 2.538 | 2.330 | 0.366 |
| box raster, solid boxes and mask march, face depth, S1 | 1.372 | 4.310 | 0.195 |
| tree walk, S5 | 15.477 | 0.382 | 1.072 |
| ray query, float march, 16^3, S5 | 19.343 | 0.306 | 3.096 |
| raster, two passes, S5 | 7.870 | 0.751 | 0.698 |
| box raster, integer march, S5 | 27.344 | 0.216 | 2.446 |
| box raster, float march, S5 | 20.944 | 0.282 | 2.042 |
| box raster, mask march, hit depth, S5 | 3.395 | 1.742 | 0.498 |
| box raster, mask march, face depth, S5 | 3.125 | 1.892 | 0.457 |
| box raster, solid boxes and mask march, face depth, S5 | 3.183 | 1.858 | 0.456 |
| raster, two passes, S2 | 136.953 | 0.043 | 14.335 |
| box raster, integer march, S2 | 146.909 | 0.040 | 12.371 |
| box raster, float march, S2 | 101.096 | 0.058 | 8.946 |
| box raster, mask march, hit depth, S2 | 23.293 | 0.254 | 2.967 |
| box raster, mask march, face depth, S2 | 12.302 | 0.481 | 2.051 |
| box raster, solid boxes and mask march, face depth, S2 | 14.317 | 0.413 | 1.985 |

S2's 1025 placements take the walks and the ray queries seconds a pass, so
only the rasters trace it.

Two more windows ran the face-depth arms alone, at 545 MHz for 98% and
100% of their time. Their p10s agree within 1%:

| Kernel | S1 | S5 | S2 |
| --- | ---: | ---: | ---: |
| box raster, mask march, face depth | 2.539 | 3.161 | 14.491 |
| box raster, solid boxes and mask march, face depth | 1.373 | 3.175 | 14.198 |

**Boxes.** How each scene's view splits its boxes:

| Scene | Tiles | Solid boxes | Marched boxes | Marched boxes that cross |
| --- | ---: | ---: | ---: | ---: |
| S1 | 388 | 381 | 45 | 13 |
| S5 | 4096 | 0 | 4096 | 0 |
| S2 | 25,301 | 3697 | 21,604 | 6957 |

A slab stays in the march when the cells left would cover over half of the
face it shows them. Without that rule S5's floor gave a slab under every
tile, and S5 ran 4.8 ms against 4.0 for the mask march alone, in p50s from
single-pass turns.

**Counts.** A counting build of arm 3 on this Mac, at 512 by 512, gives the
work per pixel when every box a ray crosses marches:

| Scene | Fragments | Cell steps | Hits |
| --- | ---: | ---: | ---: |
| S1 | 1.32 | 3.90 | 1.26 |
| S5 | 0.92 | 3.14 | 0.81 |
| S2 | 6.47 | 30.43 | 5.27 |

About a third of S5's fragments hit behind a nearer hit, and about two
thirds of S2's do. `ovrgpuprofiler` sampled the face-depth arms on Quest,
one pass a frame at 90 Hz. Its counters cover every fragment the GPU
shades. With no kernel fragments they show 16 M fragments a frame at 28
instructions each:

| Kernel | Instructions per fragment | Wave context occupancy |
| --- | ---: | ---: |
| mask march, S1 | 112 | 76% |
| solid boxes and mask march, S1 | 74 | 88% |
| mask march, S5 | 140 | 65% |
| mask march, S2 | 216 | 62% |

**Crossing boxes.** A build that keeps every box under face depth, right
or wrong, ran S2's mask march in 4.35 ms at 545 MHz against 14.49. The
boxes that cross cost S2 about 10 ms. The tiles share real space: S2's
tree crowns reach 1.9 m from trunks that stand 0.95 m inside a block's
corner, and its buildings start 0.55 to 1.95 m inside it.

**Variants that lost.**

1. Front faces under the declared depth bound, in p50s at 640 MHz from
   single-pass turns: the integer march ran S1 in 19.3 ms against 19.7 for
   back faces, the float march in 12.3 against 10.6, and the mask march in
   3.3 against 3.4. S5 ran 28.0 against 28.0, 24.4 against 21.0, and 4.9
   against 4.9
2. Boxes around 8^3 and 4^3 bricks ran S5 in 5.98 and 10.61 ms against
   3.20 for tiles, at 545 MHz
3. Sixteen distance bands, each drawing its solid boxes and then its
   marched ones, ran S5 in 3.92 ms against 3.16 for one draw, and S1 in
   1.32 against 1.37. The 15 more draws an eye cost S5 about 25 us each
4. A crossing tile's box gave way to the boxes of its 4^3 bricks, and only
   the bricks that still crossed kept hit depth. Of S2's 226,647 such
   boxes, 80,718 still crossed

**Outcomes.**

1. The solid and mask arm traces S1 in 1.37 ms and S5 in 3.18 at 545 MHz,
   both under B3's 3.5 ms. The times hold ids alone, without B3's shading
   load. The arm runs 45x the tree walk on S1 and 4.9x on S5
2. Against S9.3's raster the arm runs 0.67x on S1 and 2.5x on S5. On S2 it
   runs 9.6x and still takes 14.3 ms
3. Quest rejects a fragment before it runs only when the fragment writes
   no depth. Face depth follows from that. It gains 1.29x on S1, 1.09x on
   S5, and 1.89x on S2 over hit depth
4. A third of S2's marched boxes cross another placement's box and keep
   hit depth, which costs S2 about 10 of its 14.5 ms. S2 also runs with no
   LOD
5. Solid boxes take about 95% of S1's marched fragments and gain 1.85x
   there. S5 has none
6. The mask march's flips stay near the greedy-mesh raster's: 0.15%
   against 0.12% on S1, and 0.63% against 0.68% on S5
7. A pass's p50 moves by up to 24% with the compositor. A p10 compares two
   kernels within 1%
8. A full-screen draw costs about 25 us on Quest, so the boxes stay in few
   draws
9. The mask march runs at 62% to 76% occupancy and 112 to 216 instructions
   a fragment. Rung 4's register work now applies to this shader

## Headset view, 2026-11-11

The box raster as a live view. The Quest app draws a bench scene around the
wearer each frame, in place of its benchmarks.

- Device: Meta Quest 3 as in B2 re-baseline, off the face on a desk with
  tracking on, at 90 Hz and GPU level 4
- Command: the View steps in `voxrender-bench-quest`'s README
- Each frame sorts the scene's boxes nearest first on the CPU and draws
  both eyes in one multiview pass, from each eye's tracked pose and
  frustum. The pass runs the solid boxes and the mask march under face
  depth
- A hit takes its cell's base color, the sun without shadows, and the
  hemisphere, under the contract's tonemap. B3's shading load asks for
  more
- GPU times are the app's frame time from XR_META_performance_metrics over
  windows of 600 frames, in ms. The CPU time is the sort and the uploads

| Scene | Frame rate | GPU p50 | GPU p99 | CPU | GPU clock |
| --- | ---: | ---: | ---: | ---: | ---: |
| S1 | 90 Hz | 1.71 to 1.92 | 2.09 | 0.25 | 545 MHz |
| S5 | 90 Hz | 3.69 to 3.74 | 3.84 | 1.48 | 545 MHz |
| S2 | 71 Hz | 12.57 to 12.60 | 13.58 | 6.29 | 640 MHz |

VrApi reports no stale frames on S1 and S5, with the GPU busy 18% and 43%
of the time. On S2 it reports 22 stale frames a second and the GPU busy
97%.

**Outcomes.**

1. S1 and S5 hold 90 Hz in the headset with both eyes' own views. S1 leaves
   82% of the GPU idle, and S5 57%
2. The shaded S5 frame takes 3.7 ms against 3.2 for B2's ids. The view
   adds each eye's frustum, a material read, and the shading
3. S2 misses 90 Hz on both the GPU and the CPU. Its 25,301 boxes take
   6.3 ms a frame to sort and upload

## Box raster open ends, 2026-11-11

The open ends the raster-then-march round left, in the checklist's order.

- Device, commands, and GPU times: as in the two sections above
- CPU times are a frame's `BoxView::prepare` on Quest in ms, averaged over
  600 frames

**A box that holds an eye.** No bench view stood inside a box with live
cells, so B1 gained one: S4's garden from an empty cell inside a bush.
Under face depth the view showed a farther leaf on 38.7% of its pixels on
this Mac, at 64 by 64. The box that holds the eye draws its back faces.
The box behind it draws a front face on the same plane, and the two tied
in depth. Back faces now draw first and a part in 1024 nearer. The view
then flips no pixel on this Mac and 1 of 65,536 on Quest, and B1 passes on
both. The headset view also dropped a solid box within 2 cm of an eye. It
now draws every solid box.

**Frame CPU.** The view sorted and uploaded every box each frame. S2's
2.2 ms split into 0.56 for the placements' matrices, 0.27 for the boxes'
distances, 0.66 for the sort, 0.57 to copy the boxes in order, and 0.12 to
upload them. The order only feeds the depth test, so a sort now holds
until the head has moved 25 cm. The boxes within that reach draw their
back faces meanwhile.

| Scene | Boxes | Each frame sorts | A frame between sorts | A frame that sorts |
| --- | ---: | ---: | ---: | ---: |
| S1 | 426 | 0.25 | 0.08 | |
| S5 | 4096 | 0.45 | 0.08 | 0.28 |
| S2 | 25,301 | 2.25 | 0.24 | 1.11 |

A stale order costs the GPU nothing. With every sort taken from a point
25 cm off the head, the app's GPU frame time read 1.62 ms on S1 against
1.63, 3.77 on S5 against 3.75, and 12.65 on S2 against 12.60.

**Trims.** Five changes to the mask march's fragments, each kept for a
gain of 3% or more on S1 or S5. Times are Quest p10s in ms at 545 MHz, one
window a row. B1 passes on both devices after the last.

| State | Mask, S1 | Solid and mask, S1 | Solid and mask, S5 | Solid and mask, S2 |
| --- | ---: | ---: | ---: | ---: |
| Before | 2.539 | 1.371 | 3.167 | 14.213 |
| 1. A flat output gives the face's axis | 2.435 | 1.267 | 2.998 | 13.964 |
| 2. A loop ahead of the march leaves the first live cells | 2.248 | 1.214 | 2.736 | 12.906 |
| 3. The loops count no steps | 2.109 | 1.181 | 2.603 | 12.119 |
| 4. The setup stays in floats | 2.012 | 1.173 | 2.514 | 11.897 |
| 5. The projection is a constant, and a solid face knows its side | 2.042 | 1.080 | 2.523 | 11.984 |

1. Both triangles of a box's face start at a corner no other face starts
   at. A flat output of that corner reaches the face's fragments alone,
   so a fragment reads its face's axis and side where it took 19
   instructions to find them from its point
2. A ray that starts inside a box takes no hit in the live cells it starts
   in. The march's loop tracked that in a flag every step. A loop of its
   own now steps such a ray to its first empty cell
3. The loops end on the ray's distance to the box's far faces and count
   nothing. A finite ray always reaches that distance. The vertex stage
   sends a corner whose position or ray is not finite to the origin, so no
   other ray reaches a fragment
4. The box's low and high cells reach the fragment as floats, and the
   start cell, its bit, and the distances to the next faces come from
   them with no integer step
5. A module is built for one view, so the view's projection folds into
   it. A solid box's fragment takes the side of its face from the flat
   output and computes no ray

`ovrgpuprofiler` counts the compositor's fragments with the kernel's. A
frame with no kernel fragments shows 14 M fragments and 0.34 G
instructions, so the earlier section's instructions per fragment mix the
two. The kernel's share of a frame's instructions:

| Kernel | Before | After | Occupancy after |
| --- | ---: | ---: | ---: |
| solid boxes and mask march, S1 | 0.74 G | 0.59 G | 86% |
| solid boxes and mask march, S5 | 1.30 G | 1.05 G | 66% |

**Variants that lost.**

1. Loops that end on a compare of the distance's bits, which a NaN also
   ends, ran S5 in 2.620 ms against 2.603 for trim 3 and 2.736 before it,
   and the mask arm on S1 in 2.188 against 2.109
2. Stray cells as boxes. A face of a tile's box gave up its layer's cells
   as boxes of live cells when they filled an eighth of the layer or less,
   up to 16 cells a tile. S5's pebbles came off every tile, and its
   marched boxes shrank by a layer. S5 ran in 3.160 ms against 2.523, and
   S2 in 16.855 against 11.984. S5 gained 19,200 boxes and S2 92,512

**Outcomes.**

1. A frame's CPU time on S2 falls from 2.25 ms to 0.24, and to 0.08 on S1
   and S5. The earlier section's 6.3 ms predates the f32 sort
2. A frame that sorts takes S2 1.1 ms. A head at walking pace sorts about
   every 16th frame
3. A view from inside a box with live cells was wrong under face depth
   and no bench view showed it. B1's new scene covers it
4. The trims take 21% off S1, 20% off S5, and 16% off S2. The march's
   occupancy stays at 62% to 66%
5. A box costs 33 to 53 ns a frame on Quest beyond its fragments, from
   the stray cells' boxes. S2's 25,301 boxes then cost about 1 ms. A box
   earns its place only where it saves a few hundred steps a frame
6. S5's far rays skim its top layer, where pebbles fill 2% of the cells.
   Those steps wait for B6's LOD because a cell there stands under a pixel
   tall from about 5 m out

