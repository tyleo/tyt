// A raster of the boxes around each tile's live cells over `TileMasks`. A
// fragment starts at the point the rasterizer gives it on its box and steps
// its ray a cell at a time through the tile's bits. It writes the first hit
// and its depth, so the depth test keeps the nearest hit of every box the
// ray crosses. A hit packs as `raster.wgsl`'s does. `BoxRaster` prepends
// `FACE_BITS`.

// The most cells a ray crosses in a tile.
const STEPS: i32 = 48;

struct Frame {
    // The low bits of a packed hit that hold its voxel id.
    voxel_bits: u32,
}

struct Place {
    // The placement's grid-to-clip matrix.
    clip: mat4x4<f32>,

    // A perspective view's eye in the grid with w = 1, or the reverse of an
    // orthographic view's direction in the grid with w = 0.
    ray: vec4<f32>,

    // What a step along x and a step along y add to a voxel id.
    strides: vec4<u32>,
}

struct Box {
    @builtin(position) clip: vec4<f32>,

    // The point in the tile's cells.
    @location(0) point: vec3<f32>,

    // `Place::ray` from the tile's low corner.
    @location(1) @interpolate(flat) ray: vec4<f32>,

    // The box's low cell in the tile, then the offset of the tile's bits.
    @location(2) @interpolate(flat) low: vec4<u32>,

    // The corner past the box's high cell in the tile, then the placement's
    // index above the face's bits.
    @location(3) @interpolate(flat) high: vec4<u32>,

    // The voxel id of the tile's low cell, then `Place::strides`.
    @location(4) @interpolate(flat) voxel: vec3<u32>,

    // The depth an orthographic ray loses over a unit of its direction.
    @location(5) @interpolate(flat) fall: f32,
}

struct Fragment {
    @location(0) hit: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<storage, read> places: array<Place>;
// Per object: its `TileMasks` words.
@group(0) @binding(2) var<storage, read> bits: array<u32>;

// Bit `a` of `corner` picks the box's high face on axis `a`. `low` holds the
// box's low cell and its placement's index. `high` holds the corner past the
// box's high cell and the offset of its tile's bits.
@vertex
fn place(
    @location(0) corner: u32,
    @location(1) low: vec4<u32>,
    @location(2) high: vec4<u32>,
) -> Box {
    let far = ((vec3<u32>(corner) >> vec3<u32>(0u, 1u, 2u)) & vec3<u32>(1u)) == vec3<u32>(1u);
    let grid = select(low.xyz, high.xyz, far);
    let tile = low.xyz & vec3<u32>(0xfffffff0u);
    let placed = places[low.w];
    let strides = placed.strides.xy;

    return Box(
        placed.clip * vec4<f32>(vec3<f32>(grid), 1.0),
        vec3<f32>(grid - tile),
        vec4<f32>(placed.ray.xyz - vec3<f32>(tile) * placed.ray.w, placed.ray.w),
        vec4<u32>(low.xyz - tile, high.w),
        vec4<u32>(high.xyz - tile, low.w << FACE_BITS),
        vec3<u32>(tile.x * strides.x + tile.y * strides.y + tile.z, strides),
        (placed.clip * vec4<f32>(placed.ray.xyz, 0.0)).z,
    );
}

// The one axis of the least component of `v`. A tie goes to the earlier
// axis.
fn least(v: vec3<f32>) -> vec3<bool> {
    let x = v.x <= v.y && v.x <= v.z;
    let y = !x && v.y <= v.z;

    return vec3<bool>(x, y, !x && !y);
}

// The ray's direction through `box` at the rasterizer's point.
fn direction_of(box: Box) -> vec3<f32> {
    return box.point * box.ray.w - box.ray.xyz;
}

// The first hit of the ray from `start` in the tile's cells along
// `direction`. `before` is the rasterizer's point along the ray, in units of
// `direction` from `start`. A ray that `enters` starts on a face of the box.
// Any other ray starts inside the box, and live cells it starts in are no
// hit.
fn first_hit(
    box: Box,
    start: vec3<f32>,
    direction: vec3<f32>,
    before: f32,
    enters: bool,
) -> Fragment {
    let low = vec3<i32>(box.low.xyz);
    let high = vec3<i32>(box.high.xyz);
    let ahead = direction >= vec3<f32>(0.0);
    let step = select(vec3<i32>(-1), vec3<i32>(1), ahead);

    // The ray's units per cell on each axis.
    let rate = 1.0 / max(abs(direction), vec3<f32>(1.0e-12));

    var cell = clamp(vec3<i32>(floor(start)), low, high - vec3<i32>(1));

    // How far along the ray each axis's next cell face lies.
    var next = abs(vec3<f32>(cell) + select(vec3<f32>(0.0), vec3<f32>(1.0), ahead) - start) * rate;

    // The axis of the face the ray last crossed. An entering ray starts on
    // the face that holds its start.
    var across = least(abs(start - vec3<f32>(select(high, low, ahead))));
    var along = 0.0;
    var free = enters;

    for (var n = 0; n < STEPS; n++) {
        let column = bits[box.low.w + (u32(cell.x) << 3u) + (u32(cell.y) >> 1u)];
        let live = ((column >> (((u32(cell.y) & 1u) << 4u) + u32(cell.z))) & 1u) == 1u;

        if live && free {
            let axis = select(select(0u, 1u, across.y), 2u, across.z);

            // The face's normal points up its axis where the ray moves down
            // the axis.
            let face = axis * 2u + select(1u, 0u, any(across & ahead));
            let voxel = box.voxel.x + u32(cell.x) * box.voxel.y + u32(cell.y) * box.voxel.z
                + u32(cell.z);
            let past = along - before;

            return Fragment(
                unpack4x8unorm(voxel | ((box.high.w | face) << frame.voxel_bits)),
                (box.clip.z - past * box.fall) / (1.0 + past * box.ray.w),
            );
        }

        free = free || !live;
        across = least(next);
        along = dot(next, vec3<f32>(across));
        next += select(vec3<f32>(0.0), rate, across);
        cell += select(vec3<i32>(0), step, across);

        if any(cell < low) || any(cell >= high) {
            break;
        }
    }

    // A depth of zero fails the depth test, so a miss writes nothing.
    return Fragment(vec4<f32>(0.0), 0.0);
}

// The rasterizer's point is where the ray leaves the box. The ray starts
// where it enters the box, or at its origin when that lies inside the box.
@fragment
fn back(box: Box) -> Fragment {
    let direction = direction_of(box);
    let ahead = direction >= vec3<f32>(0.0);
    let rate = 1.0 / max(abs(direction), vec3<f32>(1.0e-12));

    // How far back along the ray each axis's entry face lies, then the ray's
    // origin.
    let faces = abs(box.point - vec3<f32>(select(box.high.xyz, box.low.xyz, ahead))) * rate;
    let entry = min(faces.x, min(faces.y, faces.z));
    let origin = (1.0 - box.clip.z * (1.0 - box.ray.w)) / (box.fall + box.ray.w);
    let before = min(entry, origin);

    return first_hit(box, box.point - direction * before, direction, before, entry <= origin);
}
