# Voxel rendering plan

Status: **open**. The shape below was agreed on 2026-09-28. Nothing is built
yet. The [checklist](checklist.md) tracks phase 1.

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

## The contract, first cut

1. **Frame.** voxj's: glTF Y-up, right-handed, +Z toward the viewer. A voxel
   at `p` fills the unit cube with min corner `p`. Placement follows the node
   DAG; one path, one placement. A voxel size in meters scales the whole scene
   the way `object mesh` does.
2. **Surface.** The boundary faces between live and non-live cells, as
   `voxsurface` enumerates them. Faces are axis-aligned unit squares with flat
   normals. No smoothing, no bevels, no sub-voxel detail.
3. **Materials.** The effective palette per voxel, in voxj's glTF vocabulary
   with glTF's defaults. The first cut shades `baseColor`, `metallic`,
   `roughness`, `emissiveColor`, `emissiveStrength`, and `occlusionStrength`.
   Every live voxel is opaque: alpha, `transmission`, and `ior` are deferred.
4. **Shading.** glTF's metallic-roughness model: Lambert diffuse and GGX
   specular in linear light.
5. **Lights.** A list of lights, each with a color and a strength, plus one
   occlusion switch. The first cut has three kinds:
   1. A `directional` light is a rotation. It shines down its local -Z, as a
      light under glTF's `KHR_lights_punctual` does. It carries a shadow
      granularity.
   2. A `point` light is a position with glTF's punctual falloff: inverse
      square, an optional `range`, and glTF's smooth cutoff at that range. It
      carries a shadow granularity too. Falloff runs in meters after the
      voxel size applies, so one rig lights a large voxel size differently
      from a small one.
   3. A `hemisphere` light is the ambient term, a sky color above and a
      ground color below, about world +Y. It has no transform.

   The occlusion switch is `none` or `corner`. `corner` is the
   neighbor-occupancy rule voxel art uses, one value per face corner from the
   three adjacent cells. It has one implementation, in `voxsurface`. The
   reference shades with it, `object mesh` bakes it, and a standalone tier
   stores it with its faces. Traced occlusion is a later value. A shadow is
   one grid ray toward the light. The ray runs to infinity for a directional
   light and ends at a point light. It is sampled at one of three
   granularities:
   1. `per-pixel`: a crisp diagonal edge across faces, the MagicaVoxel render
      and Teardown look.
   2. `per-face`: one value per voxel face, a crisp staircase at voxel
      resolution, the look Minecraft's Vibrant Visuals snaps to.
   3. `per-corner`: one value per face corner, interpolated, a soft staircase,
      the vanilla Minecraft smooth-lighting look with a sun.

   `none` turns a light's shadow off.
6. **Transforms.** Every view and light resolves to one world-space
   `TyPoseF64`, a position and a unit quaternion. Object placements keep the
   `TyTransformF64` their nodes carry, because voxels scale and cameras do
   not. A view reads both parts of its pose. A directional light reads the
   rotation and a point light reads the position. The configured form is
   three shapes, named by what they carry. An entity is never handed a part
   it has no use for. Each shape is a tagged union over the frame its values
   are read in:
   1. `world` is the document's frame
   2. `subject` has world axes centered on the subject's bounds
   3. `camera` is the view being rendered, so a light in it follows every
      view
   4. `orbit` is a position on a sphere about the subject's center, facing it

   A rotation is one of four forms, shared by every shape:
   1. `quaternion`, as a node stores it
   2. `euler`, angles about the fixed x, y, then z axes, as
      `node set rotation` takes them
   3. `look-at`, toward a point in the frame
   4. `angles`, the rotation of something at that spherical direction about
      the frame's origin, facing the origin. For a light this is where it
      comes from

   `look-at` and `angles` take the frame's +Y as up. A direction along +Y or
   -Y takes -Z as up instead. A `top` view then shows the front at the bottom
   of the image. voxsmith resolves transforms in the order subject bounds,
   then views, then lights. Floats narrow to `f32` at the same upload
   boundary where ids pack.
7. **Views.** A view is a named camera: a pose transform, a projection, and a
   field of view or an orthographic scale. Projection is perspective unless
   the view says otherwise. The subject defaults to the rendered objects.
   `select` narrows it with hierarchy-path globs. An orbit's distance
   defaults to `fit`, the rule `tyt fbx render` uses. The subject's
   world-space bounds give a center and a diagonal. The camera sits on its
   orbit at the distance, or orthographic scale, that fits the bounding
   sphere of that diagonal into the shorter image axis with a small margin.
   A sphere fits regardless of orientation, so every fitted view of one
   subject sits at one distance. The views show inline in the terminal one
   below another, or under `--to png` each writes one PNG beside the input,
   named by the input's stem or `--file-stem`. With several views, the
   view name joins the stem with a hyphen, as `object mesh` joins object
   names.
   `--print-camera` prints a view's resolved pose as a `world` transform
   with a `quaternion` rotation. The output pastes back as a flag or a
   profile entry.
8. **Output.** Linear light, the Khronos PBR Neutral tonemap, sRGB transfer,
   8-bit RGBA. The background is transparent by default. PBR Neutral keeps
   base colors true until highlights compress. A voxel palette is what a
   reviewer most needs to see unchanged.
9. **Determinism.** No randomness anywhere in the first cut. Tests compare
   images with a small per-channel tolerance, not byte equality, because
   float math differs across platforms and GPUs.

## Profiles

Views and lights are configured the way `object mesh` configures materials
and primitives:

1. A profile lives under `.vxlconfig`'s `object.render.profiles` key
2. Built-ins are embedded in the binary
3. `--profile` applies one profile whole and stacks on repeat
4. Each profile element has a mirroring flag
5. `vxl profile object render list` prints the merged namespace

The rules of the [mesh profile language](../../../ref/mesh/profile-language.md)
carry over:

1. A config profile sharing a built-in's name replaces it wholesale
2. An explicit flag replaces the element it collides with
3. Two stack members setting one element error

```ts
/** A profile; each element mirrors a `vxl object render` flag. */
interface Profile {
  /** One line the profile listings print beside the name. */
  description?: string;

  /** Mirrors `--width`. */
  width?: number;

  /** Mirrors `--height`. */
  height?: number;

  /** Mirrors `--background`; omitted, transparent. */
  background?: "transparent" | string;

  /** Mirrors `--occlusion`; omitted, `corner`. */
  occlusion?: "none" | "corner";

  /** Mirrors `--voxel-size`, meters per voxel; omitted, `1`. */
  voxelSize?: number;

  /** Mirrors `--views-from` per entry; only the views travel. */
  viewsFrom?: string[];

  /** Mirrors the `--view-*` flags, keyed by the name that suffixes the file. */
  views?: Record<string, ViewEntry>;

  /** Mirrors `--lights-from` per entry; only the rig travels. */
  lightsFrom?: string[];

  /** Mirrors the `--light-*` flags, its list position the `<light-index>`. */
  lights?: LightEntry[];
}

type Vec3 = [number, number, number];
type Quat = [number, number, number, number];

/** A view. */
interface ViewEntry {
  /** Mirrors `--view-frame`, `--view-position`, and a rotation flag, or
   *  `--view-orbit` for the whole element. */
  transform: PoseTransform;

  /** Mirrors `--view-projection`; omitted, `perspective`. */
  projection?: "perspective" | "orthographic";

  /** Mirrors `--view-fov`, the vertical field of view; omitted, `35`.
   *  Errors under `orthographic`. */
  fov?: number;

  /** Mirrors `--view-scale`, the world units across the shorter image axis
   *  under `orthographic`; omitted, `fit`. Errors under `perspective`. */
  scale?: number;

  /** Mirrors `--view-select`, hierarchy-path globs; omitted, the rendered
   *  objects. The subject that `subject` and `orbit` are about. */
  select?: string[];
}

/** A light. Mirrors `--light <light-index> <kind>`. */
type LightEntry =
  | {
      kind: "directional";
      /** Mirrors `--light-frame` and a rotation flag. */
      transform: RotationTransform;
      /** Mirrors `--light-shadow`; omitted, `per-corner`. */
      shadow?: "none" | "per-pixel" | "per-face" | "per-corner";
      /** Mirrors `--light-color`. */
      color?: string;
      /** Mirrors `--light-strength`. */
      strength?: number;
    }
  | {
      kind: "point";
      /** Mirrors `--light-frame` and `--light-position`, or
       *  `--light-orbit` for the whole element. */
      transform: PositionTransform;
      shadow?: "none" | "per-pixel" | "per-face" | "per-corner";
      color?: string;
      strength?: number;
      /** Mirrors `--light-range`, meters; omitted, no cutoff. */
      range?: number;
    }
  | {
      kind: "hemisphere";
      /** Mirrors `--light-sky` and `--light-ground`. */
      sky?: string;
      ground?: string;
      strength?: number;
    };

/** A position and a rotation. Views, and spot lights later. */
type PoseTransform =
  | { kind: "world"; position: Vec3; rotation: Rotation }
  | { kind: "subject"; position: Vec3; rotation: Rotation }
  /** Degrees. `distance` omitted, `fit`. */
  | { kind: "orbit"; azimuth: number; elevation: number;
      distance?: number | "fit" };

/** A rotation only. A directional light sits at its frame's origin. */
type RotationTransform =
  | { kind: "world"; rotation: Rotation }
  | { kind: "camera"; rotation: Rotation };

/** A position only. Point lights. */
type PositionTransform =
  | { kind: "world"; position: Vec3 }
  | { kind: "subject"; position: Vec3 }
  | { kind: "camera"; position: Vec3 }
  /** Degrees. `distance` is required. */
  | { kind: "orbit"; azimuth: number; elevation: number; distance: number };

/** Shared by every shape. Each form mirrors the `--view-*` and `--light-*`
 *  flag of its name. */
type Rotation =
  /** The form a node stores. */
  | { kind: "quaternion"; value: Quat }
  /** Fixed x, y, then z; `unit` omitted, `deg`. */
  | { kind: "euler"; value: Vec3; unit?: "deg" | "rad" }
  /** Aims -Z at a point in the frame; omitted, the frame's origin. */
  | { kind: "look-at"; target?: Vec3 }
  /** Azimuth from +Z toward +X, elevation toward +Y, facing the origin. */
  | { kind: "angles"; azimuth: number; elevation: number };
```

Views and lights are independent halves. A profile may set either. A stack
that sets no view renders the built-in `hero` view. A stack that sets no
light uses the built-in `studio` rig, the way `object mesh` falls back to its
implicit primitive. `--profile turnaround --profile dusk` composes a view set
with a light rig.

A profile can import one half of another profile, the way a mesh profile's
`valuesFrom` imports values. `viewsFrom` imports views and `lightsFrom`
imports a rig:

1. Imports land depth-first in list order, ahead of the profile's own views
   or rig
2. Only the imported half travels. The image elements and the other half
   stay behind
3. A profile's views and its rig each land once, however many imports and
   `--profile` flags bring them
4. Imports resolve after the cascade merges, so a config that overrides
   `hero` changes `turnaround`
5. An imported element collides like a stack member's. A rig is one
   element, so a profile's rig comes from its `lights` or from one import
6. An import cycle errors

The built-ins:

1. `hero`: one perspective view on an orbit at 45 degrees of azimuth and 30
   of elevation, the front-right-top.
2. `front`, `back`, `left`, `right`, `top`, `bottom`: one view each on an
   orbit along an axis.
3. `turnaround`: imports the views of `hero`, `front`, `right`, `back`, and
   `left` for one run.
4. `studio`: one directional light in the `camera` frame at `angles` of -30
   and 30, above and to the left of whoever is looking, over a hemisphere
   light. The offset gives a box three distinct shades. The three-shadow
   renders in the checklist set its shadow granularity.
5. `flat`: one directional light in the `camera` frame at `angles` of zero
   and zero, a headlight with no shadow, for judging color alone.

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

## Deferred

1. Alpha, `transmission`, and `ior`.
2. Traced occlusion as a third `occlusion` value.
3. Spot and area lights. A spot light takes the pose shape a view takes.
4. The `node` frame: a transform read in a hierarchy node's world transform,
   so a camera can ride a player.
5. Showing the image inline in the terminal when no output is given, as
   `tyt fbx render` does over Kitty, iTerm2, and Sixel. vxl would carry its
   own small implementation because it does not depend on `tyt-injection`.
6. Tiling several views into one sheet.
7. The desktop tier and `voxrender-wgpu`, each its own plan.

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

Phase 2 is the `node` frame and spot lights.
