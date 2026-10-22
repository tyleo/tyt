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
