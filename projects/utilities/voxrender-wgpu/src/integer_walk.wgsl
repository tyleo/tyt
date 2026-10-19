// voxrender's integer walk in i32 alone: the first cell past the start whose
// material differs from the start cell's, through one placement's grid. The
// entry, the end cell, and the error terms rely on integer arithmetic
// wrapping.
fn walk(placement: Placement, ray: Ray) -> Hit {
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

    // A step up each axis changes the error terms by these.
    let change = ONE * direction;
    let x_change = vec3<i32>(change.y, change.z, 0);
    let y_change = vec3<i32>(-change.x, 0, change.z);
    let z_change = vec3<i32>(0, -change.x, -change.y);

    let strides = voxel_strides(placement, flip);
    var voxel = linear_voxel(placement, select(cell, bounds - vec3<i32>(1) - cell, flip));

    let meters = length(placement.scale.xyz * vec3<f32>(ray.direction)) / f32(ONE);
    let steps = bounds.x + bounds.y + bounds.z + 3;

    for (var n = 0; n < steps; n++) {
        let first = select(0, 1, error.x > 0);
        let axis = select(first, 2, select(error.y, error.z, first == 1) > 0);

        cell += select(vec3<i32>(0), vec3<i32>(1), axis_mask(axis));

        // Only the stepped axis can pass its last cell.
        if any(cell > last) {
            return start;
        }

        error += select(select(x_change, y_change, axis == 1), z_change, axis == 2);
        voxel += pick(strides, axis);

        let material = cells[placement.cells.x + u32(voxel)];

        if material != 0u && material != inside {
            let position = select(cell, bounds - vec3<i32>(1) - cell, flip);

            // An entry faces the ray: the cell's upper face on a mirrored
            // axis.
            let ahead = pick(cell, axis) * ONE - pick(origin, axis);
            let distance = f32(ahead) / f32(pick(direction, axis)) * meters;
            let upper = pick(ray.direction, axis) < 0;

            return Hit(true, start.starts_inside, distance, position, axis, upper);
        }

        inside = material;
    }

    return start;
}
