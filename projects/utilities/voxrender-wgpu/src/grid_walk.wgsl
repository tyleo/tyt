// The types and helpers of a walk over a placement's grid. A shader that
// includes them binds the words its walk reads as `cells`.
//
// Adreno's shader compiler crashes on a dynamic index into a bool vector, so
// a component's test reads the integers instead.

const FRACTION_BITS: u32 = 13u;
const ONE: i32 = 8192;
const GUARD_BITS: u32 = 12u;
const GUARD_HALF: i32 = 2048;

// Cells an f32 guess may miss by.
const WINDOW: i32 = 2;

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

fn at_pixel(steps: Steps, x: i32, y: i32) -> vec3<i32> {
    let whole = steps.base.xyz + steps.x.xyz * x + steps.y.xyz * y;
    let fraction = steps.base_fraction.xyz + steps.x_fraction.xyz * x
        + steps.y_fraction.xyz * y + vec3<i32>(GUARD_HALF);

    return whole + (fraction >> vec3<u32>(GUARD_BITS));
}

fn miss() -> Hit {
    return Hit(false, false, 0.0, vec3<i32>(0), 0, false);
}

fn voxel_of(placement: Placement, position: vec3<i32>) -> u32 {
    let bounds = vec3<u32>(placement.bounds.xyz);
    let at = vec3<u32>(position);

    return at.x * bounds.y * bounds.z + at.y * bounds.z + at.z;
}

// Which component of a vector `axis` picks, without a dynamic index.
fn axis_mask(axis: i32) -> vec3<bool> {
    return vec3<i32>(0, 1, 2) == vec3<i32>(axis);
}

// The component of `vector` that `axis` picks, without a dynamic index.
fn pick(vector: vec3<i32>, axis: i32) -> i32 {
    return select(select(vector.x, vector.y, axis == 1), vector.z, axis == 2);
}

// The ray mirrored on each axis it moves down, so a walk only steps up, and
// the cell it starts in or enters from.
struct Mirrored {
    // False when the ray misses the grid.
    found: bool,
    origin: vec3<i32>,
    direction: vec3<i32>,
    flip: vec3<bool>,
    // The last cell the walk visits on each axis.
    last: vec3<i32>,
    cell: vec3<i32>,
    // The axis whose lower face the ray enters across, with `cell` at -1 on
    // it, or -1 when the ray starts in the grid.
    entry: i32,
}

// The entry and the end cell rely on integer arithmetic wrapping.
fn enter(placement: Placement, ray: Ray) -> Mirrored {
    let bounds = placement.bounds.xyz;
    let flip = ray.direction < vec3<i32>(0);
    let origin = select(ray.origin, bounds * ONE - ray.origin, flip);
    let direction = abs(ray.direction);

    var mirrored = Mirrored(false, origin, direction, flip, vec3<i32>(0), vec3<i32>(0), -1);

    for (var a = 0; a < 3; a++) {
        if direction[a] == 0 && (origin[a] < 0 || origin[a] >= bounds[a] * ONE) {
            return mirrored;
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

            if direction[a] > 0 && origin[a] < 0 && t > latest {
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
                    return mirrored;
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

                if direction[a] > 0 && origin[a] < 0 && c < 0 {
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
            return mirrored;
        }
    }

    mirrored.found = true;
    mirrored.last = last;
    mirrored.cell = cell;
    mirrored.entry = entry;

    return mirrored;
}

// A mirrored walk's error terms at `cell`, exact modulo 2^32 and within i32
// near the ray.
fn error_at(cell: vec3<i32>, origin: vec3<i32>, direction: vec3<i32>) -> vec3<i32> {
    let next = (cell + vec3<i32>(1)) * ONE - origin;

    return vec3<i32>(
        next.x * direction.y - next.y * direction.x,
        next.x * direction.z - next.z * direction.x,
        next.y * direction.z - next.z * direction.y,
    );
}

fn rotate(rotation: vec4<f32>, vector: vec3<f32>) -> vec3<f32> {
    let twice = 2.0 * cross(rotation.xyz, vector);

    return vector + rotation.w * twice + cross(rotation.xyz, twice);
}
