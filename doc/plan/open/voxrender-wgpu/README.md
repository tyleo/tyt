# voxrender-wgpu plan

Status: **open.** Opened 2026-10-04 after the
[research](reference/research.md). The steps live in
[checklist.md](checklist.md). Code-level choices are logged in
[implementation-decisions.md](implementation-decisions.md). Measurements go in
[benchmarks.md](reference/benchmarks.md). The
[contract](../../../ref/render/contract.md) holds the render rules. The
[follow-ups plan](../voxel-rendering-followups/README.md) points here.

## Goal

Draw the render scene in realtime in VR, on phones, and on desktop, at a cost
that grows with pixels and never with voxels. A giant scene then needs no
tuning once it holds the frame rate. Meta Quest 3 is the hard target. It must
draw both eyes at 90 Hz with voxels fine enough to read up close. Looking good
in the headset is the bar.

`voxrender-wgpu` draws into a texture the caller owns and has no window. An
engine above it owns OpenXR.

## Priority

The VR experience comes first. The research's
[Priority](reference/research.md#priority) section ranks the goals and says
what an over-budget frame gives up. Every decision below follows that ranking.

## Measure first

The research found that a full-density march on Quest 3 likely falls about 3x
short of the frame budget. The finding rests on estimates. Nobody has measured
our walk on the device, so this plan measures before it builds. The research's
section 7 lays out benchmarks B0 to B12, each with its setup, a pass or fail
threshold, and the action for each outcome. B3 picks the Quest front end from
five candidates. The renderer's design settles after B3. The steps that build
it join this plan then.

A benchmark that needs a contract rule gets the rule in the reference first. A
GPU tier is then judged against goldens the reference renders under that rule.

## Crates

1. `voxrender` takes the contract changes, each in the step whose benchmark
   needs the change. It also holds the quantization code the reference and
   every GPU tier share. A consumer draws through its `Render` trait
2. `voxrender-cpu` holds the reference renderer
3. `voxrender-bench` is new and unpublished. It holds the harness and the
   competing front ends. It runs headless on desktop and as an OpenXR app on
   Quest. B3's losing arms are deleted
4. `voxrender-wgpu` starts when B3 picks the front end. The winning arm moves
   into it. The name was free on crates.io as of 2026-09-27

A tier turns off the contract features it cannot afford, and the reference
renders that tier's goldens with the same features off. `voxsurface`'s cull
knows transparency, so a raster arm can show the glass the reference renders.

## Decisions

1. The reference walks rays in integers. It runs the integer DDA of the
   research's
   [section 3.3](reference/research.md#33-exact-traversal-and-precision) on
   inputs that shared code quantizes once, so cell and face choices agree
   across GPUs. The GPU tiers run it in i32 alone, grid entry included.
   The goldens regenerate. If B1 finds the integer step costs more than
   1.25x a float step on Quest, Quest keeps a float walk
2. A tier matches the reference within reason. An input the GPU derives in
   f32, such as a shadow's start in another placement's grid, can flip a
   few samples. B1's line bounds the share. No tier takes 64-bit paths or
   special-case math to remove the last flips
3. Nothing enters `voxrender-wgpu` until a benchmark has picked it
4. A threshold is fixed before its benchmark runs. A changed threshold is
   logged as a decision before the run, never fitted to a result
5. The Quest benchmarks run on a retail Quest 3 in developer mode
6. The headset harness builds wgpu from the OpenXR session's Vulkan instance
   and device through wgpu-hal. bevy_mod_openxr runs the same pattern on Quest
7. The repo README's Development section documents the Android setup, which
   runs by hand. No build script fetches or configures it
8. Phones come last. B12 and the phone half of B1 run when a device is at
   hand

## Out of scope

1. visionOS, until Metal ray queries are fixed
2. Async compute, mesh shaders, f16, and 64-bit atomics on Quest. Stock wgpu
   cannot reach them there
