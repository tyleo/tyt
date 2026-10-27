// Each pixel's first hit across every placement, and whether each corner of
// its face sees each light, over `grid_walk.wgsl`. The walk WGSL that
// `HitTrace` appends defines `walk`.

const DIRECTION_BITS: u32 = 16u;

// A shadow ray starts this far in from its face's edges.
const INSET: i32 = 8;

const NO_HIT: u32 = 0xffffffffu;

struct Frame {
    width: u32,
    height: u32,
    placements: u32,
    lights: u32,
}

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<storage, read> placements: array<Placement>;
// Per object: the words its walk reads.
@group(0) @binding(2) var<storage, read> cells: array<u32>;
// Per light and placement: a direction in the grid with w = 0, or the
// light's fixed point in the grid with w = 1.
@group(0) @binding(3) var<storage, read> lights: array<vec4<i32>>;
// Per pixel: placement index, voxel id, face, unused. NO_HIT for a miss.
@group(0) @binding(4) var<storage, read_write> hits: array<vec4<u32>>;
// Per pixel and light: bit c is set when corner c's shadow ray meets no
// surface. Bit 1 of c picks the face's far edge on u, bit 2 on v.
@group(0) @binding(5) var<storage, read_write> shadows: array<u32>;

// The material key at `position` in `dense_cells`'s layout.
fn material_at(placement: Placement, position: vec3<i32>) -> u32 {
    let bounds = vec3<u32>(placement.bounds.xyz);
    let at = vec3<u32>(position);

    return cells[placement.cells.x + at.x * bounds.y * bounds.z + at.y * bounds.z + at.z];
}

// The voxel id of `position`, linear in it, so a position outside the grid
// has one too.
fn linear_voxel(placement: Placement, position: vec3<i32>) -> i32 {
    let bounds = placement.bounds;

    return position.x * bounds.y * bounds.z + position.y * bounds.z + position.z;
}

// The change to the voxel id of a mirrored walk's cell when it steps up each
// axis.
fn voxel_strides(placement: Placement, flip: vec3<bool>) -> vec3<i32> {
    let bounds = placement.bounds;
    let strides = vec3<i32>(bounds.y * bounds.z, bounds.z, 1);

    return select(strides, -strides, flip);
}

// The ray from `origin` toward `point`, as voxrender's
// `RenderGridRay::toward` makes it. An offset past 2^31 wraps.
fn toward(origin: vec3<i32>, point: vec3<i32>) -> Ray {
    let offset = point - origin;
    let size = bitcast<vec3<u32>>(abs(offset));

    var major = 0;

    for (var a = 1; a < 3; a++) {
        if size[a] > size[major] {
            major = a;
        }
    }

    let reach = size[major];

    if reach == 0u {
        return Ray(origin, vec3<i32>(0), origin >> vec3<u32>(FRACTION_BITS), true);
    }

    let shift = max(32u - countLeadingZeros(reach), DIRECTION_BITS) - DIRECTION_BITS;
    let direction = sign(offset) * vec3<i32>(size >> vec3<u32>(shift));

    // The f32 guess of the end's cell, corrected by the remainder of
    // `scaled / span`, which is exact modulo 2^32 and within i32 near the
    // ray. A point on a boundary belongs to the cell the ray moves into.
    let run = abs(direction[major]);
    let span = run * ONE;

    var end = vec3<i32>(0);

    for (var a = 0; a < 3; a++) {
        let x = (f32(origin[a]) + f32(reach) * f32(direction[a]) / f32(run)) / f32(ONE);
        var c = i32(floor(x));
        var r = origin[a] * run + i32(reach) * direction[a] - c * span;

        // Moving down, the end owns its upper boundary.
        let low = select(0, 1, direction[a] < 0);

        for (var k = 0; k <= WINDOW && r < low; k++) {
            c -= 1;
            r += span;
        }

        for (var k = 0; k <= WINDOW && r >= span + low; k++) {
            c += 1;
            r -= span;
        }

        end[a] = c;
    }

    return Ray(origin, direction, end, true);
}

// The fixed point `origin` in the grid of `source`, carried through world
// space into the grid of `destination` in f32.
fn carry(origin: vec3<i32>, source: Placement, destination: Placement) -> vec3<i32> {
    let local = vec3<f32>(origin) / f32(ONE) * source.scale.xyz;
    let world = source.position.xyz + rotate(source.rotation, local);
    let inverse = vec4<f32>(-destination.rotation.xyz, destination.rotation.w);
    let grid = rotate(inverse, world - destination.position.xyz) / destination.scale.xyz;

    return vec3<i32>(round(grid * f32(ONE)));
}

// Whether the shadow ray from `origin` in the grid of placement `source`
// toward light `light` meets no surface in any placement.
fn sees_light(origin: vec3<i32>, source: u32, light: u32) -> bool {
    for (var p = 0u; p < frame.placements; p++) {
        let placement = placements[p];

        var start = origin;

        if p != source {
            start = carry(origin, placements[source], placement);
        }

        let toward_light = lights[light * frame.placements + p];

        var ray = Ray(start, toward_light.xyz, vec3<i32>(0), false);

        if toward_light.w == 1 {
            ray = toward(start, toward_light.xyz);
        }

        let hit = walk(placement, ray);

        if hit.found || hit.starts_inside {
            return false;
        }
    }

    return true;
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= frame.width || id.y >= frame.height {
        return;
    }

    let x = i32(id.x);
    let y = i32(id.y);
    let pixel = id.y * frame.width + id.x;

    var nearest = miss();
    var index = NO_HIT;

    for (var p = 0u; p < frame.placements; p++) {
        let placement = placements[p];

        let ray = Ray(
            at_pixel(placement.origin, x, y),
            at_pixel(placement.direction, x, y),
            vec3<i32>(0),
            false,
        );

        let hit = walk(placement, ray);

        // Hits at one distance go to the earlier placement.
        if hit.found && (!nearest.found || hit.distance < nearest.distance) {
            nearest = hit;
            index = p;
        }
    }

    for (var light = 0u; light < frame.lights; light++) {
        shadows[pixel * frame.lights + light] = 0u;
    }

    if !nearest.found {
        hits[pixel] = vec4<u32>(NO_HIT, 0u, 0u, 0u);

        return;
    }

    let placement = placements[index];
    let d = nearest.axis;
    let u = (d + 1) % 3;
    let v = (d + 2) % 3;

    hits[pixel] = vec4<u32>(
        index,
        voxel_of(placement, nearest.position),
        u32(d) * 2u + select(0u, 1u, nearest.upper),
        0u,
    );

    for (var light = 0u; light < frame.lights; light++) {
        var bits = 0u;

        for (var c = 0u; c < 4u; c++) {
            var origin = nearest.position * ONE;

            origin[d] += select(0, ONE, nearest.upper);
            origin[u] += select(INSET, ONE - INSET, (c & 1u) != 0u);
            origin[v] += select(INSET, ONE - INSET, (c & 2u) != 0u);

            if sees_light(origin, index, light) {
                bits |= 1u << c;
            }
        }

        shadows[pixel * frame.lights + light] = bits;
    }
}
