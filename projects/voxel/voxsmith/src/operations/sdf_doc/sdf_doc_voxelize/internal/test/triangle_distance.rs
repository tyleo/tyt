use ty_math::TyVector3F64;

/// The distance from `point` to the triangle `a`, `b`, `c`, by the closest
/// point in Ericson's Real-Time Collision Detection.
pub fn triangle_distance(
    point: TyVector3F64,
    a: TyVector3F64,
    b: TyVector3F64,
    c: TyVector3F64,
) -> f64 {
    let ab = b - a;
    let ac = c - a;
    let ap = point - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);

    if d1 <= 0.0 && d2 <= 0.0 {
        return point.distance(a);
    }

    let bp = point - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);

    if d3 >= 0.0 && d4 <= d3 {
        return point.distance(b);
    }

    let vc = d1 * d4 - d3 * d2;

    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return point.distance(a + ab * (d1 / (d1 - d3)));
    }

    let cp = point - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);

    if d6 >= 0.0 && d5 <= d6 {
        return point.distance(c);
    }

    let vb = d5 * d2 - d1 * d6;

    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return point.distance(a + ac * (d2 / (d2 - d6)));
    }

    let va = d3 * d6 - d5 * d4;

    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        return point.distance(b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6))));
    }

    let denominator = 1.0 / (va + vb + vc);
    point.distance(a + ab * (vb * denominator) + ac * (vc * denominator))
}
