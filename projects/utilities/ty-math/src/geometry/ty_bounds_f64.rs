use crate::impl_ty_bounds;
use glam::DVec3;

impl_ty_bounds!(TyBoundsF64, DVec3, f64);

#[cfg(test)]
mod tests {
    use crate::{TyBoundsF64, TyVector3F64};

    #[test]
    fn from_points_bounds_the_cloud_and_size_is_the_full_extent() {
        let bounds = TyBoundsF64::from_points([
            TyVector3F64::new(-1.0, 0.0, 2.0),
            TyVector3F64::new(3.0, -2.0, 2.0),
            TyVector3F64::new(1.0, 4.0, 5.0),
        ])
        .expect("a non-empty cloud");
        assert_eq!(bounds.min(), TyVector3F64::new(-1.0, -2.0, 2.0));
        assert_eq!(bounds.max(), TyVector3F64::new(3.0, 4.0, 5.0));
        assert_eq!(bounds.size(), TyVector3F64::new(4.0, 6.0, 3.0));
    }

    #[test]
    fn from_min_size_sets_the_min_corner_and_full_size() {
        let bounds = TyBoundsF64::from_min_size(
            TyVector3F64::new(-1.0, -2.0, 2.0),
            TyVector3F64::new(4.0, 6.0, 3.0),
        );
        assert_eq!(bounds.center, TyVector3F64::new(1.0, 1.0, 3.5));
        assert_eq!(bounds.min(), TyVector3F64::new(-1.0, -2.0, 2.0));
        assert_eq!(bounds.max(), TyVector3F64::new(3.0, 4.0, 5.0));
        assert_eq!(bounds.size(), TyVector3F64::new(4.0, 6.0, 3.0));
    }

    #[test]
    fn from_points_of_an_empty_cloud_is_none() {
        assert_eq!(
            TyBoundsF64::from_points(core::iter::empty::<TyVector3F64>()),
            None
        );
    }

    #[test]
    fn min_and_max_are_the_corners() {
        let bounds = TyBoundsF64::new(TyVector3F64::new(1.0, 2.0, 3.0), TyVector3F64::ONE);
        assert_eq!(bounds.min(), TyVector3F64::new(0.0, 1.0, 2.0));
        assert_eq!(bounds.max(), TyVector3F64::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn encapsulate_covers_both_boxes() {
        let a = TyBoundsF64::new(TyVector3F64::ZERO, TyVector3F64::ONE);
        let b = TyBoundsF64::new(TyVector3F64::new(4.0, 0.0, 0.0), TyVector3F64::ONE);
        // a spans x in [-1, 1], b spans x in [3, 5], so the union spans [-1, 5].
        let union = a.encapsulate(&b);
        assert_eq!(union.min(), TyVector3F64::new(-1.0, -1.0, -1.0));
        assert_eq!(union.max(), TyVector3F64::new(5.0, 1.0, 1.0));
    }

    #[test]
    fn scale_grows_center_and_extents() {
        let bounds =
            TyBoundsF64::new(TyVector3F64::new(1.0, 2.0, 3.0), TyVector3F64::ONE).scale(2.0);
        assert_eq!(bounds.center, TyVector3F64::new(2.0, 4.0, 6.0));
        assert_eq!(bounds.extents, TyVector3F64::new(2.0, 2.0, 2.0));
    }
}
