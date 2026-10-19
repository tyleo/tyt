# voxrender-wgpu

The realtime GPU renderer for `voxrender`'s render contract, through wgpu. It
runs on a device its caller opens.

## The walk

`trace_hits` walks each pixel's ray through every placement's grid in a
compute shader and reads back each pixel's first hit as a `PixelHit`: the
placement, the cell, the face, and the face's corner shadow bits for each
light that casts shadows. `GpuWalk` picks the traversal. `GpuWalk::Integer` runs
`voxrender`'s integer DDA in i32 over the shared quantized rays and matches
`voxrender-cpu`'s hits. `GpuWalk::Float` runs an f32 DDA over the same inputs.

```rust
let hits = trace_hits(&device, &queue, &scene, view_id, 256, GpuWalk::Integer)?;
```
