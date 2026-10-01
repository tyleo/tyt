# Implementation decisions

Code-level choices a reviewer of the Rust would want explained, recorded as
they land.

## S1. voxsurface split

- `SurfaceGrid` answers solid-at-cell with a handle: `cell` returns
  `Option<Self::Cell>`, and the provided `is_solid` takes a signed position
  and treats the outside as empty. The handle is what the mesher keys faces
  by and records per face, so a `VoxObject` grid hands back voxel ids and
  voxsmith's swatch lookups stay as they were. voxsmith's `MeshGeometry` is
  the alias `SurfaceMesh<U32Id<BVoxVoxel>>`.
- `SurfaceMesh` records the `SurfaceSpan` of each quad beside the cells
  under it. The span settles the corner order, and `corner_occlusion` reads
  a span, so one function serves a merged quad and a hit face alike.
- A span's `corners` fixes the winding: `u` then `v` turn counter-clockwise
  about `+d`, so the `+` side runs `u` first and the `-` side `v` first.
  The mesher pushes vertices in that order and `mesh_occlusion` relies on
  it. The cross-product check the old `push_face` made is gone.
- `mesh_grid_keyed` drops the old `track_materials` switch and the
  `material_indices` it filled. Nothing in voxsmith read them.
- voxsmith re-exports `SurfaceMethod` as `Method`, `SurfaceSpan` as
  `FaceSpan`, and the alias above, so its records and vxl's flags are
  unchanged. The meshing and occlusion functions are called from
  `voxsurface` directly.
- voxsurface depends on `voxcore` without its `color` feature. voxsmith
  pulls it in under `object`.

## S2. voxrender scaffold

- `RenderLight` is one enum with a struct variant per kind. Each variant
  carries the part of a pose it reads: a rotation for `Directional`, a
  position for `Point`, nothing for `Hemisphere`.
- `RenderProjection` pairs the field of view with `Perspective` and the
  scale with `Orthographic`, so a view never carries the one its projection
  ignores. The field of view is in radians.
- `RenderMaterial` colors are `TyLinSrgbF64` and the image's pixels are
  `TyLinSrgbaF32`. The scene stays `f64` and narrows at the image.
- `RenderImage::new` accepts a zero side. The buffer is empty and harmless.
  The zero-side error belongs to the operation that asks for a render.
- The `cpu` feature is declared and empty until the DDA lands.

## S3. Render scene

- An entity that mirrors one voxcore entity takes its id. Objects are
  keyed by `BVoxObject` and cells by `BVoxVoxel`, which keeps the
  correspondence a sync layer against `VoxMain` edits would need. Nothing
  builds that layer yet. Materials, placements, lights, and views match
  nothing one to one and keep brands of their own. voxcore's
  `VoxObject::raster_id` and `raster_position` became public, so the render
  grid numbers cells with voxcore's formula instead of a copy.
- Objects sit in an `IdVec<BVoxObject, Option<RenderObject>>`, indexed by
  the caller's id. The other kinds mirror `VoxState`: an `IdStruct` beside
  an `IdField`, `release_stable` so survivors keep their order, and a
  `Drop` that releases every column. There is no `gc`, because nothing
  saves a scene.
- A placement's transform maps grid units straight onto world meters. The
  flatten folds the grid origin and the voxel size into it: the path's
  world position scales by the voxel size, and the placement's scale is the
  path's scale times the voxel size. The renderer then never sees an origin
  or a voxel size. Scaling every node position by the voxel size is one
  uniform scale of the whole path, so this matches `object mesh`.
- The path walk composes node transforms with `TyTransformF64::compose`,
  which is exact under uniform scale. A non-uniform scale under a rotated
  child would differ from a matrix composition.
- Placements come from the roots. An object only an orphan node lists is
  unplaced and gets the identity placement.
- A `RenderView` carries no subject placements. The subject is resolved
  where the transforms are, in voxsmith, and `subject_bounds` is a scene
  query over placement ids.
- `subject_bounds` frames the live extent, not the grid bounds, so empty
  margins of a grid never push the camera back.
- `RenderObject::set_voxel_material` is public and unchecked, so a
  standalone object can be filled before `retain_object` checks every
  reference. Once retained, voxels change through the scene's
  `set_voxel_material`, which checks the material.
- The material table deduplicates on the bit patterns of the six values.
- A voxel's material resolves through `VoxEffectivePalette::voxel_value`,
  one lookup per property per voxel. A scalar reads a float pool and a
  color a float vector pool, dropping alpha. Any other kind errors.

## S4. Transforms

- `TyQuaternionExt::from_look_direction` takes an explicit up and returns
  `None` for a zero direction or one along the up. The up rule of the
  contract, +Y unless the direction runs along +Y or -Y, then -Z, lives in
  voxsmith's `look_rotation`, because the fallback is the contract's choice
  and not math.
- `TyVector3Ext::from_azimuth_elevation` turns from +Z toward +X and then
  toward +Y, the direction the `angles` and `orbit` forms share. Both take
  degrees and pass through `TyAngleUnit`.
- `FIT_MARGIN` is five percent of the bounding radius. `fit_distance` fits
  the sphere into the vertical field of view on a square or wide image and
  into the narrower horizontal one on a tall image. `fit_scale` is the
  sphere's diameter with the margin.
- An orbit under an orthographic projection with a `fit` distance sits one
  `fit_scale` out from the center, past the sphere. The distance never
  changes what an orthographic image frames.
- The shapes are voxsmith enums with the README's names: `PoseTransform`,
  `RotationTransform`, `PositionTransform`, and `Rotation`. `FitOrFixed`
  is one enum for the orbit distance and the orthographic scale, and
  `ViewProjection` pairs each projection with the one length it reads, so a
  `fov` under `orthographic` is unrepresentable in voxsmith. vxl errors on
  that pairing while it builds the record.
- The resolvers take the `RenderElement` they report on. A `subject` or
  `orbit` frame over a subject with no voxel, a `fit` over one, and a
  look-at aimed at the entity's own position error on that element.
- A directional light's look-at needs a target because the light sits at
  its frame's origin.
- A `camera`-frame light composes with the view's resolved pose, so the
  render operation resolves it once per view.

## S5. DDA

- `cast_ray` moves the ray into each placement's grid units with the
  inverse rotation and a division by the scale. The direction keeps its
  length, which leaves the ray parameter in world distance. Hits in
  different placements compare by that distance directly.
- A hit's face is a unit `SurfaceSpan`, which `corner_occlusion` reads
  like a merged quad. The hit's `along` holds the fractions across the
  face's `u` and `v`, the coordinates the corner blend takes.
- A ray that starts inside the grid skips its starting cell. A camera
  inside a voxel sees out of it. A shadow ray cast from a face cannot hit
  that face's voxel through rounding.
- The entry cell is the entry point floored and clamped into the grid. A
  ray that enters on a face lands in the cell behind that face. A solid
  entry cell reports its hit on the slab's entry axis.
- `RenderViewRays` lowers a view's projection once per render into pixel
  (0, 0)'s ray plus per-pixel steps for the origin and the direction.
  Perspective zeroes the origin steps and orthographic zeroes the direction
  steps. Building a pixel's ray takes no branch on the projection and
  recomputes no per-view constant. The contract keeps `RenderProjection`
  unchanged.
- The `cpu` feature gates `RenderViewRays`, `cast_ray`, `RenderRay`, and
  `RenderHit`. The GPU crate computes its own hits.

## S6. Shading

- Shading math runs on `TyLinSrgbF64` through palette's component
  arithmetic. The image narrows to `f32` at `set_pixel`.
- The hemisphere term reflects off the diffuse color plus the
  normal-incidence reflectance. The reflectance keeps a metal under
  ambient light alone from going black: the metal reflects the sky in its
  base color. glTF fixes the BRDF and leaves the ambient approximation to
  the renderer.
- A floor of `1e-3` on GGX alpha keeps a roughness of zero from producing
  a NaN at the reflection.
- The corner occlusion darkens only the hemisphere term, as glTF's
  occlusion texture does. The material's occlusion strength scales it.
- A shadow sample starts `1e-4` grid units out along the face normal. A
  per-pixel or per-corner sample also sits at least `1e-3` of the face in
  from its edges. Every sample starts in the empty cell in front of its
  face, and a corner ray still meets a solid neighbor across the edge.
- `bilinear` serves both the per-corner shadow blend and the corner
  occlusion blend. It reads values in the span's winding order.
- A point light's shadow ray ends at the light, so an occluder past the
  light casts nothing.
- `render.rs` is the only caller of the shading and shadow helpers, so
  they live there as private functions with their tests.

## S7. Output encoding

- `tonemap` implements the published Khronos PBR Neutral curve at the
  image's `f32` precision. A mid gray keeps its value less the flare
  offset. Only a color whose brightest channel passes `0.8` compresses.
- `RenderOutput::from_image` composites the background after the tonemap
  and the transfer. The caller's sRGB byte background lands in the file
  unchanged.
- `RenderOutput` and `tonemap` stay outside the `cpu` feature because a
  GPU tier reads back a linear image and encodes it the same way.

## S8. Profiles

- The profile types mirror the README's TypeScript with serde:
  1. The shapes and rotation forms tag on `kind` in kebab case
  2. Every struct denies an unknown key
  3. Every element is optional, so a stack member can set a view's
     projection alone
  4. The entry types carry an `Entry` suffix, as the mesh profile's do
- Each field's type rejects a bad value at load:
  1. `width` and `height` are `NonZeroU32`
  2. Strengths are `NonNegativeF64`
  3. Ranges, distances, fields of view, and scales are `PositiveF64`
  4. Colors are `SrgbColor`, parsed from a `#RRGGBB` hex
- The record builder checks the field of view's upper bound.
- Views stack by name, element by element. The lights stack as one
  element, the rig. Stacking two rigs errors instead of merging lights by
  position. A view set and a rig compose.
- `viewsFrom` and `lightsFrom` resolve as the stack lands, the way mesh's
  program builder resolves `valuesFrom`. A cycle errors when a stack
  reaches it. The claims track the profile that set each element, so an
  error reports the imported profile itself.
- Each profile's views and rig land once. A stack of `turnaround` and
  `hero` lands `hero`'s view once instead of erroring on a collision with
  itself.
- `lightsFrom` takes a list for symmetry with `viewsFrom`. Only one entry
  can bring a rig because a rig is one element.
- `studio` sets no shadow granularity, so it takes the record builder's
  default. That default is the value's one home, which S12 changes.
- `flat` has a strength of pi. Under a head-on Lambert term, pi renders a
  white base color as white.
- vxl reaches the voxrender enums through voxsmith's `object_render`
  re-exports, so vxl still speaks voxsmith only.
- The mirroring flags land with the `object render` command in S10
  because a flag without a command has no parser to test.

## S9. voxsmith operation

- `render` returns one `RenderedView` per view id, the image beside the
  view resolved to world space. A caller prints the pose back as a `world`
  transform. Resolving it again would repeat the subject query.
- The image is voxrender's `RenderOutput`, 8-bit sRGB with straight alpha
  and the background composited in. `encode_render_png` beside `render`
  encodes one through `EncodePng` with the sRGB transfer stamped. A caller
  that shows the image in a terminal needs the pixels, and one that writes
  a file needs the bytes, so `render` stops at the pixels.
- A view's `select` narrows the subject to the rendered objects its globs
  match. The globs go through `select_objects`, the command line's object
  selection, so a node path selects its subtree. Every placement of a matched
  object joins the subject. A `select` that matches no rendered object
  errors on the view.
- The scene flattens once. Because each view retains its resolved view and
  lights, renders, and releases them, a `camera`-frame light resolves
  against the view it lights and no light carries over.
- `RenderRecord` colors are linear `TyLinSrgbF64`, the contract's form. The
  caller converts its sRGB hex before the record leaves it.
- The image and voxel size checks sit on the record elements `Image` and
  `VoxelSize`, and a run with no view errors on `Views`, as the mesh
  operation reports its primitive table.
- voxsmith's `Error` gains a `Render` variant wrapping voxrender's behind
  the `render` feature. A scene construction failure reports voxrender's
  message.

## S10. vxl command

- The command always loads the profile set, because a run that sets no
  view takes `hero`'s and one that sets no rig takes `studio`'s, and both
  read from the cascade so a config can override them.
- `RenderProfileStack` replaces the stacking function. `--profile` lands a
  profile whole, `--views-from` its views, and `--lights-from` its rig, in
  any order, because the claims catch a collision whichever flag lands
  first. The default view and rig land through the same stack after the
  flags, so they never collide with what a flag set.
- A view's transform is one element. `--view-orbit` sets it whole, and
  `--view-frame`, `--view-position`, and a rotation flag set it together.
  Any of the three claims the element, so a posed view with a part missing
  errors instead of falling back to the profile's transform.
- `--light` declares the rig the way `--primitive` declares the mesh's
  primitives: an index with its kind, numbered from `0` with no gaps, and
  any occurrence replaces the profile's rig whole. Without one the profile's
  rig stands, and the index flags fill its lights.
- A light's kind decides which flags apply to it. A flag its kind never
  reads errors instead of landing in a field the render ignores.
- The Euler flags take degrees. A profile entry can set `rad`.
- A look-at flag takes its target, so `0 0 0` is the frame's origin that a
  profile entry reaches by omitting `target`.
- The default shadow granularity lives in `LightElements::finish`, the one
  place a light with no granularity reads it. S12 sets it.
- Views come out in name order because the table is a map keyed by name,
  so the view ids are stable across runs however the flags are ordered.
- `flag_occurrences`, `parse_flag_index`, and `parse_flag_value` moved from
  the mesh command into the crate's `internal` module, their second caller
  being this command.
- The command shows each view inline through viuer, the crate `tyt fbx
  render` uses: the Kitty or iTerm2 graphics protocol where the terminal
  answers the probe, ANSI half blocks elsewhere, and no probe at all off a
  terminal. `--to png` writes the views beside the input instead, named
  by the input's stem or `--file-stem`, the mesh command's name for the
  stem its templates fill. The user wanted the preview as the default
  because a look should cost no file, the graphics protocols because
  half blocks are coarse, the flag over an output path because a typed
  name misled (with several views no file carried it), and the kind as
  an enum over a bool so a third destination can join. vxl binds viuer
  in its
  `DependenciesImpl` behind a `DisplayImage` trait rather than through
  tyt-injection, so it takes on none of the tyt crates.
- Several views show one below another with a blank line between. A
  composite grid in the style of tyt's `display_images_in_grid` was tried
  and dropped: the user's terminal placed the larger image and drew
  nothing, and the user preferred a vertical strip.

## S11. Golden images

- The fixtures are `test_utilities` functions taking the shadow
  granularity, each retaining its view and lights, so a golden pins
  the scene API end to end. `check_goldens` renders the four variants of
  a fixture and compares each against its PNG, or rewrites the PNGs under
  `VOXRENDER_UPDATE_GOLDENS`.
- The goldens are 64 by 64 and sit in `src/goldens`, beside the tests in
  `render.rs` that embed them. The size keeps the sixteen files under 30
  kilobytes together while a shadow edge still spans several pixels.
- The cube and the slab pair render the same under every variant, because
  neither has a crease and the cube has nothing to shadow. Their goldens
  pin the shading and the placement math. The L-shape and the room carry
  the shadow and occlusion differences.
- voxrender's goldens are plain RGBA PNGs from the `png` dev-dependency.
  The sRGB chunks are voxsmith's encoder's to write, and voxsmith's golden
  goes through `encode_render_png` in that file's tests, so it checks the
  chunk beside the pixels.
- `test_utilities` is gated on `cpu` beside `test`, because the checker
  calls `render` and the fixtures have no other caller.

## S12. Three-shadow renders

- The four assets rendered under `turnaround` and `studio` at 512 by 512
  with `--light-shadow 0` at each granularity, plus a 3x close-up of each
  from the hero orbit, posed in the `subject` frame at a third of the fit
  distance. The renders stayed in a scratch directory.
- The close-ups tell the three apart. `per-pixel` draws the tall pivot's
  shadow across the low one as a crisp diagonal. `per-face` snaps the same
  edge to the voxel grid as a jagged step. `per-corner` spreads it over one
  voxel as a gradient, the way the corner occlusion darkens the creases
  beside it. On the dome of the mixed shapes `per-face` leaves each step's
  shadow blocky where `per-corner` runs it smoothly down the flank. The
  emissive row shows no difference because nothing in it casts a shadow.
- `per-corner` is the pick. It reads as one look with the occlusion. The
  standalone tier computes it exactly, where `per-pixel` needs a shadow map
  that drifts from the reference.
- The pick lives in the record builder's `DEFAULT_SHADOW`, which every
  light without a granularity reads, so `studio` stays bare and takes it.
  The README's schema note on an omitted shadow says `per-corner`.
- The occlusion agrees with `object mesh` by construction. Both call
  voxsurface's corner rule, and S1's test pins the mesh side. The renders
  with and without occlusion show the creases of the reactor's base plate
  darkening under `corner` and flat under `none`. The checklist's look at
  the mesh in a viewer beside the render is still open.

## S13. Docs

- The contract got its own page, `contract.md`, beside the command page,
  because every renderer implements it and the realtime tiers will read it
  without the vxl flags. `render.md` holds the command, its options, and the
  file naming, and `profile-language.md` the schema, the loading, the
  built-ins, and the stacking.
- The reference index links back to the open plan instead of carrying the
  plan the way `doc/ref/mesh` does, because the plan stays open through
  phase 2. The plan's contract and profile sections shrank to pointers under
  their old headings, so the links into them hold.
- The schema marks a view's and a light's `transform` optional because the
  flags can supply it, and the record errors when nothing does. The plan's
  block had it required.
- `voxrender` and `voxsurface` had READMEs from S1 and S2, so the step added
  none. voxsmith's README now says the render shares the mesh bake's PNG
  encoder.
- The vxl-commands README's intro gained rendering, and `vxl object render`
  and `vxl profile object render list` joined its command list.

## S15. Spot lights

- `RenderLight::Spot` carries a position and a rotation beside the point
  light's falloff values and the two cone half-angles in radians. The scene
  checks the cones as glTF does: the inner angle is zero or more and below
  the outer, which is at most a quarter turn. The cone falloff is glTF's
  smooth ramp over the cosines, with glTF's floor of `0.001` under the gap
  between them, which keeps a hard edge finite when two angles sit a
  rounding apart.
- The shader skips the shadow ray of a hit the cone leaves dark, because a
  spot leaves most of a scene dark.
- The spot's shape is a fourth voxsmith enum, `SpotTransform`, over all five
  frames with a numeric orbit distance. Sharing `PoseTransform` with views
  would hand a view the `camera` frame it never reads and a spot the `fit`
  distance it cannot resolve, the silent no-op README decision 18 rules
  out. `pose_in_frame` resolves the posed frames of both shapes through one
  `TyTransformF64`: the identity for `world`, a translation for `subject`,
  the view's pose for `camera`, and the node's world transform for `node`.
- The cone angles are degrees in `LightRecord::Spot` and the profile, and
  turn to radians where the record lowers into the scene, as the field of
  view does.
- `--light-cone` is one element holding both angles. A flag replaces a
  profile's pair whole, where a profile's `innerCone` and `outerCone` fill
  in one at a time. The record builder checks the pair after the defaults
  fill, which makes a profile setting `innerCone` alone at `45` or more an
  error.
- `--light-orbit` on a spot sets the whole pose and clashes with a rotation
  flag as it does with `--light-frame` and `--light-position`. The orbit
  slot holds its three numbers until the kind picks the shape.
- The room fixture's build moved into `lit_room`, which takes the key light,
  so `room_scene` and `spot_room_scene` share the walls, the pillar, the
  fill light, and the view. The spot golden aims the light at the pillar
  from the point light's position at three times its strength, because the
  cone lights the far floor alone and the walls beside the light go dark.
