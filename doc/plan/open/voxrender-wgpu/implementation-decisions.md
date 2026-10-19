# Implementation decisions

Code-level choices a reviewer of the Rust would want explained, recorded as
they land.

## S1. Integer traversal

- `RenderRayWalk::new` takes one `RenderGridRay` per placement, in
  placement order. Primary rays come from `RenderGridViewRays` and shadow
  rays from fixed points on the face. `from_ray` quantizes a world ray for
  callers that hold one, such as `cast_ray`.
- The walk mirrors each axis the ray moves down, so it only steps up. A ray
  from outside starts one cell short of the face it enters through, which
  makes the entry an ordinary first step.
- Entry products and end cells use `i128`, so the walk is total over any
  `RenderGridRay`, whose fields are public. `GRID_RANGE_BITS` keeps the
  same values within `i64` for the GPU tiers.
- The error terms also give a hit's place across its face: `ONE * d` plus
  or minus a term before the step. `along` divides that once in `f64`.
  `point` rounds it, and a per-pixel shadow ray starts there.
- A point or spot light's ray comes from `toward`: the offset shifted right
  to 16 bits, ending where it reaches the light's coordinate on its major
  axis. A shift needs only a leading-bit count on the GPU, where a 64-bit
  division would not fit WGSL. The end replaces the `f64` distance cap, so
  `RenderRayWalk` lost `max_distance` and `cast_ray` filters by distance.
- A shadow ray's origin enters the other placements' grids through world
  space in `f64`. Only the hit's own placement starts from the exact fixed
  point. Exact origins across placements would need an integer map per
  placement pair, and no scene needs that yet.
- `ShadowTarget` stays in world space, and each sample quantizes the light
  into every placement's grid. That gives the same rays as a per-render
  table, which a GPU tier would upload instead.
- The largest corner component of a view's direction lands under 2^16 by
  `1 + (width + height) >> 12`, the most the steps' rounding can add. No
  pixel's direction exceeds 2^16.
- Halves round away from zero, so mirrored inputs quantize mirrored.
- A point out of range errors as `GridRange` where it is quantized, so
  `shade_ray`, `shade_hit`, and `shadow_factor` return `Result`.
- The goldens regenerated. 15 of 28 moved, each on at most 10 pixels and
  by at most 1/255 per channel:
  1. glass per-pixel, per-face, and per-corner
  2. l-shape per-pixel, per-face, and per-corner
  3. room, all four
  4. spot-room, all four
  5. two-placements per-corner

## S2. Desktop harness and primary ids

- The scene builders live in `voxrender::fixtures`, compiled under
  `any(test, feature = "fixtures")`. `bar_scene` and `check_goldens` stay
  test-only because they need `RenderRay` under `cpu` and the `png`
  dev-dependency. `precision_view_scene` holds the precision-sim's ten
  views as the scene's views.
- `GRID_RANGE_BITS` also bounds a grid's sides, so a hit point anywhere in
  its grid fits i32. That replaces S1's `i128` entry: grid rays and hit
  points hold i32, the walk computes in i64, and
  `RenderGridRay::in_range` states what the walk asserts on each ray.
- `RenderViewRays::to_grid_rays` and `RenderGridRay::from_ray` take the
  grid's bounds for the mirror check. `to_grid_rays` rejects far corners in
  f64 before their steps could overflow, and the integer corners settle the
  range's edge.
- A view's steps round their guard bits off halves up rather than away from
  zero. The bench then splits each step into whole units and a fraction
  under 2^12. The GPU sums the units with wrapping i32 and rounds only the
  fractions, whose sum stays under 2^27, which matches the reference
  exactly. No golden moved.
- `RenderGridViewRays`' fields are public so a GPU tier can upload them.
- `voxrender-bench` is binary-only. `first_hit.wgsl` writes each pixel's
  placement index, voxel id, and face. Placements merge by f32 distance,
  with ties going to the earlier placement as in the reference. A cell
  holds its material id plus one, so the shader finds the first hit by
  material change alone and needs no opacity.
- naga's Metal backend casts signed adds and multiplies through unsigned
  types, so the wrapping the entry relies on holds there.

## S3. The CPU renderer gets its own crate

- `voxrender` keeps every quantization type and function public and
  ungated because `voxrender-cpu` and every GPU tier share them.
  It no longer depends on `voxsurface`.
- `Render`'s default implementation cannot live beside the trait because
  `voxrender-cpu` depends on `voxrender`. voxsmith's `DependenciesImpl`
  implements it over `voxrender-cpu`, as it does `EncodePng` over `png`.
- voxsmith's `impl` feature pulls in `voxrender-cpu` even without `render`.
  Cargo cannot enable a dependency only when two features are both on. The
  default features hold both.
- The goldens and `test_utilities` moved with the renderer. The
  `VOXRENDER_UPDATE_GOLDENS` variable kept its name.

## S4. Shadow bits and B1 on this Mac

- A shadow bit records whether a corner's shadow ray meets any surface or
  starts inside a live cell. Glass clears the bit, so B1 checks which cells
  the ray visits and leaves the throughput to shading.
- Every light but the hemisphere casts the bits, at every corner, whatever
  its granularity. Bit 1 of a corner picks the face's far edge on `u` and
  bit 2 on `v`, the same on every face.
- `voxrender-cpu` exports `shadow_rays` and `ShadowTarget`, so the bench
  builds the reference's shadow rays from the reference's code.
- The shader carries a start into another grid through the placements'
  positions, rotations, and scales in f32, as the reference does in f64.
- The GPU's `toward` wraps an offset past 2^31, about 1.3 km at 1 cm
  voxels. Its end cell takes the entry's f32 guess and wrapping remainder.
  No B1 scene needs the correction there: disabling it changes nothing.
- The float walk runs over the integer walk's quantized inputs, which a
  Quest fallback would share. Only the traversal differs, so its flips are
  what Quest's goldens would tolerate.
- `precision_lights` holds the simulation's eight suns and two point
  lights. The S1 corner test and `precision_shadow_scene` share them. The
  scene's floor is a second placement, so its shadow rays also walk the
  cells' grid.

## S5. Quest harness and B0

- Gradle and cargo-ndk build the APK. cargo-apk last released in 2023 and
  xbuild in 2022. Gradle is Android's official build. It takes the Khronos
  loader as a Maven dependency.
- The SDK installs through Google's Android CLI because Google deprecated
  `sdkmanager`. The build never runs `cmdline-tools`, so no step installs
  it.
- Each tool installs by its maintainer's recommended route. Gradle comes
  from the checked-in wrapper.
- The Gradle task that runs cargo-ndk sets `ANDROID_NDK_HOME` from the
  build's `ndkVersion`. A shell profile never pins a repo's NDK.
- The Quest app lives in its own crate, `voxrender-bench-quest`, a
  `cdylib` that exports `android_main`. The desktop bench then never links
  openxr, ash, or android-activity. The crate compiles only for Android,
  so the pre-commit hook lints it with `--target aarch64-linux-android`
  when a commit touches it.
- The runtime creates the Vulkan instance and device through
  `XR_KHR_vulkan_enable2` from wgpu-hal's instance extensions, device
  extensions, and features. wgpu takes ownership of both. The app asks for
  Vulkan 1.1, the version that made multiview core.
- wgpu wraps each swapchain image with a no-op drop callback, so the
  runtime keeps the image. The wrap starts it as a color target because
  OpenXR hands out color images in `COLOR_ATTACHMENT_OPTIMAL` and takes them
  back in that layout.
- The swapchain takes the runtime's first sRGB format among
  `R8G8B8A8_SRGB` and `B8G8R8A8_SRGB`.
- A frame submits no layer until the views' orientations are valid. The
  runtime rejects a projection layer with untracked poses.
- The manifest sets `hasCode="false"`, and `android.builtInKotlin=false`
  keeps AGP 9 from adding the Kotlin standard library. The Khronos loader's
  AAR holds no Java.
- The manifest declares hand tracking as optional, so the headset launches
  the app without controllers. The bench takes no input.
- B0's probe logs one `b0` line per result. The startup results come before
  the session runs. The visibility mask and the frame times need the
  headset worn.
- The app asks for 90 Hz through `XR_FB_display_refresh_rate`, the rate the
  frame budget assumes. The runtime starts at 72 Hz.
- The compositor and app GPU times come from `XR_META_performance_metrics`,
  sampled after each rendered frame. Sampling skips 300 frames and reports
  p50 and p99 over each window of 600 after them. For about six seconds
  after launch the compositor draws extra layers. The first window can
  overlap them.
- Each indirect draw runs beside a direct draw of the same triangle, so a
  blank target points at the indirect call instead of the harness.
- Each swapchain format is tried three ways: attachment usage alone,
  OpenXR's storage and mutable-format usage, and Vulkan's flags through
  `XR_META_vulkan_swapchain_create_info`.
- The visibility mask is probed once the session gains focus and again
  whenever the runtime says it changed.
- A Vulkan feature struct joins the query chain only when the device lists
  its extension.
- Timed runs pin the GPU with `adb shell setprop debug.oculus.gpuLevel 4`.
  VrApi held level 4 at 545 MHz through B0's measured frames.
  `XR_EXT_performance_settings` offers only hints that the runtime's clock
  throttling treats as a baseline. The app logs the property's value with
  its frame times.

## S6. B1 on Quest

- `voxrender-bench` is a library. A benchmark takes a `Gpu` holding the
  caller's adapter info, device, and queue, and returns its report.
  `voxrender-bench-desktop` holds the CLI and opens a headless device.
  `voxrender-bench-quest` passes the OpenXR session's device. One desktop
  binary serves macOS, Windows, and Linux because wgpu picks each one's
  backend.
- `voxrender-wgpu` starts at S6 with B1's walk in `first_hit.wgsl`.
  `trace_hits` uploads a scene, traces a view, and reads back each pixel's
  `PixelHit`. B1's desktop half picked the integer walk. The float
  walk comes along because decision 1 can still make it Quest's walk.
  B1 judges both from `voxrender-bench`, so the benchmark crate holds only
  the reference side and the report.
- `ShadowTarget` moved from `voxrender-cpu` to `voxrender` because the GPU
  tier must not depend on the reference. `RenderLight::shadow_target` and
  `RenderViewRays::to_scene_grid_rays` hold the setup the reference, the
  GPU tier, and the bench each repeated. The reference's `render` now uses
  `to_scene_grid_rays`. No golden moved.
- B1's report lists the first mismatching pixels of each scene and walk.
  They went to stderr before, which Android drops.
- The Quest app runs B1 before it opens the session, at the desktop run's
  256 by 256 pixels. The reference renders on the headset, so the app reads
  no files. The app logs the report one line at a time because logcat cuts
  long messages.
- The Adreno 740's shader compiler, version E031.50.14.00, segfaults in
  `vkCreateComputePipelines` on a dynamic index into a bool vector. A
  bisect on the headset traced it to `entering[a]` and `flip[axis]`, and
  naga's runtime checks played no part. `first_hit.wgsl` tests the integer
  components instead. The output on this Mac did not change.
- android-activity catches a panic in `android_main` and finishes the
  activity. The default hook's message reaches no log, so the app sets a
  hook that logs it.
- Gradle's `cargoNdk` task always runs and leaves change tracking to cargo.
  Its inputs had covered only the app crate, so a change in
  `voxrender-bench` or `voxrender-wgpu` left the APK stale.
