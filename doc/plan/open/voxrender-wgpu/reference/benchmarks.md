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
- Command: `cargo run --release -p voxrender-bench -- b1 --size 256`
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
