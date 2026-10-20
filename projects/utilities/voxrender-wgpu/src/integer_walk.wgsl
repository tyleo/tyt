// voxrender's integer walk in i32 alone: the first cell past the start whose
// material differs from the start cell's, through one placement's grid.
fn walk(placement: Placement, ray: Ray) -> Hit {
    let mirrored = enter(placement, ray);

    if !mirrored.found {
        return miss();
    }

    let bounds = placement.bounds.xyz;
    let origin = mirrored.origin;
    let direction = mirrored.direction;
    let flip = mirrored.flip;
    let last = mirrored.last;
    let entry = mirrored.entry;

    var cell = mirrored.cell;

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

    var error = error_at(cell, origin, direction);

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
