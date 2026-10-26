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
