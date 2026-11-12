// A shaded view of the boxes around each tile's live cells, over
// `mask_march.wgsl`. One multiview pass draws both eyes. A hit takes its
// cell's material, a sun without shadows, and a hemisphere, under the
// contract's tonemap.

// Every eye is a perspective view.
const RAY_W: f32 = 1.0;

struct Frame {
    // The unit direction to the sun in the world.
    sun: vec4<f32>,

    // The sun's light on a matte face that looks at it.
    sunlight: vec4<f32>,

    // The hemisphere's light on a face that looks up, then on one that looks
    // down.
    sky: vec4<f32>,
    ground: vec4<f32>,

    // The scene's placements. Each eye has a `Place` for each, the first
    // eye's first.
    placements: u32,
}

struct Place {
    // The placement's grid-to-clip matrix for the eye.
    clip: mat4x4<f32>,

    // The eye in the grid, with w = 1.
    ray: vec4<f32>,

    // The world direction of each of the grid's axes, then the meters a cell
    // spans along it.
    x: vec4<f32>,
    y: vec4<f32>,
    z: vec4<f32>,
}

// Whose cell a hit holds.
struct Source {
    // The offset of the tile's words.
    tile: u32,

    // The index of the eye's `Place`.
    place: u32,
}

struct Box {
    @builtin(position) clip: vec4<f32>,

    // The point in the tile's cells.
    @location(0) point: vec3<f32>,

    // `Place::ray` from the tile's low corner.
    @location(1) @interpolate(flat) ray: vec3<f32>,

    // The box's low cell in the tile, then the corner past its high cell.
    @location(2) @interpolate(flat) low: vec3<f32>,
    @location(3) @interpolate(flat) high: vec3<f32>,

    // Zero, since a perspective ray's depth falls by its distance alone.
    @location(4) @interpolate(flat) fall: f32,

    // The axis of the fragment's face, plus 4 on the box's high face.
    @location(5) @interpolate(flat) face: u32,

    // The offset of the tile's words, then the index of the eye's `Place`.
    @location(6) @interpolate(flat) words: vec2<u32>,

    // The first of the box's entries in `crossers`, their count, and the
    // index of the eye's first `Place`.
    @location(7) @interpolate(flat) crossers: vec3<u32>,
}

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<storage, read> places: array<Place>;
// Per object: its `TileMasks` words, with materials.
@group(0) @binding(2) var<storage, read> bits: array<u32>;
// Each material's base color.
@group(0) @binding(3) var<storage, read> colors: array<vec4<f32>>;
// Two words per box: its low cell and its placement's index, then the corner
// past its high cell and the offset of its tile's words.
@group(0) @binding(4) var<storage, read> boxes: array<vec4<u32>>;
// Two words per box that shares space with a box: the low corner of a box
// around the shared space and the other box's index in `boxes` as bits,
// plus `SOLID` for a solid one, then the high corner.
@group(0) @binding(5) var<storage, read> crossers: array<vec4<f32>>;

// The box's corner `corner` for `eye`. Bit `a` of `corner` picks the box's
// high face on axis `a`. `low` holds the box's low cell and its placement's
// index. `high` holds the corner past the box's high cell and the offset of
// its tile's words. `more` holds the first of the box's entries in
// `crossers` and their count.
fn corner_of(eye: u32, corner: u32, low: vec4<u32>, high: vec4<u32>, more: vec4<u32>) -> Box {
    let far = ((vec3<u32>(corner) >> vec3<u32>(0u, 1u, 2u)) & vec3<u32>(1u)) == vec3<u32>(1u);
    let grid = select(low.xyz, high.xyz, far);
    let tile = low.xyz & vec3<u32>(0xfffffff0u);
    let index = eye * frame.placements + low.w;
    let clip = places[index].clip * vec4<f32>(vec3<f32>(grid), 1.0);
    let ray = places[index].ray.xyz - vec3<f32>(tile);

    // A corner whose position or ray is not finite goes to the origin,
    // where its box draws nothing.
    return Box(
        select(vec4<f32>(0.0), clip, sound(clip, vec4<f32>(ray, 0.0))),
        vec3<f32>(grid - tile),
        ray,
        vec3<f32>(low.xyz - tile),
        vec3<f32>(high.xyz - tile),
        0.0,
        min(corner, 7u - corner) | (corner & 4u),
        vec2<u32>(high.w, index),
        vec3<u32>(more.xy, eye * frame.placements),
    );
}

@vertex
fn place(
    @builtin(view_index) eye: u32,
    @location(0) corner: u32,
    @location(1) low: vec4<u32>,
    @location(2) high: vec4<u32>,
    @location(3) more: vec4<u32>,
) -> Box {
    return corner_of(eye, corner, low, high, more);
}

// `place` for the back faces `inside` shades.
@vertex
fn place_near(
    @builtin(view_index) eye: u32,
    @location(0) corner: u32,
    @location(1) low: vec4<u32>,
    @location(2) high: vec4<u32>,
    @location(3) more: vec4<u32>,
) -> Box {
    var box = corner_of(eye, corner, low, high, more);

    box.clip.z *= NEARER;

    return box;
}

// The contract's tonemap, Khronos PBR Neutral.
fn tonemap(color: vec3<f32>) -> vec3<f32> {
    let darkest = min(color.r, min(color.g, color.b));
    let offset = select(0.04, darkest - 6.25 * darkest * darkest, darkest < 0.08);
    let lifted = color - offset;
    let peak = max(lifted.r, max(lifted.g, lifted.b));

    if peak < 0.76 {
        return lifted;
    }

    let compressed = 1.0 - 0.0576 / (peak - 0.52);
    let toward_white = 1.0 - 1.0 / (0.15 * (peak - compressed) + 1.0);

    return mix(lifted * (compressed / peak), vec3<f32>(compressed), toward_white);
}

fn source_of(box: Box) -> Source {
    return Source(box.words.x, box.words.y);
}

// The box at `index` of `boxes`, as the eye's ray along `direction` through
// `box` meets it.
fn other_of(box: Box, direction: vec3<f32>, index: u32) -> Other {
    let low = boxes[index * 2u];
    let high = boxes[index * 2u + 1u];
    let tile = low.xyz & vec3<u32>(0xfffffff0u);
    let place = box.crossers.z + low.w;
    let own = places[box.words.y];
    let its = places[place];

    // The direction in meters, then in the other grid's cells.
    let world = own.x.xyz * (direction.x * own.x.w)
        + own.y.xyz * (direction.y * own.y.w)
        + own.z.xyz * (direction.z * own.z.w);

    return Other(
        high.w,
        vec3<f32>(low.xyz - tile),
        vec3<f32>(high.xyz - tile),
        its.ray.xyz - vec3<f32>(tile),
        vec3<f32>(dot(its.x.xyz, world) / its.x.w, dot(its.y.xyz, world) / its.y.w, dot(its.z.xyz, world) / its.z.w),
        Source(high.w, place),
    );
}

// The shaded color of `cell` in `source`'s tile, across the face on the one
// axis of `across`. The face's normal points up its axis where the ray moves
// down the axis.
fn surface(source: Source, cell: vec3<i32>, across: vec3<bool>, ahead: vec3<bool>) -> vec4<f32> {
    let tile = source.tile;
    let header = bits[tile + 128u];
    let size = header & 15u;

    var material = header >> 4u;

    if size != 0u {
        let at = u32((cell.x * 16 + cell.y) * 16 + cell.z) * size;
        let entry = (bits[tile + 129u + (at >> 5u)] >> (at & 31u)) & ((1u << size) - 1u);

        material = bits[tile + 129u + 128u * size + entry];
    }

    let index = source.place;
    let axis = select(select(places[index].x, places[index].y, across.y), places[index].z, across.z);
    let normal = select(axis.xyz, -axis.xyz, any(across & ahead));
    let up = 0.5 + 0.5 * normal.y;
    let light = mix(frame.ground.rgb, frame.sky.rgb, up)
        + frame.sunlight.rgb * max(dot(normal, frame.sun.xyz), 0.0);

    return vec4<f32>(tonemap(colors[material].rgb * light), 1.0);
}
