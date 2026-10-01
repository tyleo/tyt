# Voxel rendering plan

Status: **open**. The shape below was agreed on 2026-09-28. Phase 1 and
phase 2 are built. The [checklist](checklist.md) tracks both, and the
[reference pages](../../../ref/render/README.md) hold the contract and the
profile language.

## Goal

Render a voxcore scene three ways from one definition of the image:

1. **Review images.** A PNG of a document, made offline on any machine with no
   GPU, for image models and people to look at what a voxel tool made.
2. **Realtime.** The renderer a future custom Rust engine draws with.
3. **VR.** The same renderer on a standalone headset.

The review PNG should show what the game shows. That holds only if every
renderer implements one contract. The plan is one contract, one reference
implementation, and realtime implementations that must match it.

## Modules, not an engine

The engine does not exist yet. This plan does not build it. It builds the
library crates an engine would compose later. Each crate takes data in and
hands data out. None of them opens a window, runs a frame loop, reads input,
loads assets from disk, or owns a scene between frames. A crate that needs a
file writes through an injected dependency, the way voxsmith injects `png`
today.

## Shape

1. **The contract.** What the image of a scene is: the frame, the surface,
   the materials, the lights, the views, and the output encoding. Written as
   a reference page and mirrored by shared types.
2. **The reference.** A CPU renderer that follows the contract literally. It
   makes the review images and the golden images every other renderer is
   tested against.
3. **The tiers.** Realtime renderers over the same contract, each built for a
   hardware budget. A tier turns off the contract features it cannot afford.
   The reference renders that tier's golden images with the same features
   off.

## Crates

The chain is `vxl` over `voxsmith` over `voxrender` over `voxsurface` over
`voxcore`. Each layer speaks only to the layers below it. voxsmith's
`object mesh` reaches `voxsurface` directly.

1. `voxsurface`, a new crate at `projects/utilities/voxsurface`, split out of
   voxsmith's `object mesh`. Holds what a grid's surface is: solidity, the
   boundary faces with culled and greedy merging, and the neighbor-occupancy
   corner occlusion as plain floats. Depends on `voxcore`, `ty-math`, and
   `branded-id` only.
   1. The mesher reads its grid through a small trait: bounds and solid at a
      cell. `VoxObject` implements it in the crate. `voxrender`'s flattened
      object implements it too, so the mesher never sees layers or palettes.
   2. Greedy merging keeps its per-cell key and span-fits predicate. That is
      how `object mesh` merges by what its program allows and a realtime tier
      merges by material.
   3. voxsmith keeps the merge rules, provenance, the value-language program,
      the records, and the writers. It lifts the occlusion floats into a
      value-language array where it computed them before.
2. `voxrender`, a new crate at `projects/utilities/voxrender`. Holds the
   contract types, the render scene, and the CPU renderer behind a default
   `cpu` feature. Depends on `voxsurface`, `voxcore`, `ty-math`, and
   `branded-id` only.
   1. The render scene is a `VoxMain` flattened for drawing: per object, one
      effective material per live voxel and a table of resolved material
      values; per placement, a world transform; per view and light, a world
      pose. Every renderer consumes it, CPU or GPU.
   2. The scene is built the way voxcore builds `VoxMain`. Every entity kind
      has a branded id, so nothing indexed is addressed by a bare integer.
      Objects and voxels take voxcore's ids because each mirrors one voxcore
      object or voxel. Placements, materials, lights, and views each get a
      brand of their own. Ids are `U32Id` or `UsizeId` over a brand, stored
      in `IdVec` and the `soa` pools. A voxel's material is an id into the
      scene's material table. A placement references its object by id, and a
      view references its subject placements by id. Mutation is `retain_*`
      and `release_*` with voxcore's cross-reference checks, which lets an
      engine keep one scene alive and edit it. The flatten from a `VoxMain`
      is one constructor over that API. The only integers are pixel
      coordinates, which address an image.
   3. The CPU renderer takes a render scene, a view id, and the lights, and
      returns a linear RGBA image buffer. It ray casts the dense grid directly
      with a DDA and reads solidity and corner occlusion from `voxsurface`.
   4. If the GPU crate wants the render scene without the CPU renderer, the
      `cpu` feature is the seam. The scene splits into its own crate only if
      that gate proves awkward. Packing ids into narrow GPU formats happens
      only at that crate's upload boundary.
3. voxsmith gains an `object_render` operation under its `object` group. It
   selects the objects, resolves every transform against the scene, calls
   `voxrender` once per view, and returns each view's image beside its
   resolved view. `encode_render_png` encodes an image through the existing
   injected encoder. Flags and profiles lower into one plain record that vxl
   hands over, the seam the mesh plan settled on. The record carries branded
   ids, never list positions. Its transforms are the configured shapes. vxl
   cannot resolve them because their frames need the scene. Its lights are an
   `IdVec<BRenderLight, ...>`, and every reference to one is a
   `U32Id<BRenderLight>`, as the mesh record's materials are an
   `IdVec<BMeshMaterial, ...>` referenced by `U32Id<BMeshMaterial>`. Views
   sit in an `IdVec<BRenderView, ...>` too, each carrying its name.
4. vxl gains `object render`, a thin front over the voxsmith operation. It
   takes the `--select` / `--select-index` object selectors of `object mesh`.
   The selected objects render together, placed by the hierarchy nodes
   reaching them, exactly as `object mesh` arranges them in one mesh. Views
   and lights come from [profiles](#profiles) and the flags that mirror them.
5. Later, `voxrender-wgpu`: the realtime tiers. It uploads the same render
   scene and draws it into a texture the caller owns. It has no window either.
   Presenting is the engine's job.

## The contract

[The contract](../../../ref/render/contract.md) says what the image of a
scene is: the frame, the surface, the materials, the shading, the lights with
their shadows and occlusion, the transforms, the views, the output encoding,
and the determinism the tests rely on. It moved out of this plan when phase 1
landed, so the reference page owns the wording every renderer follows.

## Profiles

Views and lights are configured the way `object mesh` configures materials
and primitives: profiles under `.vxlconfig`'s `object.render.profiles` key,
built-ins embedded in the binary, `--profile` applying one whole and stacking
on repeat, a mirroring flag per element, and `vxl profile object render list`
printing the merged namespace. The
[profile language](../../../ref/render/profile-language.md) holds the schema,
the loading, the built-ins, and the stacking, and the
[command reference](../../../ref/render/render.md) holds the flags.

## Realtime tiers

The contract specifies the surface, not the technique that finds it.
Rasterizing the surface's quads and marching a ray through the volume reach
the same faces, so both are valid implementations of primary visibility.

1. **Standalone.** The first tier, for headsets with mobile tiled GPUs.
   Primary visibility rasterizes the faces of each object with 4x MSAA and
   multiview stereo. `voxsurface` greedy-meshes those faces with the material
   as the merge key. Lighting is the part a grid makes special. It is
   computed at face granularity by the same grid rays the reference casts,
   one per face or face corner, in compute or on the CPU, amortized over
   frames. The values are stored with the faces, or in a per-object light
   volume where greedy merging shares a quad across faces. The fragment
   shader is a lookup. Relighting is local because a
   face's light depends only on cells within its ray length, so an edit or a
   moving object re-lights a bounded region. Under `per-face` or `per-corner`
   shadows the tier computes exactly what the reference computes. `per-pixel`
   shadows need a shadow map and drift from the reference at the edges.
   Culling walks the grid: a flood fill through empty cells from the camera
   skips objects no line of empty space reaches, on top of bounding-box
   culling. Faces pack into one integer each.
2. **Desktop.** A later tier. Adds Teardown's approach where the budget
   allows: an object drawn by rasterizing its bounding box and marching a DDA
   through a 3D texture, with traced sun, occlusion, and reflections and a
   denoiser over them.

Teardown's pipeline is a poor fit for standalone because every stage of it is
bandwidth-heavy on a tiled GPU:

1. A per-pixel loop of dependent 3D texture fetches
2. A deferred buffer of several 16-bit targets
3. A traced light per pixel
4. A spatiotemporal denoiser that reprojects previous frames and smears under
   head motion, which VR exposes

Marching also writes depth from the fragment shader, which disables the
tiler's early depth rejection. It gets no MSAA on a scene made of hard edges.
The data model survives unchanged, so a desktop tier can add the marching
later without touching the contract or the render scene.

## World size

An object is the chunk unit. A world is many placed objects, culled by their
bounding boxes. Nothing in the first cut needs a world-wide volume. voxcore's
grid cap of `2^27` cells bounds an object, never a scene.

## Decisions

1. One contract, one CPU reference, tiers that must match the reference.
2. Library crates only. Engine responsibilities stay out. The chain is `vxl`
   over `voxsmith` over `voxrender` over `voxsurface` over `voxcore`.
3. The reference marches the dense grid directly. It is the simplest correct
   renderer and validates the DDA a GPU marcher would reuse.
4. The standalone tier rasterizes meshes. Volume marching is a desktop tier.
5. Occlusion in the first cut is the neighbor-occupancy rule, so the standalone
   tier can match the reference exactly.
6. The shadow granularity is a switch, not a decision. The reference renders
   all three. The review tool's first milestone rendered all three over real
   assets on 2026-09-29 and picked `per-corner` as the default every light
   without a granularity takes, `studio`'s included, and the game's default
   look. Its soft staircase reads like the corner occlusion beside it. The
   standalone tier computes it exactly, where `per-pixel` needs a shadow
   map.
7. The standalone tier lights at face granularity from grid rays, the same
   rays the reference casts, so the headset shows the light the PNG shows.
8. The crate is `voxrender`, free on crates.io as of 2026-09-27. The GPU
   crate will be `voxrender-wgpu`.
9. The tonemap is Khronos PBR Neutral.
10. The command is `vxl object render` over the object selectors, so a render
    picks objects the way `object mesh` does.
11. Views and lights are profile elements with mirroring flags, cascaded and
    stacked as the mesh profiles are. Views key by name because the name
    suffixes the file and lets view profiles compose. Lights index by list
    position because a rig is one list. `viewsFrom` and `lightsFrom` import
    one half of a profile, the way `valuesFrom` imports a mesh profile's
    values.
12. Projection is per view and perspective by default.
13. `fit` is the bounding-sphere rule, so every fitted view of a subject
    sits at one distance whatever its orientation.
14. The default image is 1024 by 1024. Square suits a single asset. A
    reviewer's model downsamples anyway.
15. `voxrender` uses `branded-id` for every entity, with the `Render` type
    prefix and `BRender*` brands, mirroring voxcore's `Vox` and `BVox*`. No
    public API hands out or takes a bare list position.
16. A `<light-index>` is an integer only while vxl parses it. vxl converts it
    to a `U32Id<BRenderLight>` as it builds the record, exactly where
    `object mesh` turns `<material-index>` into a `U32Id<BMeshMaterial>`.
    voxsmith and everything below it see ids only. A view's name is the
    command-line handle the same way. The record resolves it to a
    `U32Id<BRenderView>` before it leaves vxl.
17. The mesher and the corner occlusion move out of voxsmith into
    `voxsurface`, free on crates.io as of 2026-09-28. `voxmesh` is taken. The
    PNG's occlusion and the exported mesh's occlusion then agree by
    construction. `voxrender-wgpu` gets greedy meshing without a second
    mesher. The rule: a crate breaks out when two crates that cannot depend
    on each other need one piece of code. The DDA stays in `voxrender`
    because `voxrender-wgpu` depends on it anyway.
18. Every view and light resolves to one world-space `TyPoseF64`. The
    configured transforms are three shapes named by what they carry: a pose
    for views, a rotation for directional lights, and a position for point
    lights. One combined shape was considered and dropped for two reasons.
    It accepts a part the entity never reads, a silent no-op. It needs prose
    rules the separate shapes carry in their types.
19. A light is a rotation, not a direction vector, so it hangs on a node the
    way a light under glTF's `KHR_lights_punctual` does. The rotation forms
    are one vocabulary shared by views and lights.
20. Point lights are in the first cut. They are a position. The shadow ray
    only changes from infinite to finite.
21. Bloom reads the emissive term alone, before the tonemap. A key light
    never makes a white face glow. A material's `emissiveStrength` says how
    hard the material glows. Bloom is off by default, which keeps the phase 1
    goldens and `flat` unchanged. The `glow` built-in turns it on. Proposed
    on 2026-09-30. Tuned on 2026-10-01 over `emissive`, whose eight voxels
    step from `0` to `20` in emissive strength: `glow` keeps the proposed
    strength of `1`, radius of `0.03`, and threshold of `1`. The radius is
    the standard deviation of the widest blur. `0.06` reads as haze.
    `energy-reactor` stays flat under `glow`: its lines emit at `0.6`,
    under glTF's default, and a line that thin and dim blurs to nothing
    even at a threshold of `0.3`.

## Deferred

Transparency and the GPU come after phase 2, each its own plan:

1. Alpha, `transmission`, and `ior`. The contract settles what a transparent
   voxel is before any tier follows it.
2. `voxrender-wgpu`, the standalone tier, and later the desktop tier.

The rest wait for a reason to pull them in:

1. Traced occlusion as a third `occlusion` value.
2. Area lights.
3. Tiling several views into one sheet.

`tyt fbx render`'s remaining surface does not carry over: its render engine
and sample count belong to Blender, its near and far planes to a depth
buffer, and its Euler camera pose to Blender's camera convention. Its
`--subject` lives on as `select`, its orbit as the `orbit` shape, and its
zoom as a numeric `distance`. Its lighting presets live on as light
profiles.

## Next

The checklist covers phase 1:

1. The `voxsurface` split, with `object mesh` moved onto it and its tests
   passing unchanged
2. The `voxrender` crate scaffold
3. The render scene flatten
4. The three transform shapes, their frames, the rotation forms, and `fit`
5. The DDA
6. The shading with the three light kinds and the three shadow
   granularities, over `voxsurface`'s occlusion
7. The profile schema with its built-ins and mirroring flags
8. The voxsmith operation with PNG output
9. The vxl command over the object selectors, printing to the terminal by
   default, with `--print-camera`
10. Golden-image tests over a fixture set
11. The three-shadow renders over real assets

Phase 2 adds:

1. The `node` frame, so a camera or a light can ride a hierarchy node
2. Spot lights, over the pose shape a view takes
3. Bloom over the emissive term, off by default, with the `glow` built-in

Transparency and `voxrender-wgpu` come after, as [Deferred](#deferred) says.
