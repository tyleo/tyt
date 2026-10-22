// A candidate's march: the tree walk over the brick along the placement's
// integer pixel ray. Its distances are in meters, the world ray's units.

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

    let ray = Ray(
        at_pixel(placement.origin, x, y),
        at_pixel(placement.direction, x, y),
        vec3<i32>(0),
        false,
    );

    return tree_walk(placement, ray, corner, extent);
}
