// The f32 DDA of the draft plan over the integer walk's inputs and mirrored
// tie order, for the record. Only the traversal differs from the integer
// walk's.
fn walk(placement: Placement, ray: Ray) -> Hit {
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

    let strides = voxel_strides(placement, flip);
    var voxel = linear_voxel(placement, select(cell, bounds - vec3<i32>(1) - cell, flip));

    let meters = length(placement.scale.xyz * vec3<f32>(ray.direction));
    let steps = bounds.x + bounds.y + bounds.z + 3;

    for (var n = 0; n < steps; n++) {
        let first = select(0, 1, t_max.y < t_max.x);
        let first_t = select(t_max.x, t_max.y, first == 1);
        let axis = select(first, 2, t_max.z < first_t);
        let t = select(first_t, t_max.z, axis == 2);
        let stepped = axis_mask(axis);

        cell += select(vec3<i32>(0), vec3<i32>(1), stepped);

        // Only the stepped axis can pass its last cell.
        if any(cell > last) {
            return start;
        }

        t_max += select(vec3<f32>(0.0), t_delta, stepped);
        voxel += pick(strides, axis);

        let material = cells[placement.cells.x + u32(voxel)];

        if material != 0u && material != inside {
            let position = select(cell, bounds - vec3<i32>(1) - cell, flip);
            let upper = pick(ray.direction, axis) < 0;

            return Hit(true, start.starts_inside, t * meters, position, axis, upper);
        }

        inside = material;
    }

    return start;
}
