use crate::operations::sdf_doc::ArcSpan;
use sdfcore::SdfCaps;
use ty_math::TyVector2F64;

/// The signed distance to the point at `offset` from the band along the circle
/// of `radius` around the origin within `span`, reaching `half_width` to each
/// side. Round caps close the band with half circles, and flat caps cut it
/// square.
pub fn arc_distance(
    offset: TyVector2F64,
    radius: f64,
    half_width: f64,
    span: &ArcSpan,
    caps: SdfCaps,
) -> f64 {
    let folded = span.fold(offset);

    match caps {
        SdfCaps::Flat => {
            let p = span.to_end_frame(folded);
            let band = (p.length() - radius).abs() - half_width;
            let cap = TyVector2F64::new(p.x, ((radius - p.y).abs() - half_width).max(0.0)).length();

            band.max(if p.x < 0.0 { -cap } else { cap })
        }

        SdfCaps::Round => {
            let to_curve = if span.is_past_end(folded) {
                (folded - span.end(radius)).length()
            } else {
                (folded.length() - radius).abs()
            };

            to_curve - half_width
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{arc_curve_distance, assert_shape2d, segment_distance2d};
    use sdfcore::{SdfCaps, SdfShape2d};
    use ty_math::TyVector2F64;

    const CENTER: TyVector2F64 = TyVector2F64::new(0.1, -0.1);

    const GRID: (TyVector2F64, TyVector2F64, u32) =
        (TyVector2F64::splat(-1.0), TyVector2F64::splat(1.0), 17);

    fn arc(from_degrees: f64, to_degrees: f64, caps: Option<SdfCaps>) -> SdfShape2d {
        SdfShape2d::Arc {
            center: CENTER,
            radius: 0.6,
            from_degrees,
            to_degrees,
            width: 0.2,
            caps,
        }
    }

    #[test]
    fn round_caps_close_the_band_with_half_circles() {
        for (from_degrees, to_degrees) in [(0.0, 90.0), (-30.0, 200.0), (10.0, 370.0), (45.0, 50.0)]
        {
            assert_shape2d(
                vec![arc(from_degrees, to_degrees, None)],
                GRID,
                1e-12,
                |point| arc_curve_distance(point, CENTER, 0.6, from_degrees, to_degrees) - 0.1,
            );
        }
    }

    #[test]
    fn flat_caps_cut_the_band_square() {
        for (from_degrees, to_degrees) in [(0.0, 90.0), (-30.0, 200.0), (45.0, 50.0)] {
            let end = |degrees: f64, radius: f64| {
                let (sin, cos) = degrees.to_radians().sin_cos();
                CENTER + TyVector2F64::new(cos, sin) * radius
            };

            assert_shape2d(
                vec![arc(from_degrees, to_degrees, Some(SdfCaps::Flat))],
                GRID,
                1e-12,
                |point| {
                    let to_outline = [
                        arc_curve_distance(point, CENTER, 0.7, from_degrees, to_degrees),
                        arc_curve_distance(point, CENTER, 0.5, from_degrees, to_degrees),
                        segment_distance2d(point, end(from_degrees, 0.5), end(from_degrees, 0.7)),
                        segment_distance2d(point, end(to_degrees, 0.5), end(to_degrees, 0.7)),
                    ]
                    .into_iter()
                    .fold(f64::INFINITY, f64::min);

                    let offset = point - CENTER;
                    let angle = offset.y.atan2(offset.x).to_degrees();
                    let within =
                        (angle - from_degrees).rem_euclid(360.0) <= to_degrees - from_degrees;
                    let inside = within && (0.5..=0.7).contains(&offset.length());

                    if inside { -to_outline } else { to_outline }
                },
            );
        }
    }
}
