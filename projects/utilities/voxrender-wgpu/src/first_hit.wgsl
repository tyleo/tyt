// Each pixel's first hit across every placement, and whether each corner of
// its face sees each light, by voxrender's integer walk in i32 alone. The
// entry, the end cell, and the error terms rely on integer arithmetic
// wrapping.

const FRACTION_BITS: u32 = 13u;
const ONE: i32 = 8192;
const GUARD_BITS: u32 = 12u;
const GUARD_HALF: i32 = 2048;
const DIRECTION_BITS: u32 = 16u;

// A shadow ray starts this far in from its face's edges.
const INSET: i32 = 8;

// Cells an f32 guess may miss by.
const WINDOW: i32 = 2;

const NO_HIT: u32 = 0xffffffffu;

// Walks with an f32 DDA instead, for the record.
override FLOAT_WALK: bool = false;

struct Frame {
    width: u32,
    height: u32,
    placements: u32,
    lights: u32,
}

// A view's per-pixel steps in one grid, each split into whole units and a
// guard fraction in [0, 2^GUARD_BITS).
struct Steps {
    base: vec4<i32>,
    base_fraction: vec4<i32>,
    x: vec4<i32>,
    x_fraction: vec4<i32>,
    y: vec4<i32>,
    y_fraction: vec4<i32>,
}

struct Placement {
    bounds: vec4<i32>,
    // x is the offset of the placement's object in `cells`.
    cells: vec4<u32>,
    scale: vec4<f32>,
    position: vec4<f32>,
    rotation: vec4<f32>,
    origin: Steps,
    direction: Steps,
}

struct Ray {
    origin: vec3<i32>,
    direction: vec3<i32>,
    // The last cell the walk visits when `ends`.
    end: vec3<i32>,
    ends: bool,
}

struct Hit {
    found: bool,
    // Whether the ray starts in a live cell.
    starts_inside: bool,
    distance: f32,
    // The hit cell's position in its grid.
    position: vec3<i32>,
    axis: i32,
    // Whether the face's normal points up its axis.
    upper: bool,
}

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<storage, read> placements: array<Placement>;
// Per cell: 0 when empty, else a material key.
@group(0) @binding(2) var<storage, read> cells: array<u32>;
// Per light and placement: a direction in the grid with w = 0, or the
// light's fixed point in the grid with w = 1.
@group(0) @binding(3) var<storage, read> lights: array<vec4<i32>>;
// Per pixel: placement index, voxel id, face, unused. NO_HIT for a miss.
@group(0) @binding(4) var<storage, read_write> hits: array<vec4<u32>>;
// Per pixel and light: bit c is set when corner c's shadow ray meets no
// surface. Bit 1 of c picks the face's far edge on u, bit 2 on v.
@group(0) @binding(5) var<storage, read_write> shadows: array<u32>;

fn at_pixel(steps: Steps, x: i32, y: i32) -> vec3<i32> {
    let whole = steps.base.xyz + steps.x.xyz * x + steps.y.xyz * y;
    let fraction = steps.base_fraction.xyz + steps.x_fraction.xyz * x
        + steps.y_fraction.xyz * y + vec3<i32>(GUARD_HALF);

    return whole + (fraction >> vec3<u32>(GUARD_BITS));
}

// The index in the error terms of axes a < b.
fn pair(a: i32, b: i32) -> i32 {
    return a + b - 1;
}

fn miss() -> Hit {
    return Hit(false, false, 0.0, vec3<i32>(0), 0, false);
}

fn material_at(placement: Placement, position: vec3<i32>) -> u32 {
    let bounds = vec3<u32>(placement.bounds.xyz);
    let at = vec3<u32>(position);

    return cells[placement.cells.x + at.x * bounds.y * bounds.z + at.y * bounds.z + at.z];
}

fn voxel_of(placement: Placement, position: vec3<i32>) -> u32 {
    let bounds = vec3<u32>(placement.bounds.xyz);
    let at = vec3<u32>(position);

    return at.x * bounds.y * bounds.z + at.y * bounds.z + at.z;
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

// The first cell past the start whose material differs from the start
// cell's, through one placement's grid.
fn walk(placement: Placement, ray: Ray) -> Hit {
    if FLOAT_WALK {
        return float_walk(placement, ray);
    }

    return integer_walk(placement, ray);
}

fn integer_walk(placement: Placement, ray: Ray) -> Hit {
    let bounds = placement.bounds.xyz;
    let flip = ray.direction < vec3<i32>(0);

    // Mirror each axis the ray moves down, so the walk only steps up.
    let origin = select(ray.origin, bounds * ONE - ray.origin, flip);
    let direction = abs(ray.direction);

    for (var a = 0; a < 3; a++) {
        if direction[a] == 0 && (origin[a] < 0 || origin[a] >= bounds[a] * ONE) {
            return miss();
        }
    }

    var last = bounds - vec3<i32>(1);

    if ray.ends {
        last = min(last, select(ray.end, bounds - vec3<i32>(1) - ray.end, flip));
    }

    let entering = direction > vec3<i32>(0) & origin < vec3<i32>(0);
    var cell = origin >> vec3<u32>(FRACTION_BITS);
    var entry = -1;

    if any(entering) {
        // The f32 guess of the face the ray reaches last. A near tie can pick
        // a face too early, which the correction below finds.
        var latest = -1.0;

        for (var a = 0; a < 3; a++) {
            let t = -f32(origin[a]) / f32(direction[a]);

            if entering[a] && t > latest {
                latest = t;
                entry = a;
            }
        }

        for (var attempt = 0; attempt < 3; attempt++) {
            var behind = -1;

            for (var a = 0; a < 3; a++) {
                if a == entry || direction[a] == 0 {
                    continue;
                }

                let e = entry;

                // The f32 guess of the ray's cell on `a` at the face of `e`.
                let x = (f32(origin[a]) + -f32(origin[e]) / f32(direction[e]) * f32(direction[a]))
                    / f32(ONE);

                if x > f32(bounds[a] + WINDOW) {
                    return miss();
                }

                var c = i32(floor(max(x, -f32(WINDOW) - 2.0)));

                // `r` is exact modulo 2^32 and within i32 near the ray. A
                // crossing at the face on a later axis comes after the entry.
                let span = ONE * direction[e];
                var r = origin[a] * direction[e] - origin[e] * direction[a] - c * span;

                for (var k = 0; k <= WINDOW && (r < 0 || (r == 0 && a > e)); k++) {
                    c -= 1;
                    r += span;
                }

                for (var k = 0; k <= WINDOW && (r > span || (r == span && a < e)); k++) {
                    c += 1;
                    r -= span;
                }

                cell[a] = c;

                if entering[a] && c < 0 {
                    behind = a;
                }
            }

            if behind < 0 {
                break;
            }

            // The ray had not crossed `behind` yet at the guessed face.
            entry = behind;
        }

        cell[entry] = -1;
    }

    for (var a = 0; a < 3; a++) {
        if a != entry && (cell[a] < 0 || cell[a] >= bounds[a]) {
            return miss();
        }
    }

    var inside = 0u;

    if entry < 0 {
        inside = material_at(placement, select(cell, bounds - vec3<i32>(1) - cell, flip));
    }

    var start = miss();
    start.starts_inside = inside != 0u;

    if all(direction == vec3<i32>(0)) {
        return start;
    }

    for (var a = 0; a < 3; a++) {
        if a != entry && cell[a] > last[a] {
            return start;
        }
    }

    let next = (cell + vec3<i32>(1)) * ONE - origin;

    var error = vec3<i32>(
        next.x * direction.y - next.y * direction.x,
        next.x * direction.z - next.z * direction.x,
        next.y * direction.z - next.z * direction.y,
    );

    let meters = length(placement.scale.xyz * vec3<f32>(ray.direction)) / f32(ONE);
    let steps = bounds.x + bounds.y + bounds.z + 3;

    for (var n = 0; n < steps; n++) {
        let first = select(0, 1, error[0] > 0);
        let axis = select(first, 2, error[pair(first, 2)] > 0);

        let ahead = (cell[axis] + 1) * ONE - origin[axis];

        cell[axis] += 1;

        if cell[axis] > last[axis] {
            return start;
        }

        for (var other = 0; other < 3; other++) {
            if other == axis {
                continue;
            }

            let change = ONE * direction[other];

            if axis < other {
                error[pair(axis, other)] += change;
            } else {
                error[pair(other, axis)] -= change;
            }
        }

        let position = select(cell, bounds - vec3<i32>(1) - cell, flip);
        let material = material_at(placement, position);

        if material != 0u && material != inside {
            // An entry faces the ray: the cell's upper face on a mirrored
            // axis.
            let distance = f32(ahead) / f32(direction[axis]) * meters;

            return Hit(true, start.starts_inside, distance, position, axis, flip[axis]);
        }

        inside = material;
    }

    return start;
}

// The f32 DDA of the draft plan over the same inputs and the same mirrored
// tie order. Only the traversal differs from the integer walk's.
fn float_walk(placement: Placement, ray: Ray) -> Hit {
    let bounds = placement.bounds.xyz;
    let flip = ray.direction < vec3<i32>(0);
    let mirrored = select(ray.origin, bounds * ONE - ray.origin, flip);

    for (var a = 0; a < 3; a++) {
        if ray.direction[a] == 0 && (mirrored[a] < 0 || mirrored[a] >= bounds[a] * ONE) {
            return miss();
        }
    }

    var last = bounds - vec3<i32>(1);

    if ray.ends {
        last = min(last, select(ray.end, bounds - vec3<i32>(1) - ray.end, flip));
    }

    let origin = vec3<f32>(mirrored) / f32(ONE);
    let direction = vec3<f32>(abs(ray.direction));

    // The ray enters through the lower face it reaches last. At a tie the
    // higher axis is crossed last.
    var entry = -1;
    var entered = 0.0;

    for (var a = 0; a < 3; a++) {
        if direction[a] > 0.0 && origin[a] < 0.0 {
            let t = -origin[a] / direction[a];

            if entry < 0 || t >= entered {
                entry = a;
                entered = t;
            }
        }
    }

    var cell = vec3<i32>(floor(origin + direction * entered));

    if entry >= 0 {
        cell[entry] = -1;
    }

    for (var a = 0; a < 3; a++) {
        if a != entry && (cell[a] < 0 || cell[a] >= bounds[a]) {
            return miss();
        }
    }

    var inside = 0u;

    if entry < 0 {
        inside = material_at(placement, select(cell, bounds - vec3<i32>(1) - cell, flip));
    }

    var start = miss();
    start.starts_inside = inside != 0u;

    if all(direction == vec3<f32>(0.0)) {
        return start;
    }

    for (var a = 0; a < 3; a++) {
        if a != entry && cell[a] > last[a] {
            return start;
        }
    }

    // An axis the ray does not move along is never reached.
    let never = 3.0e38;
    var t_max = vec3<f32>(never);
    var t_delta = vec3<f32>(never);

    for (var a = 0; a < 3; a++) {
        if direction[a] > 0.0 {
            t_max[a] = (f32(cell[a] + 1) - origin[a]) / direction[a];
            t_delta[a] = 1.0 / direction[a];
        }
    }

    let meters = length(placement.scale.xyz * vec3<f32>(ray.direction));
    let steps = bounds.x + bounds.y + bounds.z + 3;

    for (var n = 0; n < steps; n++) {
        let first = select(0, 1, t_max.y < t_max.x);
        let axis = select(first, 2, t_max.z < t_max[first]);
        let t = t_max[axis];

        cell[axis] += 1;

        if cell[axis] > last[axis] {
            return start;
        }

        t_max[axis] += t_delta[axis];

        let position = select(cell, bounds - vec3<i32>(1) - cell, flip);
        let material = material_at(placement, position);

        if material != 0u && material != inside {
            return Hit(true, start.starts_inside, t * meters, position, axis, flip[axis]);
        }

        inside = material;
    }

    return start;
}

fn rotate(rotation: vec4<f32>, vector: vec3<f32>) -> vec3<f32> {
    let twice = 2.0 * cross(rotation.xyz, vector);

    return vector + rotation.w * twice + cross(rotation.xyz, twice);
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
