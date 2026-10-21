# voxrender-wgpu

The realtime GPU renderer for `voxrender`'s render contract, through wgpu. It
runs on a device its caller opens.

## The walk

`HitTrace` uploads a scene and one view's rays, then walks each pixel's ray
through every placement's grid in a compute shader. `dispatch` records the
walk into a caller's pass. `trace` walks once and reads back each pixel's
first hit as a `PixelHit`: the placement, the cell, the face, and the face's
corner shadow bits for each light that casts shadows.

`HitTrace::new` runs `voxrender`'s integer DDA in i32 over the shared
quantized rays. Its hits match `voxrender-cpu`'s. `HitTrace::with_walk`
takes another walk in WGSL and the words it reads per object, so a
benchmark can trace a candidate through the same harness. A benchmark's own
shader can walk the same placements over `GRID_WALK_WGSL`, the walks' shared
types and helpers, and the bytes `placement_bytes` packs.

```rust
let trace = HitTrace::new(&device, &scene, view_id, 256, 256)?;
let hits = trace.trace(&device, &queue)?;
```
