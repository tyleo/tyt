# Voxel rendering follow-ups

Status: **open.** The
[voxel rendering plan](../../closed/voxel-rendering/README.md) closed on
2026-10-01 with the contract, the CPU reference, and `vxl object render`
built. This plan lists what it left open. A part gets its design, its
checklist, and its decisions log when it starts.

## Transparency

Alpha, `transmission`, and `ior`. Built as the
[voxel transparency plan](../../closed/voxel-transparency/README.md), closed
2026-10-04 with the reference, the mesher, and the `glass` profile done.

## `voxrender-wgpu`

The realtime tiers over the same render scene, drawing into a texture the
caller owns. Opened 2026-10-04 as the
[voxrender-wgpu plan](../voxrender-wgpu/README.md). It revisits the
[Realtime tiers](../../closed/voxel-rendering/README.md#realtime-tiers) of the
closed rendering plan with the VR experience first.

## Glass

Back faces, Fresnel by angle, and a transparent PNG that keeps glass's tint.
Built as the [voxel glass plan](../../closed/voxel-glass/README.md), closed
2026-10-04 with the reference and the `glass` profile done.

## Contact sheet

Several views tiled into one image. Built in step S17 of the
[voxel modeling checklist](../voxel-modeling/checklist.md) as the `sheet` image
kind.

## Reflections

Mirror reflections, for
[finding 20](../voxel-modeling/trials-round-6.md#findings) of the voxel
modeling trials. A flat metal face misses the key light's highlight and takes
the hemisphere light at its normal. A face toward the camera sits on the
horizon between the white sky and the #404040 ground, where gold reads olive at
half brightness.

A reflective face can trace a ray along the mirror direction and show what the
ray hits, or the hemisphere where the ray leaves the scene. A face toward the
camera still reflects the horizon, so `studio` also needs a backdrop that stays
bright down to it. Area lights could instead give a flat metal face a broad
highlight. The plan weighs the two answers and settles how a rough surface blurs
a reflection.

## Waiting for a reason

1. Traced occlusion as a third `occlusion` value
2. Area lights
