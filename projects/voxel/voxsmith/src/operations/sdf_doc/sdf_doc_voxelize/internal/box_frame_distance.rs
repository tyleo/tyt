use ty_math::TyVector3F64;

/// The signed distance to the point at `offset` from the twelve bars along the
/// edges of the box with `half_extents` around the origin. Each bar runs twice
/// `half_thickness` across inside the box.
pub fn box_frame_distance(
    offset: TyVector3F64,
    half_extents: TyVector3F64,
    half_thickness: f64,
) -> f64 {
    let p = offset.abs() - half_extents;
    let q = (p + half_thickness).abs() - half_thickness;

    bar_distance(TyVector3F64::new(p.x, q.y, q.z))
        .min(bar_distance(TyVector3F64::new(q.x, p.y, q.z)))
        .min(bar_distance(TyVector3F64::new(q.x, q.y, p.z)))
}

/// The box distance of the point `q` folded past a box's corner.
fn bar_distance(q: TyVector3F64) -> f64 {
    q.max(TyVector3F64::ZERO).length() + q.max_element().min(0.0)
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{assert_shape3d, box_reference};
    use sdfcore::SdfShape3d;
    use ty_math::TyVector3F64;

    #[test]
    fn a_box_frame_measures_to_its_nearest_bar() {
        let min = TyVector3F64::new(-0.5, 0.0, -0.4);
        let max = TyVector3F64::new(0.5, 0.75, 0.4);
        let thickness = 0.125;

        // The twelve bars, each `thickness` across inside the corners.
        let mut bars = Vec::new();

        for axis in 0..3 {
            let across = [(axis + 1) % 3, (axis + 2) % 3];

            for first in [min[across[0]], max[across[0]] - thickness] {
                for second in [min[across[1]], max[across[1]] - thickness] {
                    let mut bar_min = min;
                    let mut bar_max = max;
                    bar_min[across[0]] = first;
                    bar_max[across[0]] = first + thickness;
                    bar_min[across[1]] = second;
                    bar_max[across[1]] = second + thickness;
                    bars.push((bar_min, bar_max));
                }
            }
        }

        let shape = SdfShape3d::BoxFrame {
            min,
            max,
            thickness,
        };
        let grid = (TyVector3F64::splat(-1.0), TyVector3F64::splat(1.0), 17);

        assert_shape3d(Vec::new(), vec![shape], grid, 1e-12, |point| {
            bars.iter()
                .map(|(bar_min, bar_max)| box_reference(point, *bar_min, *bar_max))
                .fold(f64::INFINITY, f64::min)
        });
    }
}
