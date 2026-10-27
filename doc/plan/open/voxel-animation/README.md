# Voxel animation

Status: **open**, drafted 2026-10-19. Nothing is designed yet. This page sets
the goal and the model, and collects the questions to settle before a design.

## Goal

An agent hears how an animation looks, writes it into the model file, and
builds it into a `.voxj` that carries the clips. A test setup then plays it.
The realtime renderer is the target, so the model suits voxrender first and
glTF export second. The CPU reference poses a clip at a time for the agent's
review stills.

The work splits into three tracks:

1. [voxj and voxcore](#voxj-and-voxcore): the animation model and its wire
   format
2. [Model files and sdfj](#model-files-and-sdfj): clips written in
   TypeScript, carried through `vxl sdf-doc build` and `voxelize`
3. [Rendering](#rendering): clips played in `voxrender-wgpu` and posed in the
   CPU reference

## Model

An animation is a clip: a name and one timeline holding three typed track
lists.

1. **Rigid tracks** drive a node's translation, rotation, or scale. Each
   moving piece is its own voxel object. Playback moves placements and never
   touches voxels. Most motion rides rigid tracks
2. **Flipbook tracks** pick which of a node's child objects shows. Each frame
   is its own object. Fire, water, and squash and stretch take this kind
3. **Palette tracks** drive a material property, such as a pulsing
   `emissiveStrength`

Time counts seconds. A rigid or palette track interpolates by glTF's `STEP`,
`LINEAR`, or `CUBICSPLINE`. `STEP` gives the held-frame look. A flipbook track
always steps. A node's transform sets its rest pose. A playing track replaces
the property it drives.

## voxj and voxcore

voxj gains an `animations` list of clips. Times, interpolation names, and
quaternion rotations follow glTF's. Keys sit in plain JSON arrays rather than
glTF's binary accessors:

```jsonc
{
  "animations": [
    {
      "name": "attack",

      "rigidTracks": [
        {
          "node": 4,
          "path": "rotation",
          "interpolation": "LINEAR",
          "times": [0, 0.2],
          "values": [
            [0, 0, 0, 1],
            [0, 0, 0.38, 0.92],
          ],
        },
      ],

      "flipbookTracks": [
        {
          "node": 7,
          "times": [0, 0.1, 0.2],
          "frames": [0, 1, 2],
        },
      ],

      "paletteTracks": [],
    },
  ],
}
```

A flipbook frame indexes the node's `childObjects`. voxcore holds clips
natively instead of in a format's ext because meshing, rendering, and several
bridges all read them. Tracks reference nodes by branded id.

Bridges:

1. mvox: MagicaVoxel's transform-node keyframes map to rigid tracks and its
   per-frame shape models to flipbook tracks. Both sit in `MVoxExt` today and
   promote to clips
2. vmax: `animations.vmaxa` is stored opaque. Reading it needs reverse
   engineering
3. `vxl object mesh`: rigid tracks write as glTF channels. A flipbook track
   writes one node per frame with step-keyed `KHR_node_visibility`. Palette
   tracks write through `KHR_animation_pointer`. `meshdoc` has no animation
   model yet. `gltf-meshdoc` only keeps a read file's animations as ext. A
   glTF import sorts each channel into a track list by its target path

## Model files and sdfj

The rig exists. Each [part](../../../../projects/sdf/sdfj-builder/docs/typescript-api.md#parts)
writes its own object under a node at its pivot, nested as the parts nest. A
part voxelizes unrotated and its node takes the rotation, so a clip keyed on
part nodes never re-voxelizes.

sdfj takes the same `animations` list. Its tracks target sdfj nodes and
materials. `voxelize` carries each track onto the voxj node a part writes or
the palette materials an sdfj material writes. Each track kind differs from
voxj's in one way:

1. **Rigid tracks** key rotations in the `[axis, degrees]` form parts use.
   `voxelize` turns each key into a quaternion for glTF's shortest-path slerp.
   Two keys more than 180 degrees apart turn the short way. A full spin needs
   keys between
2. **Flipbook tracks** pick among an sdfj node's `childObjects`. A flipbook
   part takes one step list per frame and writes an object for each. Every
   other part writes one object
3. **Palette tracks** target an sdfj material. A material that `shades`
   varies writes several palette materials. The track drives every one

The agent loop extends the `vxl-model` skill: write the clips, build, render
stills at chosen times, revise.

## Rendering

The [render contract](../../../ref/render/contract.md) keeps describing one
image of one scene. A sampler turns a clip and a time into node poses, frame
choices, and material values. One sampler feeds every renderer through
`RenderScene::set_placement_transform` and `set_voxel_material`.
`RenderScene` gains a placement visibility flag. A flipbook frame change then
keeps every placement id.

1. `vxl object render` poses a clip at a time for review stills. Animated
   goldens are such stills
2. `voxrender-wgpu` samples once per frame. It holds only the GPU walk because
   its design settles after B3. Its plan takes
   [moving scenes](../voxrender-wgpu/README.md#moving-scenes) from any
   source. Its benchmark scene S6 scripts this plan's three track kinds
   without a clip format

S6 measures four costs animation adds to the GPU tiers:

1. **Lighting.** A moving part invalidates its faces' shadows and the
   light-space tiles under it. The frame refills them (research 3.8, item 8).
   A crowd lands on the over-budget fallback to per-face shadows
2. **Overlap.** A rig holds many small placements that overlap at the joints.
   Overlap is the remaining top-level cost (research 3.5)
3. **Pass changes.** A palette track that moves a material's pass across zero
   moves its placements between the opaque and transparent paths (research
   3.9) and changes corner occlusion
4. **Flipbook memory.** Every frame stays resident in the brick pool. A frame
   change invalidates light tiles as an edit does

## Design notes

1. No skinning. A skin bends cells out of the axis-aligned grids voxrender
   walks. A skin would also need SpaceWarp's motion vectors per vertex
   instead of per placement
2. No morph targets. Morph targets need one topology across frames, and
   re-meshing changes the topology
3. Typed track lists inside one clip, not one top-level list per kind. A clip
   is what the engine plays, blends, and loops. One action often uses every
   kind on one timeline, and three top-level lists would join that clip only
   by name. The types still keep an interpolated flipbook unwritable
4. Flipbooks get their own track rather than glTF's visibility-keyed sibling
   nodes. The glTF writer makes the sibling nodes at export
5. Seconds over integer frames, because the engine samples at 90 Hz in VR
6. sdfj rotations interpolate as voxj's do. The two documents then agree on
   every pose between keys

## Questions

1. **The test setup.** No realtime viewer exists. `voxrender-wgpu` has no
   window. The candidates are a desktop viewer app over `voxrender-wgpu`, a
   `.glb` played in an existing glTF viewer until that app exists, and the
   Quest bench app
2. **Pass crossings.** Either a palette track can move a material's pass
   across zero and the renderer reclassifies its placements each frame, or
   the track errors
3. **The model-file API.** How a clip refers to its parts and materials, and
   the call that makes a flipbook part
4. **The rest frame.** Which child object a flipbook node shows while no clip
   plays
5. **Material track targets.** A track needs a path vocabulary for voxj
   materials, which live in palettes over value pools and layers
6. **A released node under a track.** The release errors, as an object
   release does while an mvox shape entry draws it, or the track drops
7. **Where the sampler lives.** voxcore, voxrender, or its own crate
8. **vxl commands.** Whether clips need `vxl animation` commands to list,
   show, and edit them, or arrive only through model files and imports
