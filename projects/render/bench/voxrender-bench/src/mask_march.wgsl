// A march through a tile's bits in a box raster's fragments. A fragment
// steps its ray a cell at a time, from the point the rasterizer gives it on
// its box.
//
// `front` and `inside` discard a miss and keep their face's depth, which
// lets the depth test run before the march. `exact` writes the hit's depth.
// `solid` marches nothing.
//
// `inside` shades the back faces of a box a ray can start inside. A hit in
// such a box lies before the box's back face. The front face of the box
// behind can share that face's plane. A vertex entry for `inside` scales its
// depth by `NEARER`, which keeps the hit ahead of that box's.
//
// The WGSL that includes this file defines:
//
// 1. `Box`, the fragment's inputs: `clip`, its position. `point`, its point
//    in the tile's cells. `ray`, a perspective view's eye from the tile's low
//    corner with w = 1, or the reverse of an orthographic view's direction
//    with w = 0. `low`, the box's low cell in the tile and then the offset of
//    the tile's words in `bits`. `high`, the corner past the box's high cell.
//    `fall`, the depth an orthographic ray loses over a unit of its direction
// 2. `bits`, whose words hold a tile's cells as `TileMasks` lays them out
// 3. `surface`, what a fragment writes for a hit cell, given the one axis of
//    the face the ray crossed into it and the axes the ray moves up

// The most cells a ray crosses in a tile.
const STEPS: i32 = 48;

// What a perspective view's reversed depth scales by to come a part in 1024
// nearer.
const NEARER: f32 = 1.0 + 1.0 / 1024.0;

struct Fragment {
    @location(0) hit: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

struct Hit {
    found: bool,

    // What `surface` gives the hit.
    color: vec4<f32>,

    depth: f32,
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
) -> Hit {
    let low = vec3<i32>(box.low.xyz);
    let high = vec3<i32>(box.high.xyz);
    let ahead = direction >= vec3<f32>(0.0);

    // The ray's units per cell on each axis.
    let rate = 1.0 / max(abs(direction), vec3<f32>(1.0e-12));

    let cell = clamp(vec3<i32>(floor(start)), low, high - vec3<i32>(1));

    // The cell's bit in the tile, and what a step along each axis adds to
    // it.
    var bit = (cell.x * 16 + cell.y) * 16 + cell.z;
    let strides = select(vec3<i32>(-256, -16, -1), vec3<i32>(256, 16, 1), ahead);

    // How far along the ray each axis's next cell face lies.
    var next = abs(vec3<f32>(cell) + select(vec3<f32>(0.0), vec3<f32>(1.0), ahead) - start) * rate;

    // How far along the ray the box ends, short by more than the sums of
    // `next` can drift. A step that ends before it stays in the box.
    let ends = abs(vec3<f32>(select(low, high, ahead)) - start) * rate;
    let end = min(ends.x, min(ends.y, ends.z)) * 0.99999;

    // The axis of the face the ray last crossed. An entering ray starts on
    // the face that holds its start.
    var across = least(abs(start - vec3<f32>(select(high, low, ahead))));
    var along = 0.0;
    var free = enters;

    for (var n = 0; n < STEPS; n++) {
        let live = ((bits[box.low.w + (u32(bit) >> 5u)] >> (u32(bit) & 31u)) & 1u) == 1u;

        if live && free {
            let at = vec3<i32>(bit >> 8u, (bit >> 4u) & 15, bit & 15);
            let past = along - before;

            return Hit(
                true,
                surface(box, at, across, ahead),
                (box.clip.z - past * box.fall) / (1.0 + past * box.ray.w),
            );
        }

        free = free || !live;
        across = least(next);
        along = min(next.x, min(next.y, next.z));

        if along >= end {
            break;
        }

        next += select(vec3<f32>(0.0), rate, across);
        bit += select(select(strides.z, strides.y, across.y), strides.x, across.x);
    }

    return Hit(false, vec4<f32>(0.0), 0.0);
}

// The first hit of the ray that leaves the box at the rasterizer's point.
// The ray starts where it enters the box, or at its origin when that lies
// inside the box.
fn hit_from_back(box: Box) -> Hit {
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

// A front face's fragment. The rasterizer's point is where the ray enters
// the box.
@fragment
fn front(box: Box) -> @location(0) vec4<f32> {
    let hit = first_hit(box, box.point, direction_of(box), 0.0, true);

    if !hit.found {
        discard;
    }

    return hit.color;
}

// A back face's fragment of a box a ray can start inside.
@fragment
fn inside(box: Box) -> @location(0) vec4<f32> {
    let hit = hit_from_back(box);

    if !hit.found {
        discard;
    }

    return hit.color;
}

// A back face's fragment that writes its hit's depth. A depth of zero fails
// the depth test, so a miss writes nothing.
@fragment
fn exact(box: Box) -> Fragment {
    let hit = hit_from_back(box);

    return Fragment(hit.color, hit.depth);
}

// A solid box's fragment. The rasterizer's point lies on a live cell's face.
@fragment
fn solid(box: Box) -> @location(0) vec4<f32> {
    let low = vec3<i32>(box.low.xyz);
    let high = vec3<i32>(box.high.xyz);
    let ahead = direction_of(box) >= vec3<f32>(0.0);
    let across = least(abs(box.point - vec3<f32>(select(high, low, ahead))));
    let cell = clamp(vec3<i32>(floor(box.point)), low, high - vec3<i32>(1));

    return surface(box, cell, across, ahead);
}
