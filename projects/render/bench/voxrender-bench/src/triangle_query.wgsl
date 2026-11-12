enable wgpu_ray_query;

// Each pixel's first hit through a hardware ray query over a TLAS of
// placements. Each BLAS holds its object's greedy quads as triangles, so
// the ray tracing unit finds the hit and no march runs.

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

@group(0) @binding(0) var<uniform> frame: Frame;
// Per placement: its grid's size.
@group(0) @binding(1) var<storage, read> bounds: array<vec4<u32>>;
// Per object: each quad's face, its axis doubled, plus one where the face
// looks up the axis. A quad's two triangles follow one another.
@group(0) @binding(2) var<storage, read> faces: array<u32>;
// Per placement: its object's first quad.
@group(0) @binding(3) var<storage, read> first_quads: array<u32>;
@group(0) @binding(4) var tlas: acceleration_structure;
// Per pixel: placement index, voxel id, face, unused. NO_HIT for a miss.
@group(0) @binding(5) var<storage, read_write> hits: array<vec4<u32>>;

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

    // An opaque triangle commits its hit as the query proceeds.
    while rayQueryProceed(&query) {}

    let hit = rayQueryGetCommittedIntersection(&query);
    let pixel = id.y * frame.width + id.x;

    if hit.kind == RAY_QUERY_INTERSECTION_NONE {
        hits[pixel] = vec4<u32>(NO_HIT, 0u, 0u, 0u);

        return;
    }

    let p = hit.instance_custom_data;
    let face = faces[first_quads[p] + hit.primitive_index / 2u];
    let axis = face >> 1u;

    // The hit's point in the placement's cells. A face lies on a whole
    // cell's plane, and its cell sits under a face that looks up its axis.
    let point = hit.world_to_object * vec4<f32>(origin + direction * hit.t, 1.0);

    var cell = vec3<i32>(floor(point));

    cell[axis] = i32(round(point[axis])) - i32(face & 1u);

    let size = bounds[p].xyz;
    let at = vec3<u32>(clamp(cell, vec3<i32>(0), vec3<i32>(size) - 1));

    hits[pixel] = vec4<u32>(p, at.x * size.y * size.z + at.y * size.z + at.z, face, 0u);
}
