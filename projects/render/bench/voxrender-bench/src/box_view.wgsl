// A shaded view of the boxes around each tile's live cells, over
// `mask_march.wgsl`. One multiview pass draws both eyes. A hit takes its
// cell's material, a sun without shadows, and a hemisphere, under the
// contract's tonemap.

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

    // The world directions of the grid's axes.
    x: vec4<f32>,
    y: vec4<f32>,
    z: vec4<f32>,
}

struct Box {
    @builtin(position) clip: vec4<f32>,

    // The point in the tile's cells.
    @location(0) point: vec3<f32>,

    // `Place::ray` from the tile's low corner.
    @location(1) @interpolate(flat) ray: vec4<f32>,

    // The box's low cell in the tile, then the offset of the tile's words.
    @location(2) @interpolate(flat) low: vec4<u32>,

    // The corner past the box's high cell in the tile, then the index of the
    // eye's `Place`.
    @location(3) @interpolate(flat) high: vec4<u32>,

    // Zero, since a perspective ray's depth falls by its distance alone.
    @location(4) @interpolate(flat) fall: f32,
}

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<storage, read> places: array<Place>;
// Per object: its `TileMasks` words, with materials.
@group(0) @binding(2) var<storage, read> bits: array<u32>;
// Each material's base color.
@group(0) @binding(3) var<storage, read> colors: array<vec4<f32>>;

// Bit `a` of `corner` picks the box's high face on axis `a`. `low` holds the
// box's low cell and its placement's index. `high` holds the corner past the
// box's high cell and the offset of its tile's words.
@vertex
fn place(
    @builtin(view_index) eye: u32,
    @location(0) corner: u32,
    @location(1) low: vec4<u32>,
    @location(2) high: vec4<u32>,
) -> Box {
    let far = ((vec3<u32>(corner) >> vec3<u32>(0u, 1u, 2u)) & vec3<u32>(1u)) == vec3<u32>(1u);
    let grid = select(low.xyz, high.xyz, far);
    let tile = low.xyz & vec3<u32>(0xfffffff0u);
    let index = eye * frame.placements + low.w;

    return Box(
        places[index].clip * vec4<f32>(vec3<f32>(grid), 1.0),
        vec3<f32>(grid - tile),
        vec4<f32>(places[index].ray.xyz - vec3<f32>(tile), 1.0),
        vec4<u32>(low.xyz - tile, high.w),
        vec4<u32>(high.xyz - tile, index),
        0.0,
    );
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

// The shaded color of `cell` in the box's tile, across the face on the one
// axis of `across`. The face's normal points up its axis where the ray moves
// down the axis.
fn surface(box: Box, cell: vec3<i32>, across: vec3<bool>, ahead: vec3<bool>) -> vec4<f32> {
    let tile = box.low.w;
    let header = bits[tile + 128u];
    let size = header & 15u;

    var material = header >> 4u;

    if size != 0u {
        let at = u32((cell.x * 16 + cell.y) * 16 + cell.z) * size;
        let entry = (bits[tile + 129u + (at >> 5u)] >> (at & 31u)) & ((1u << size) - 1u);

        material = bits[tile + 129u + 128u * size + entry];
    }

    let index = box.high.w;
    let axis = select(select(places[index].x, places[index].y, across.y), places[index].z, across.z);
    let normal = select(axis.xyz, -axis.xyz, any(across & ahead));
    let up = 0.5 + 0.5 * normal.y;
    let light = mix(frame.ground.rgb, frame.sky.rgb, up)
        + frame.sunlight.rgb * max(dot(normal, frame.sun.xyz), 0.0);

    return vec4<f32>(tonemap(colors[material].rgb * light), 1.0);
}
