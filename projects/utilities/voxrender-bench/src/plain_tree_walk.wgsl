// The tree walk as a renderer would trace it.

fn count(size: i32) {}

fn walk(placement: Placement, ray: Ray) -> Hit {
    return tree_walk(placement, ray, vec3<i32>(0), placement.bounds.xyz);
}
