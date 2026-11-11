// A raster of the boxes around each tile's live cells. Each fragment marches
// its pixel's ray through its box and writes the first hit and its depth, so
// the depth test keeps the nearest hit of every box the ray crosses. A hit
// packs as `raster.wgsl`'s does. The WGSL that `BoxRaster` prepends defines
// `march` and `FACE_BITS`.

// The cells a box's faces sit past its cells, so a box the pixel's ray only
// grazes still reaches the march.
const PAD: f32 = 0.015625;

struct Frame {
    // The low bits of a packed hit that hold its voxel id.
    voxel_bits: u32,

    // Pixel (0, 0)'s world ray, then the steps to the next pixel along a row
    // and down a column. A pixel's direction is normalized.
    origin: vec4<f32>,
    origin_x: vec4<f32>,
    origin_y: vec4<f32>,
    direction: vec4<f32>,
    direction_x: vec4<f32>,
    direction_y: vec4<f32>,

    // The rows of the world-to-clip matrix that give a point's z and w.
    clip_z: vec4<f32>,
    clip_w: vec4<f32>,
}

struct Box {
    @builtin(position) clip: vec4<f32>,

    // The box's low cell in its placement's grid, then the placement's index.
    @location(0) @interpolate(flat) low: vec4<u32>,

    // The box's cells along each axis.
    @location(1) @interpolate(flat) extent: vec3<u32>,
}

struct Fragment {
    @location(0) hit: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<storage, read> placements: array<Placement>;
// Per object: its `Tree64` words.
@group(0) @binding(2) var<storage, read> cells: array<u32>;
// Each placement's grid-to-clip matrix.
@group(0) @binding(3) var<storage, read> clips: array<mat4x4<f32>>;

// The marches count no iterations.
fn count(size: i32) {}

// Bit `a` of `corner` picks the box's high face on axis `a`. `high` holds the
// corner past the box's high cell.
@vertex
fn place(
    @location(0) corner: u32,
    @location(1) low: vec4<u32>,
    @location(2) high: vec4<u32>,
) -> Box {
    let far = ((vec3<u32>(corner) >> vec3<u32>(0u, 1u, 2u)) & vec3<u32>(1u)) == vec3<u32>(1u);
    let grid = select(vec3<f32>(low.xyz) - PAD, vec3<f32>(high.xyz) + PAD, far);

    return Box(clips[low.w] * vec4<f32>(grid, 1.0), low, high.xyz - low.xyz);
}

fn first_hit(box: Box) -> Fragment {
    let pixel = vec2<i32>(box.clip.xy);
    let at = vec2<f32>(pixel);
    let origin = frame.origin.xyz + frame.origin_x.xyz * at.x + frame.origin_y.xyz * at.y;
    let direction = normalize(
        frame.direction.xyz + frame.direction_x.xyz * at.x + frame.direction_y.xyz * at.y,
    );

    let index = box.low.w;
    let hit = march(
        index,
        pixel.x,
        pixel.y,
        vec3<i32>(box.low.xyz),
        vec3<i32>(box.extent),
        origin,
        direction,
    );

    if !hit.found {
        discard;
    }

    let bounds = vec3<u32>(placements[index].bounds.xyz);
    let cell = vec3<u32>(hit.position);
    let voxel = (cell.x * bounds.y + cell.y) * bounds.z + cell.z;
    let key = (index << FACE_BITS) | (u32(hit.axis) * 2u + select(0u, 1u, hit.upper));
    let point = vec4<f32>(origin + direction * hit.distance, 1.0);

    return Fragment(
        unpack4x8unorm(voxel | (key << frame.voxel_bits)),
        dot(frame.clip_z, point) / dot(frame.clip_w, point),
    );
}

@fragment
fn back(box: Box) -> Fragment {
    return first_hit(box);
}
