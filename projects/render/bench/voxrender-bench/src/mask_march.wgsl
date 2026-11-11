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
// 1. `RAY_W`, 1 under a perspective view and 0 under an orthographic one
// 2. `Box`, the fragment's inputs: `clip`, its position. `point`, its point
//    in the tile's cells. `ray`, a perspective view's eye from the tile's low
//    corner, or the reverse of an orthographic view's direction. `low`, the
//    box's low cell in the tile. `high`, the corner past the box's high
//    cell. `words`, whose x holds the offset of the tile's words in `bits`.
//    `fall`, the depth an orthographic ray loses over a unit of its
//    direction. `face`, the axis of the fragment's face, plus 4 on the box's
//    high face. `BoxDraw`'s triangles carry it from the corner each face
//    starts at
// 3. `bits`, whose words hold a tile's cells as `TileMasks` lays them out
// 4. `surface`, what a fragment writes for a hit cell, given the one axis of
//    the face the ray crossed into it and the axes the ray moves up
//
// Its vertex entries place a corner only when `sound` holds for the corner's
// position and its `ray`.

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

// Whether each component of `a` and of `b` is a finite number. The test
// reads bits, which a compiler that takes every number as finite leaves
// alone.
fn sound(a: vec4<f32>, b: vec4<f32>) -> bool {
    let top = vec4<u32>(0x7f800000u);

    return all((bitcast<vec4<u32>>(a) & top) != top) && all((bitcast<vec4<u32>>(b) & top) != top);
}

// The one axis of the least component of `v`. A tie goes to the earlier
// axis.
fn least(v: vec3<f32>) -> vec3<bool> {
    let x = v.x <= v.y && v.x <= v.z;
    let y = !x && v.y <= v.z;

    return vec3<bool>(x, y, !x && !y);
}

// A ray's place among a tile's cells.
struct March {
    // The cell's bit in the tile.
    bit: i32,

    // How far along the ray each axis's next cell face lies.
    next: vec3<f32>,

    // The axis of the face the ray last crossed, and how far along the ray
    // that face lies.
    across: vec3<bool>,
    along: f32,
}

// Whether the cell at `bit` of the box's tile is live.
fn live(box: Box, bit: i32) -> bool {
    return ((bits[box.words.x + (u32(bit) >> 5u)] >> (u32(bit) & 31u)) & 1u) == 1u;
}

// Moves `march` a cell along its ray. `rate` holds the ray's units per cell
// on each axis. `strides` holds what a step along each axis adds to a bit.
// Returns false once the ray has passed `end`.
//
// A march's loops count no steps. This test ends them, because each step
// moves a finite ray on by a cell. `sound` keeps any other ray from a
// fragment.
fn advance(march: ptr<function, March>, rate: vec3<f32>, strides: vec3<i32>, end: f32) -> bool {
    let next = (*march).next;
    let across = least(next);
    let along = min(next.x, min(next.y, next.z));

    (*march).across = across;
    (*march).along = along;
    (*march).next = next + select(vec3<f32>(0.0), rate, across);
    (*march).bit += select(select(strides.z, strides.y, across.y), strides.x, across.x);

    return along < end;
}

// The ray's direction through `box` at the rasterizer's point.
fn direction_of(box: Box) -> vec3<f32> {
    return box.point * RAY_W - box.ray;
}

// The first hit of the ray from `start` in the tile's cells along
// `direction`. `before` is the rasterizer's point along the ray, in units of
// `direction` from `start`. A ray that `enters` starts on the face of the
// box across the one axis of `entry`. Any other ray starts inside the box,
// and live cells it starts in are no hit.
fn first_hit(
    box: Box,
    start: vec3<f32>,
    direction: vec3<f32>,
    before: f32,
    enters: bool,
    entry: vec3<bool>,
) -> Hit {
    let ahead = direction >= vec3<f32>(0.0);

    // The ray's units per cell on each axis.
    let rate = 1.0 / max(abs(direction), vec3<f32>(1.0e-12));

    let cell = clamp(floor(start), box.low, box.high - 1.0);

    // The cell's bit in the tile, and what a step along each axis adds to
    // it.
    let bit = i32(dot(cell, vec3<f32>(256.0, 16.0, 1.0)));
    let strides = select(vec3<i32>(-256, -16, -1), vec3<i32>(256, 16, 1), ahead);

    // How far along the ray each axis's next cell face lies.
    let within = start - cell;
    let next = abs(select(within, 1.0 - within, ahead)) * rate;

    // How far along the ray the box ends, short by more than the sums of
    // `next` can drift. A step that ends before it stays in the box.
    let ends = abs(select(box.low, box.high, ahead) - start) * rate;
    let end = min(ends.x, min(ends.y, ends.z)) * 0.99999;

    var march = March(bit, next, entry, 0.0);

    // A ray that starts inside the box first leaves the live cells it
    // starts in.
    if !enters {
        loop {
            if !live(box, march.bit) {
                break;
            }

            if !advance(&march, rate, strides, end) {
                return Hit(false, vec4<f32>(0.0), 0.0);
            }
        }
    }

    loop {
        if live(box, march.bit) {
            let at = vec3<i32>(march.bit >> 8u, (march.bit >> 4u) & 15, march.bit & 15);
            let past = march.along - before;

            return Hit(
                true,
                surface(box, at, march.across, ahead),
                (box.clip.z - past * box.fall * (1.0 - RAY_W)) / (1.0 + past * RAY_W),
            );
        }

        if !advance(&march, rate, strides, end) {
            break;
        }
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
    let faces = abs(box.point - select(box.high, box.low, ahead)) * rate;
    let entry = min(faces.x, min(faces.y, faces.z));
    let origin = (1.0 - box.clip.z * (1.0 - RAY_W)) / (box.fall * (1.0 - RAY_W) + RAY_W);
    let before = min(entry, origin);

    return first_hit(
        box,
        box.point - direction * before,
        direction,
        before,
        entry <= origin,
        least(faces),
    );
}

// The one axis of the fragment's face.
fn face_of(box: Box) -> vec3<bool> {
    return vec3<u32>(box.face & 3u) == vec3<u32>(0u, 1u, 2u);
}

// A front face's fragment. The rasterizer's point is where the ray enters
// the box.
@fragment
fn front(box: Box) -> @location(0) vec4<f32> {
    let hit = first_hit(box, box.point, direction_of(box), 0.0, true, face_of(box));

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
    let cell = vec3<i32>(clamp(floor(box.point), box.low, box.high - 1.0));

    // A ray meets a box's low face moving up the face's axis.
    return surface(box, cell, face_of(box), vec3<bool>(box.face < 4u));
}
