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

Mirror reflections, for findings 5 and 20 of the voxel modeling
[trials](../voxel-modeling/trials-round-6.md#findings). Built in step S34 of the
[voxel modeling checklist](../voxel-modeling/checklist.md), which gives
`studio` a light ground for a metal's sides to reflect.

## Waiting for a reason

1. Traced occlusion as a third `occlusion` value
2. Area lights
3. A tonemap that keeps a dark color's hue. The flare offset subtracts the same
   amount from every channel and turns a dark brown redder
4. A backdrop that shows through openings in place of the white background
5. Light cast from emissive materials onto nearby surfaces
6. A clip plane that opens a room or a hull at render time
