# Checklist

The design is in the [README](README.md). Check steps off as they land. Log
code-level choices in an `implementation-decisions.md` beside this file. The
resample plan has one.

## Ground rules

- `voxsurface` and `voxrender` depend on `voxcore`, `ty-math`, and
  `branded-id` only. Neither mentions a vxl flag or command. voxsmith fronts
  `voxrender`. vxl speaks voxsmith only.
- Every `voxrender` entity has a `BRender*` brand and a `Render*` type. No
  public API takes or returns a bare list position.
- A value that does not fit errors. A configured part its entity would never
  read is unrepresentable.
- Math that is not about voxels or images goes in `ty-math`: the look-at
  rotation, the spherical direction, and the projection matrices.
- Tests are inline `#[cfg(test)] mod tests` per file. voxsmith tests build
  scenes through `test_utilities`. A golden PNG sits beside the test that
  embeds it with `include_bytes!` and compares per channel within a tolerance.
- A new crate starts at `0.1.0` with `[package.metadata.workspaces]
  independent = true`. It joins the workspace members and `[patch.crates-io]`.

## Phase 1

- [x] **S1. voxsurface split.** Create `projects/utilities/voxsurface`. Move
      `is_solid`, `FaceSpan`, `MeshGeometry`, `Method`, `mesh_slices`,
      `object_to_mesh_geometry`, and the corner rule of `compute_occlusion`
      out of voxsmith's `object_mesh`.
      1. The mesher reads its grid through a trait with a bounds method and a
         solid-at-cell method. `VoxObject` implements it in the crate
      2. Greedy merging keeps its per-cell key and span-fits predicate
      3. The corner occlusion returns plain floats, one per face corner from
         the three adjacent cells
      4. voxsmith re-exports the moved types under their old names. Its
         `merge_rules`, `provenance`, program, records, and writers stay.
         `compute_occlusion` lifts the floats into a value-language array
      5. Tests move with their code. voxsmith's mesh tests pass unchanged
- [x] **S2. voxrender scaffold.** Create `projects/utilities/voxrender` with a
      default `cpu` feature. Add:
      1. The brands `BRenderObject`, `BRenderPlacement`, `BRenderMaterial`,
         `BRenderVoxel`, `BRenderLight`, and `BRenderView`
      2. `RenderMaterial` over the six shaded properties with glTF's defaults
      3. `RenderLight` over the three kinds
      4. `RenderView` over a `TyPoseF64`, a projection, and a field of view
         or an orthographic scale
      5. `RenderOcclusion` and `RenderShadow`
      6. `RenderImage`, a linear RGBA `f32` buffer
- [x] **S3. Render scene.** Add `RenderScene` over `IdVec` and the `soa`
      pools, mutated through `retain_*` and `release_*` with cross-reference
      checks. One constructor flattens a `VoxMain`:
      1. Each selected object yields one effective material per live voxel,
         resolved by the layer-override rule `object mesh` uses and
         deduplicated into the material table
      2. Each node path reaching a selected object yields one placement
         carrying the path's world `TyTransformF64` scaled by the voxel size
      3. An unplaced object gets the identity placement `write_hierarchy`
         gives it

      The flattened object implements `voxsurface`'s grid trait. A subject's
      world bounds come from the eight transformed corners of each of its
      placements.
- [x] **S4. Transforms.**
      1. `ty-math` gains a look-at rotation on `TyQuaternionF64` and a unit
         vector from azimuth and elevation
      2. `voxrender` gains the `fit` rule. The subject's bounding sphere fits
         into the shorter image axis with a small margin, as a distance under
         perspective and as a scale under orthographic
      3. voxsmith gains the three shapes, the four frames, and the four
         rotation forms as tagged unions. It resolves them to `TyPoseF64` in
         the order subject bounds, views, lights. A `camera`-frame light
         resolves once per view

      Angles arrive in degrees and pass through `TyAngleUnit` into radians. A
      look-at whose eye is its target errors. A direction along +Y or -Y
      within `unit_rotation_tolerance` takes -Z as up. The `top` and `bottom`
      built-ins exercise it.
- [x] **S5. DDA.** Rays start at pixel centers. Perspective rays fan out by
      the field of view and aspect. Orthographic rays run parallel across the
      scale. Each ray enters every placement through the inverse placement
      transform, clips to the grid's slab, and steps the dense grid
      Amanatides-Woo style. The nearest hit across placements yields the
      cell, the face axis, and the face coordinates for corner interpolation.
      Tests: a hit on each of a voxel's six faces, a miss, a ray from inside
      a voxel, and the nearer of two placements.
- [x] **S6. Shading.** Shade with glTF's metallic-roughness model in linear
      light: Lambert diffuse, GGX specular with Smith visibility and Schlick
      Fresnel, and the emissive term. The hemisphere term mixes sky and
      ground by the normal's +Y component, scaled by the corner occlusion and
      `occlusionStrength`. A directional light shines down its rotation's -Z.
      A point light attenuates by the inverse square times glTF's smooth
      cutoff at `range`. A shadow is one grid ray toward the light across
      every placement. The ray is infinite for a directional light and ends
      at a point light. `per-pixel` casts it from the hit, `per-face` from
      the face center, and `per-corner` from each corner with a bilinear
      blend across the face. Tests: `studio` gives a cube three shades, a
      wall shadows the floor behind it at each granularity, and a point light
      reaches zero at `range`.
- [x] **S7. Output encoding.** Apply the Khronos PBR Neutral tonemap to
      `RenderImage`, then `ty-math`'s sRGB transfer to 8-bit RGBA with
      straight alpha. A pixel no ray hits is transparent, or the background
      color at full alpha.
- [x] **S8. Profiles.** Add `RenderConfig` under `object.render` and
      `RenderProfile` per the README schema to vxl. The unions tag on `kind`,
      keys are camelCase, and an unknown key errors. Embed the built-ins
      `hero`, `front`, `back`, `left`, `right`, `top`, `bottom`,
      `turnaround`, `studio`, and `flat` as jsonc. Stack `--profile` by the
      mesh rules. Resolve `viewsFrom` and `lightsFrom` imports as the stack
      lands. Add `vxl profile object render list` over voxsmith's
      `profile_list`. The mirroring flags:
      1. `--width`, `--height`, `--background`, `--occlusion`,
         `--voxel-size`, `--views-from`, and `--lights-from`
      2. Per view name: `--view-frame`, `--view-position`,
         `--view-quaternion`, `--view-euler`, `--view-look-at`,
         `--view-angles`, `--view-orbit`, `--view-projection`, `--view-fov`,
         `--view-scale`, and `--view-select`
      3. Per light index: `--light`, `--light-frame`, `--light-position`,
         the four rotation flags, `--light-orbit`, `--light-shadow`,
         `--light-color`, `--light-strength`, `--light-range`,
         `--light-sky`, and `--light-ground`
- [x] **S9. voxsmith operation.** Add `object_render` under
      `operations/object` behind a `render` feature that pulls `voxrender`
      in. `RenderRecord` carries the image elements,
      `views: IdVec<BRenderView, ViewRecord>` with each view's name, and
      `lights: IdVec<BRenderLight, LightRecord>`. Its transforms stay in
      their configured shapes. `render` takes the `VoxMain`, the selected
      object ids, and the record. It flattens, resolves, calls `voxrender`
      once per view, and returns the image per view id beside the resolved
      view. `encode_render_png` encodes an image through `EncodePng` with
      the sRGB transfer stamped. These error:
      1. An empty selection
      2. A duplicate or unknown object
      3. A zero image side
      4. A `select` matching nothing
      5. A run with no view
- [x] **S10. vxl command.** Add `object render` under `commands/object` with
      the `flags`, `profile`, `record`, and `run` layout of `object mesh`. It
      flattens the `ObjectSelection` group. The views show inline in the
      terminal, or `--to png` writes them beside the input as `<stem>.png`,
      several views as `<stem>-<view>.png`, the stem the input's or
      `--file-stem`. The record
      builder turns a `<light-index>` into a `U32Id<BRenderLight>` and a view
      name into a `U32Id<BRenderView>`. `--print-camera` prints each rendered
      view's resolved pose as its `--view-frame <name> world`,
      `--view-position`, and `--view-quaternion` flags on one line. Parse
      tests per flag family.
- [x] **S11. Golden images.** Build fixture scenes in `voxrender` through the
      scene API: one cube, an L-shape that shadows itself, two placements of
      one object, and a point-lit room. Each fixture gets one PNG per shadow
      granularity plus one with `occlusion` off. The PNGs embed beside the
      test, decode through a `png` dev-dependency, and compare per channel
      within a tolerance of two. An environment variable rewrites them.
      voxsmith gets one end-to-end golden over `two_material_scene` under
      `hero` and `studio` to pin the PNG encoding.
- [x] **S12. Three-shadow renders.** Render `energy-reactor`,
      `mixed-shapes-with-material`, `emissive`, and `pivots` from
      `submodules/tyt-assets/src/vmax` under `turnaround` and `studio` at
      each granularity. Open each object's `object mesh` output beside its
      render and check the occlusion agrees. Pick a granularity, write it
      into `studio`, and record it in README decision 6. The renders stay out
      of the repository.
- [x] **S13. Docs.** Move the contract and the profile schema to
      `doc/ref/render`, shaped like `doc/ref/mesh`. Link the README's
      contract section to it. Add a rendering paragraph to the vxl README's
      Objects section. Add `object render` to the vxl-commands README command
      list and its object-selectors convention. Give each new crate a README.

## Phase 2

- [x] **S14. The `node` frame.** A fifth frame every shape takes: the values
      are read in one hierarchy node path's world transform, so a camera can
      ride a player. The entry carries a `path`, a glob over node paths with
      the object selectors' rules. The glob must match exactly one path. Zero
      or several matches error, listing the paths that matched. The position
      and rotation compose with that path's world transform, scale included.
      That transform is the one the flatten gives the path's placements, with
      the voxel size applied.
      1. voxsmith gains a walk over node paths that yields each path's world
         transform beside the path. Every shape's frames gain `node`,
         resolved through that walk
      2. vxl gains `--view-node <view> <path>` and
         `--light-node <light-index> <path>`. Each is read only under the
         `node` frame: one given under another frame errors, as does a
         `node` frame given no path
      3. A profile entry takes `{ "kind": "node", "path": ... }` with the
         position and rotation of its shape
      4. The contract's transforms list the frame, the render page gains the
         two flags, and the profile language's schema gains the kind
      5. Tests: a view riding a translated, rotated, and scaled node, a glob
         matching two paths, and each flag error
- [ ] **S15. Spot lights.** A fourth light kind. A `spot` light is a pose:
      it sits at a position and shines down its rotation's -Z. It falls off
      as a point light does, times glTF's cone falloff between an inner and
      an outer cone angle in degrees, `0` and `45` by default. The inner
      angle stays below the outer, which is at most `90`. The shadow ray
      ends at the light.
      1. `voxrender` gains `RenderLight::Spot` over a position, a rotation,
         the color, the strength, the range, the two cone angles, and the
         shadow granularity
      2. voxsmith's `LightRecord::Spot` takes the pose shape a view takes
         plus the `camera` frame, so a spot can ride the camera as a
         headlight. `orbit` sets the whole pose: a point on the sphere about
         the subject's center, facing the center
      3. vxl's `--light <light-index> spot` takes `--light-frame`,
         `--light-position`, and one rotation flag, or `--light-orbit`. vxl
         gains `--light-cone <light-index> <inner> <outer>`, one element in
         degrees. `--light-cone` on another kind errors, as does a reversed
         or over-wide pair. `--light-range` applies as it does to a point
         light
      4. A profile entry is `{ "kind": "spot" }` with a `transform`,
         `innerCone`, `outerCone`, `range`, `shadow`, `color`, and
         `strength`
      5. The contract's lights list the kind, the render page gains the
         flag, and the profile language's schema gains the entry
      6. Tests: a spot over a floor lights a disc that fades between the
         cones and nothing past the outer, the shadow stops at the light,
         and the point-lit room fixture gets a spot golden
- [ ] **S16. Bloom.** A post pass over the linear image before the tonemap,
      fed by the emissive term alone, so a key light never makes a white
      face glow. A material's `emissiveStrength` says how hard the material
      glows. The profile says how the lens responds, through three scalars.
      Each is a profile element mirroring one flag and stacks as `occlusion`
      does.
      1. `bloomStrength` scales the halo. `0`, the default, skips the pass
      2. `bloomRadius` sets the halo's reach as a fraction of the shorter
         image side, `0.03` by default, which makes a small render glow
         like a large one
      3. `bloomThreshold` is the luminance the emission must exceed, in
         linear light, `1` by default. At `1`, a material at glTF's default
         `emissiveStrength` stays flat and one pushed above it glows
      4. The shader keeps each hit pixel's emission in a second buffer. The
         pass takes the part over the threshold, hue preserved, blurs it by
         a Gaussian per octave from the radius halving to one pixel,
         averages the octaves, scales by the strength, and adds the halo to
         every pixel. A miss pixel the halo reaches takes the halo's peak
         channel, clamped to one, as its alpha and the halo's hue as its
         color. The output composites such a pixel over the background
         color, or keeps it under `transparent`
      5. `voxrender` gains `RenderBloom` over the three values, and `render`
         takes it
      6. voxsmith's `RenderRecord` gains `bloom`, checked with the other
         scalars: a finite non-negative strength and threshold, and a finite
         positive radius
      7. vxl gains `--bloom-strength`, `--bloom-radius`, and
         `--bloom-threshold`, the three profile keys, and the `glow` built-in
         at strength `1`, radius `0.03`, and threshold `1`. `glow` stacks
         over any rig
      8. The contract gains a bloom section before Output, the render page
         the three entries, and the profile language the keys and the
         built-in
      9. Tests: a fixture with an emissive strip at strength `4` gets a
         `glow` golden, strength `0` leaves the image byte-identical, a
         threshold above the emission leaves the image unchanged, and the halo
         lands on transparent pixels past the silhouette with alpha
      10. Render `emissive` from `submodules/tyt-assets/src/vmax` under
          `glow`, tune the built-in's values, and record them in README
          decision 21
