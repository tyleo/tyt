// The ids a mask march writes, over `mask_march.wgsl`: a raster of the boxes
// around each tile's live cells over `TileMasks`. A hit packs as
// `raster.wgsl`'s does. `BoxRaster` prepends `FACE_BITS`.

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

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<storage, read> places: array<Place>;
// Per object: its `TileMasks` words.
@group(0) @binding(2) var<storage, read> bits: array<u32>;

// The box's corner `corner`. Bit `a` of `corner` picks the box's high face on
// axis `a`. `low` holds the box's low cell and its placement's index. `high`
// holds the corner past the box's high cell and the offset of its tile's bits.
fn corner_of(corner: u32, low: vec4<u32>, high: vec4<u32>) -> Box {
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

@vertex
fn place(
    @location(0) corner: u32,
    @location(1) low: vec4<u32>,
    @location(2) high: vec4<u32>,
) -> Box {
    return corner_of(corner, low, high);
}

// `place` for the back faces `inside` shades, under a perspective view.
@vertex
fn place_near(
    @location(0) corner: u32,
    @location(1) low: vec4<u32>,
    @location(2) high: vec4<u32>,
) -> Box {
    var box = corner_of(corner, low, high);

    box.clip.z *= NEARER;

    return box;
}

// The packed hit of `cell` in the box's tile, across the face on the one axis
// of `across`. The face's normal points up its axis where the ray moves down
// the axis.
fn surface(box: Box, cell: vec3<i32>, across: vec3<bool>, ahead: vec3<bool>) -> vec4<f32> {
    let axis = select(select(0u, 1u, across.y), 2u, across.z);
    let face = axis * 2u + select(1u, 0u, any(across & ahead));
    let voxel = box.voxel.x + u32(cell.x) * box.voxel.y + u32(cell.y) * box.voxel.z + u32(cell.z);

    return unpack4x8unorm(voxel | ((box.high.w | face) << frame.voxel_bits));
}
