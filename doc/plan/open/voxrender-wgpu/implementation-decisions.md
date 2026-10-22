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
- `HitTrace` replaced `trace_hits`. It uploads a scene and a view's rays
  once. `dispatch` records the walk into the caller's pass, and `trace`
  walks once and reads back the hits.
- `B1Timer` in `voxrender-bench` times B1's kernels. The Quest frame loop
  and the desktop's `b1-times` drive it. Each pass times one kernel over
  both eyes at the eye resolution, and the passes cycle through six
  kernels:
  1. the integer and float walks over the far wall scene 8 cells deep
  2. the same walks 16 cells deep
  3. the i32 and f32 dependent chains
- The first 300 passes warm up. A window holds 600 passes per kernel and
  reports each kernel's p50 and p99.
- The far wall scene views an empty grid from inside, so no ray enters
  from outside. It has no lights, so the walks trace primary rays alone.
  The step ratio divides the walks' growth from 8 to 16 cells deep, which
  cancels each pixel's setup and output. On this Mac a walk's time grows
  linearly with depth.
- The chain runs 64 links of an add, a compare, and a select per
  invocation, one invocation per pixel. It stores only on an end that never
  comes, so it stays ALU-bound.
- Metal shows a pass's timestamps to a resolve only once the pass's command
  buffer completes. A resolve in the same command buffer read the slot's
  previous pass. A slot now resolves in its own submit after
  `on_submitted_work_done` reports its pass.
- Metal overlapped consecutive passes that wrote separate buffers, which
  inflated each pass's time. The timer records nothing while its last pass
  runs. An unpaced desktop loop still inflated times by about 20% and
  doubled p99, so `b1-times` waits for each submit to finish.
- The Quest device requires `TIMESTAMP_QUERY`, which B0 listed.
- The first Quest timing put the integer step at 2.75x the float step,
  while the i32 chain ran at 1.03x the f32 chain. The step's cost came from
  its code. Both walks now pick the axis, step the cell, and update the
  error terms or `t_max` with vector selects, with no dynamic index. Each
  steps its cell's voxel id by a per-axis stride. Only a hit computes its
  position and distance. B1's report on this Mac did not change.
- With decision 1 settled, the float walk left `voxrender-wgpu` for
  `voxrender-bench`, which traces it for B1's record. `integer_walk.wgsl`
  holds the integer walk. `HitTrace::with_walk` appends another walk's
  WGSL to `first_hit.wgsl` in its place. `GpuWalk` is gone. B1's report on
  this Mac did not change.

## S7. Bench scenes and the per-object structure

- The research's five scenes live in `voxrender-bench`, built from
  `voxrender`'s fixtures and generators seeded by a cell hash. Each has
  90-degree eye views at head height:
  1. S1 art room: a 3 m room of 2 cm voxels open on two sides, with 1 cm
     props, seen at arm's length over its table and from a fitted
     orthographic view
  2. S2 city: 16 by 16 blocks in 1025 placements, buildings of 25 cm voxels
     scaled by up to a fifth, trees of 10 cm, and cars of 5 cm. Every fifth
     building and ninth car turns freely. Seen from a street and from the
     air
  3. S3 greenhouse: a 6 m frame of 2 cm voxels glazed with 84 overlapping
     panes, alternate panes a voxel further out and the roof's tilted 3
     degrees, with 1 cm plants inside
  4. S4 foliage: picket fences one voxel wide on one-voxel gaps, one of them
     at 33 degrees, sparse bushes, and sparse tree crowns
  5. S5 grazing floor: 20.48 m of 2 cm tiles four voxels deep. Its second
     view is orthographic, 4 m across, 10 degrees above the floor, because
     a view fitted to the whole floor at that angle covers a tenth of its
     pixels
- The scenes join B1's ids run, which takes 28 s at 256 by 256 pixels on
  this Mac. Each scene's tests check that every view hits on at least a
  tenth of its pixels
- `voxrender-bench`'s `Tree64` builds one object's 64-tree into one word
  array. Its doc gives the layout. Choices beyond the checklist:
  1. A slot word tags its kind in its top 3 bits. An empty tile is word 0
  2. A node's child slots follow its two mask words, so a node holds no
     child pointer and takes 8 B plus 4 B per non-empty child
  3. A leaf's palette holds only the values its cells use
  4. The build errors when a material id or a word offset is past a slot's
     29 bits. The cell cap does not bound the offsets, since a thin object
     can take more words than cells
- The trees against the harness's dense u32 cells:

  | Scene | Cells | Live | Dense KiB | Tree KiB | 1, 2-bit leaves | Uniform children |
  | --- | ---: | ---: | ---: | ---: | ---: | ---: |
  | S1 art room | 3358318 | 187806 | 13118 | 126 | 4984, 650 | 496 |
  | S2 city | 673024 | 341997 | 2629 | 337 | 7394, 5702 | 1308 |
  | S3 greenhouse | 9727072 | 291502 | 37996 | 203 | 8821, 326 | 744 |
  | S4 foliage | 1729600 | 358049 | 6756 | 773 | 969, 23629 | 32 |
  | S5 grazing floor | 4194304 | 2952989 | 16384 | 2162 | 0, 65536 | 0 |

  None of the five has a uniform 16^3 tile or a 4 or 8-bit leaf
- The tree walk is `voxrender-bench`'s `tree_walk.wgsl`.
  `HitTrace::with_walk` takes the words a walk reads per object, and
  `dense_cells` gives the integer walk's. The integer walk's entry and error
  terms moved into `first_hit.wgsl`'s `enter` and `error_at`, which the tree
  walk shares
- A cell in a leaf takes one integer step. A cell in an empty or uniform
  region jumps to the first cell past the region's cube, cut to the grid:
  1. An f32 guess picks the face the ray leaves across and the cell on each
     other axis
  2. Each cell's remainder corrects it under the entry's tie rules
  3. A cell past the cube moves the exit to its axis
  4. `error_at` gives the landing cell's error terms directly. They are exact
     because a cell on the ray keeps them within i32
- The bench builds each object's tree twice per trace: once to report a
  build error, and once inside `with_walk`, whose `cells` cannot fail

## S8. B2 on Quest

- The ray-query BLAS holds one AABB per 4^3 brick or per 16^3 tile. The run
  times both, and the faster and steadier one wins
- `B1Timer` became `PassTimer`, which times B1's or B2's kernels. The desktop
  runs B2 through `b2-times`. The Quest app now times B2. It still runs B1's
  ids before the session opens
- B2's step loop reads a 256^3 grid in one of four layouts:
  1. A storage buffer of u32 cells in raster order
  2. A 3D texture and a 2D array of R32Uint, the array with one layer per x
  3. u32-pair masks, one pair per 4^3 brick. A step reads the mask word and
     its bit's rank as `tree_walk.wgsl` does
- Each step adds the last read, masked by a zero the compiler cannot see, to
  its address. A read of a value no cell holds would end the loop, as a hit
  ends a walk
- Hashed cell values keep the data from compressing. The steps run along
  the diagonal a DDA takes, one axis per step. The image's x runs along the
  buffer's contiguous axis
- The loops run 8, 16, and 32 steps. The report fits a least-squares line of
  p50 against the count per layout and gives its slope as ms per step and
  Gsteps/s. On Quest, B1's walk took about 0.8 ms per step over both eyes,
  so 32 steps keep a pass near 25 ms
- B2's tree walk traces S1 and S5 without their lights, because B2 asks for
  the primary ray rate. Each pixel walks every placement, as in B1
- A counted variant of the tree walk gives the iterations per ray. Each call
  returns a hit nearer than the last, with the running sum of loop
  iterations as its voxel id. The pixel's hit then carries the total over
  every placement. The bench reads the counts once before timing. The timed
  walk keeps the unread increment
- The report gives each scene's Grays/s, iterations per ray, and
  Giterations/s. Giterations/s compares with the step loops' Gsteps/s
- A ray-query candidate marches its brick two ways, and the run times both:
  1. The integer march runs the tree walk, cut to the brick, along the
     placement's integer pixel ray
  2. The float march runs an f32 DDA along the world ray in the grid.
     It crosses a region of one value in a single jump, as the tree walk does
- The faster march that keeps B1's line wins. Both stay in the bench, so the
  choice can rerun. B1 judges the integer marches and counts the float
  marches' flips for the record, as with the float walk. A float march over
  the line can win only through a logged change to the line
- A brick's first cell is a hit only when its value differs from the cell
  before the entry face, as on a walk over the whole grid. Each AABB reaches
  1/64 cell past its brick, so a brick that the f32 world ray only grazes
  still reaches the integer march
- The ray-query trace casts primary rays only. Shadow rays through it wait
  for B5 or B3
- `voxrender-wgpu` exports `GRID_WALK_WGSL`, the walks' shared WGSL, and
  `placement_bytes`, so the ray-query shader walks the same placements.
  `RenderViewRays` exposes its pixel steps for the world ray
- The desktop and Quest devices request `EXPERIMENTAL_RAY_QUERY` through
  wgpu's experimental token
- Naga 30 places a `rayQueryGenerateIntersection` argument's evaluation
  after the call. The shader evaluates the distance before its `if`
- The float march carries the world ray into the grid through the
  placement's transform. On Quest, the grid ray from the candidate's
  `world_to_object` missed 15 to 84% of each scene's pixels
- B2's window holds 120 passes per kernel, where B1's holds 600, to keep a
  worn run short. Most of B2's 22 kernels take longer than a frame
