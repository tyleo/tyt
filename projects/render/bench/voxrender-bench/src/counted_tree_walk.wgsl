// The tree walk, reporting the iterations a pixel's walks take over every
// placement that leave a region of side `SIDE`. The bench prepends `SIDE` as a
// const. The last placement's call returns a hit nearer than any other, at the
// voxel id of the running sum. The earlier calls return their hits, so each
// walk ends where the trace's would.

var<private> calls: u32;

// The counted iterations of every walk in this invocation so far.
var<private> iterations: u32;

fn count(size: i32) {
    if size == SIDE {
        iterations += 1u;
    }
}

fn walk(placement: Placement, ray: Ray) -> Hit {
    var hit = miss();

    if !misses(placement, ray) {
        hit = tree_walk(placement, ray, vec3<i32>(0), placement.bounds.xyz);
    }

    calls += 1u;

    if calls < frame.placements {
        return hit;
    }

    return Hit(true, false, -1.0, vec3<i32>(0, 0, i32(iterations)), 0, false);
}
