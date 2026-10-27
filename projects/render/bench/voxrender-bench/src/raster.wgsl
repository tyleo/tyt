// A greedy-mesh raster. Each fragment writes the first hit the march would
// report, packed into 32 bits: the voxel id of the cell under the hit point,
// then the face, then the placement. A pixel without a hit keeps the clear's
// all-ones.

struct Frame {
    // The low bits of a packed hit that hold its voxel id.
    voxel_bits: u32,

    placements: u32,
}

struct Varyings {
    @builtin(position) clip: vec4<f32>,

    // The point in its placement's grid.
    @location(0) grid: vec3<f32>,

    // The placement's index above the face's code.
    @location(1) @interpolate(flat) key: u32,
}

@group(0) @binding(0) var<uniform> frame: Frame;

// Each placement's grid-to-clip matrix. The first eye's matrices come first.
@group(0) @binding(1) var<storage, read> clips: array<mat4x4<f32>>;

// Each placement's grid size.
@group(0) @binding(2) var<storage, read> bounds: array<vec4<u32>>;

// `corner` holds a grid corner and its face's code. `clip` picks the
// placement's matrix for the eye.
fn place_corner(corner: vec4<u32>, clip: u32, placement: u32) -> Varyings {
    let grid = vec3<f32>(corner.xyz);

    return Varyings(
        clips[clip] * vec4<f32>(grid, 1.0),
        grid,
        placement << FACE_BITS | corner.w,
    );
}

@fragment
fn hit(in: Varyings) -> @location(0) vec4<f32> {
    let face = in.key & ((1u << FACE_BITS) - 1u);
    let axis = face >> 1u;

    // The face's plane lies on a whole cell along its axis. A face whose
    // normal points up the axis bounds the cell below the plane.
    var cell = vec3<u32>(floor(in.grid));
    cell[axis] = u32(round(in.grid[axis])) - (face & 1u);

    let size = bounds[in.key >> FACE_BITS];
    let voxel = (cell.x * size.y + cell.y) * size.z + cell.z;

    return unpack4x8unorm(voxel | (in.key << frame.voxel_bits));
}
