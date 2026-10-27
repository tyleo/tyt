// The tree walk, reporting the iterations a pixel's walks take over every
// placement that leave a region of side `SIDE`. The bench prepends `SIDE` as a
// const. Each call returns a hit nearer than the last, at the voxel id of the
// running sum. The pixel's hit then carries the total.

var<private> calls: i32;

// The counted iterations of every walk in this invocation so far.
var<private> iterations: u32;

fn count(size: i32) {
    if size == SIDE {
        iterations += 1u;
    }
}

fn walk(placement: Placement, ray: Ray) -> Hit {
    _ = tree_walk(placement, ray, vec3<i32>(0), placement.bounds.xyz);
    calls += 1;

    return Hit(true, false, -f32(calls), vec3<i32>(0, 0, i32(iterations)), 0, false);
}
