use crate::operations::sdf_doc::spline_points;
use ty_math::{TyVector3F64, TyVector4F64};

/// A point along the curve of a `sweep`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SweepSample {
    /// Where the curve runs.
    pub position: TyVector3F64,

    /// The profile's scale.
    pub scale: f64,

    /// The curve's unit direction.
    pub tangent: TyVector3F64,
}

impl SweepSample {
    /// The samples along the smooth curve through `points`, with the profile's
    /// scale running smoothly through `scales` or holding 1 without them.
    pub fn along(points: &[TyVector3F64], scales: Option<&[f64]>) -> Vec<Self> {
        let controls: Vec<TyVector4F64> = points
            .iter()
            .enumerate()
            .map(|(index, point)| point.extend(scales.map_or(1.0, |scales| scales[index])))
            .collect();
        let curve = spline_points(&controls, false, |a, b| a.truncate().distance(b.truncate()));
        let last = curve.len() - 1;

        (0..=last)
            .map(|index| {
                let before = curve[index.saturating_sub(1)];
                let after = curve[(index + 1).min(last)];

                SweepSample {
                    position: curve[index].truncate(),
                    scale: curve[index].w,
                    tangent: (after - before).truncate().normalize(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::SweepSample;
    use ty_math::TyVector3F64;

    #[test]
    fn a_sweep_s_samples_run_through_each_point_and_scale() {
        let points = [
            TyVector3F64::new(0.0, 0.0, 0.0),
            TyVector3F64::new(1.0, 0.5, 0.0),
            TyVector3F64::new(2.0, 0.0, 1.0),
        ];
        let samples = SweepSample::along(&points, Some(&[1.0, 0.5, 0.0]));

        assert_eq!(samples.len(), 2 * 16 + 1);
        for (index, point) in points.iter().enumerate() {
            assert_eq!(samples[index * 16].position, *point);
        }
        assert_eq!(samples[16].scale, 0.5);
        assert_eq!(samples[32].scale, 0.0);
        assert!(samples.iter().all(|sample| sample.scale >= 0.0));
        assert!(
            samples
                .iter()
                .all(|sample| (sample.tangent.length() - 1.0).abs() < 1e-12)
        );
    }

    #[test]
    fn a_sweep_without_scales_holds_scale_1() {
        let samples = SweepSample::along(
            &[TyVector3F64::ZERO, TyVector3F64::new(0.0, 0.0, 2.0)],
            None,
        );

        assert!(
            samples
                .iter()
                .all(|sample| (sample.scale - 1.0).abs() < 1e-12)
        );
        assert!(
            samples
                .iter()
                .all(|sample| sample.tangent == TyVector3F64::Z)
        );
    }
}
