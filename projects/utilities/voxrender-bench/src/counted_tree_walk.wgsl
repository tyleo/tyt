// The tree walk, reporting the iterations a pixel's walks take over every
// placement. Each call returns a hit nearer than the last, at the voxel id of
// the running sum. The pixel's hit then carries the total.

var<private> calls: i32;

fn walk(placement: Placement, ray: Ray) -> Hit {
    _ = tree_walk(placement, ray, vec3<i32>(0), placement.bounds.xyz);
    calls += 1;

    return Hit(true, false, -f32(calls), vec3<i32>(0, 0, i32(iterations)), 0, false);
}
