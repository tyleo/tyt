/// The quadratic smooth maximum of the distances `a` and `b`, whose rounding
/// reaches `radius` along each surface.
pub fn smooth_intersect_distance(a: f64, b: f64, radius: f64) -> f64 {
    let h = (radius - (a - b).abs()).max(0.0) / radius;
    a.max(b) + h * h * radius / 4.0
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::smooth_intersect_distance;

    #[test]
    fn the_rounding_lifts_only_within_its_reach() {
        assert_eq!(smooth_intersect_distance(0.0, 0.0, 1.0), 0.25);
        assert_eq!(smooth_intersect_distance(0.5, 0.0, 1.0), 0.5625);
        assert_eq!(smooth_intersect_distance(-1.5, -0.25, 1.0), -0.25);
    }
}
