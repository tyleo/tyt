enable wgpu_ray_query;

// Each pixel's first hit through a hardware ray query over a TLAS of
// placements. Each BLAS holds one AABB per non-empty brick. The march WGSL
// that `RayQueryTrace` appends defines `march`.

const NO_HIT: u32 = 0xffffffffu;

struct Frame {
    width: u32,
    height: u32,
    // Pixel (0, 0)'s world ray, then the steps to the next pixel along a row
    // and down a column. A pixel's direction is normalized.
    origin: vec4<f32>,
    origin_x: vec4<f32>,
    origin_y: vec4<f32>,
    direction: vec4<f32>,
    direction_x: vec4<f32>,
    direction_y: vec4<f32>,
}

struct Brick {
    corner: vec4<i32>,
    // The brick's cells along each axis, cut to its grid.
    extent: vec4<i32>,
}

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<storage, read> placements: array<Placement>;
// Per object: its `Tree64` words.
@group(0) @binding(2) var<storage, read> cells: array<u32>;
// Per object: its bricks in primitive order.
@group(0) @binding(3) var<storage, read> bricks: array<Brick>;
// Per placement: its object's first brick.
@group(0) @binding(4) var<storage, read> first_bricks: array<u32>;
@group(0) @binding(5) var tlas: acceleration_structure;
// Per pixel: placement index, voxel id, face, unused. NO_HIT for a miss.
@group(0) @binding(6) var<storage, read_write> hits: array<vec4<u32>>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= frame.width || id.y >= frame.height {
        return;
    }

    let x = f32(id.x);
    let y = f32(id.y);
    let origin = frame.origin.xyz + frame.origin_x.xyz * x + frame.origin_y.xyz * y;
    let direction = normalize(
        frame.direction.xyz + frame.direction_x.xyz * x + frame.direction_y.xyz * y,
    );

    var query: ray_query;

    rayQueryInitialize(&query, tlas, RayDesc(RAY_FLAG_NONE, 0xffu, 0.0, 3.0e38, origin, direction));

    var nearest = miss();
    var index = NO_HIT;

    while rayQueryProceed(&query) {
        let candidate = rayQueryGetCandidateIntersection(&query);
        let p = candidate.instance_custom_data;
        let brick = bricks[first_bricks[p] + candidate.primitive_index];

        let hit = march(
            p,
            i32(id.x),
            i32(id.y),
            brick.corner.xyz,
            brick.extent.xyz,
            origin,
            direction,
        );

        // Naga places a `rayQueryGenerateIntersection` argument's
        // evaluation after the call, so `distance` is evaluated before the
        // `if`.
        let distance = hit.distance;

        // Hits at one distance go to the earlier placement.
        let nearer = !nearest.found || distance < nearest.distance
            || (distance == nearest.distance && p < index);

        if hit.found && nearer {
            nearest = hit;
            index = p;

            rayQueryGenerateIntersection(&query, distance);
        }
    }

    let pixel = id.y * frame.width + id.x;

    if !nearest.found {
        hits[pixel] = vec4<u32>(NO_HIT, 0u, 0u, 0u);

        return;
    }

    hits[pixel] = vec4<u32>(
        index,
        voxel_of(placements[index], nearest.position),
        u32(nearest.axis) * 2u + select(0u, 1u, nearest.upper),
        0u,
    );
}
