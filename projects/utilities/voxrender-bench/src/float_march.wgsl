// A candidate's march: an f32 DDA over the brick along the world ray in the
// placement's grid. It crosses a region of one value in a single jump. Its
// distances are in the world ray's units.

fn march(
    index: u32,
    x: i32,
    y: i32,
    corner: vec3<i32>,
    extent: vec3<i32>,
    world_origin: vec3<f32>,
    world_direction: vec3<f32>,
) -> Hit {
    let placement = placements[index];
    let grid = placement.bounds.xyz;
    // Adreno's `world_to_object` gives grid rays that miss, so the march
    // carries the world ray through the placement's transform.
    let inverse = vec4<f32>(-placement.rotation.xyz, placement.rotation.w);
    let origin = rotate(inverse, world_origin - placement.position.xyz) / placement.scale.xyz;
    let direction = rotate(inverse, world_direction) / placement.scale.xyz;
    let far_corner = corner + extent;

    // The span of the ray inside the brick, and the axis it enters across, or
    // -1 when it starts inside.
    var near = 0.0;
    var far = 3.0e38;
    var entry = -1;

    for (var a = 0; a < 3; a++) {
        if direction[a] == 0.0 {
            if origin[a] < f32(corner[a]) || origin[a] >= f32(far_corner[a]) {
                return miss();
            }

            continue;
        }

        let up = direction[a] > 0.0;
        let enters = (f32(select(far_corner[a], corner[a], up)) - origin[a]) / direction[a];
        let leaves = (f32(select(corner[a], far_corner[a], up)) - origin[a]) / direction[a];

        if enters > near {
            near = enters;
            entry = a;
        }

        far = min(far, leaves);
    }

    if near >= far {
        return miss();
    }

    let step = vec3<i32>(sign(direction));

    var cell = clamp(
        vec3<i32>(floor(origin + direction * near)),
        corner,
        far_corner - vec3<i32>(1),
    );

    // The value the ray leaves before the first cell.
    var inside = 0u;

    if entry < 0 {
        inside = region_at(placement, cell).value;
    } else {
        cell[entry] = select(far_corner[entry] - 1, corner[entry], step[entry] > 0);

        var before = cell;
        before[entry] -= step[entry];

        if all(before >= vec3<i32>(0)) && all(before < grid) {
            inside = region_at(placement, before).value;
        }
    }

    var t = near;
    var axis = entry;

    for (var n = 0; n < extent.x + extent.y + extent.z + 3; n++) {
        let region = region_at(placement, cell);

        if axis >= 0 && region.value != 0u && region.value != inside {
            return Hit(true, false, t, cell, axis, direction[axis] < 0.0);
        }

        inside = region.value;

        // The region's cube, aligned in the grid and cut to the brick.
        let size = vec3<i32>(region.size);
        let low = max(cell - cell % size, corner);
        let high = min(cell - cell % size + size, far_corner);

        var exit = 3.0e38;

        for (var a = 0; a < 3; a++) {
            if direction[a] != 0.0 {
                let face = f32(select(low[a], high[a], step[a] > 0));
                let at = (face - origin[a]) / direction[a];

                if at < exit {
                    exit = at;
                    axis = a;
                }
            }
        }

        var next = clamp(vec3<i32>(floor(origin + direction * exit)), low, high - vec3<i32>(1));
        next[axis] = select(low[axis] - 1, high[axis], step[axis] > 0);

        if next[axis] < corner[axis] || next[axis] >= far_corner[axis] {
            return miss();
        }

        cell = next;
        t = exit;
    }

    return miss();
}
