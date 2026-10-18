// Each pixel's first hit across every placement, by voxrender's integer walk
// in i32 alone. Integer arithmetic wraps, which the entry and the error terms
// rely on.

const FRACTION_BITS: u32 = 13u;
const ONE: i32 = 8192;
const GUARD_BITS: u32 = 12u;
const GUARD_HALF: i32 = 2048;

// Cells an f32 guess at the entry may miss by.
const WINDOW: i32 = 2;

const NO_HIT: u32 = 0xffffffffu;

struct Frame {
    width: u32,
    height: u32,
    placements: u32,
    pad: u32,
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
    origin: Steps,
    direction: Steps,
}

struct Hit {
    found: bool,
    distance: f32,
    voxel: u32,
    face: u32,
}

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<storage, read> placements: array<Placement>;
// Per cell: 0 when empty, else a material key.
@group(0) @binding(2) var<storage, read> cells: array<u32>;
// Per pixel: placement index, voxel id, face, unused. NO_HIT for a miss.
@group(0) @binding(3) var<storage, read_write> hits: array<vec4<u32>>;

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
    return Hit(false, 0.0, 0u, 0u);
}

fn first_hit(placement: Placement, ray_origin: vec3<i32>, ray_direction: vec3<i32>) -> Hit {
    let bounds = placement.bounds.xyz;
    let flip = ray_direction < vec3<i32>(0);

    // Mirror each axis the ray moves down, so the walk only steps up.
    let origin = select(ray_origin, bounds * ONE - ray_origin, flip);
    let direction = abs(ray_direction);

    for (var a = 0; a < 3; a++) {
        if direction[a] == 0 && (origin[a] < 0 || origin[a] >= bounds[a] * ONE) {
            return miss();
        }
    }

    if all(direction == vec3<i32>(0)) {
        return miss();
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

    let next = (cell + vec3<i32>(1)) * ONE - origin;

    var error = vec3<i32>(
        next.x * direction.y - next.y * direction.x,
        next.x * direction.z - next.z * direction.x,
        next.y * direction.z - next.z * direction.y,
    );

    let plane = u32(bounds.y * bounds.z);
    let offset = placement.cells.x;

    var inside = 0u;

    if entry < 0 {
        let position = vec3<u32>(select(cell, bounds - vec3<i32>(1) - cell, flip));
        inside = cells[offset + position.x * plane + position.y * u32(bounds.z) + position.z];
    }

    let meters = length(placement.scale.xyz * vec3<f32>(ray_direction)) / f32(ONE);
    let steps = bounds.x + bounds.y + bounds.z + 3;

    for (var n = 0; n < steps; n++) {
        let first = select(0, 1, error[0] > 0);
        let axis = select(first, 2, error[pair(first, 2)] > 0);

        let ahead = (cell[axis] + 1) * ONE - origin[axis];

        cell[axis] += 1;

        if cell[axis] >= bounds[axis] {
            return miss();
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

        let position = vec3<u32>(select(cell, bounds - vec3<i32>(1) - cell, flip));
        let voxel = position.x * plane + position.y * u32(bounds.z) + position.z;
        let material = cells[offset + voxel];

        if material != 0u && material != inside {
            // An entry faces the ray: the cell's upper face on a mirrored
            // axis.
            let face = u32(axis) * 2u + select(0u, 1u, flip[axis]);

            return Hit(true, f32(ahead) / f32(direction[axis]) * meters, voxel, face);
        }

        inside = material;
    }

    return miss();
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= frame.width || id.y >= frame.height {
        return;
    }

    let x = i32(id.x);
    let y = i32(id.y);

    var nearest = miss();
    var index = NO_HIT;

    for (var p = 0u; p < frame.placements; p++) {
        let placement = placements[p];

        let hit = first_hit(
            placement,
            at_pixel(placement.origin, x, y),
            at_pixel(placement.direction, x, y),
        );

        // Hits at one distance go to the earlier placement.
        if hit.found && (!nearest.found || hit.distance < nearest.distance) {
            nearest = hit;
            index = p;
        }
    }

    var result = vec4<u32>(NO_HIT, 0u, 0u, 0u);

    if nearest.found {
        result = vec4<u32>(index, nearest.voxel, nearest.face, 0u);
    }

    hits[id.y * frame.width + id.x] = result;
}
