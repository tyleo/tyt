use crate::TyVector3;
use std::ops::{Add, Mul, Sub};

/// An axis-aligned bounding box with component type `T`, a center and per-axis
/// half-extents.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TyBounds<T = f64> {
    /// The center of the box.
    pub center: TyVector3<T>,

    /// The half-size on each axis.
    pub extents: TyVector3<T>,
}

impl<T> TyBounds<T> {
    /// Creates a box from its `center` and `extents`.
    pub fn new(center: TyVector3<T>, extents: TyVector3<T>) -> Self {
        Self { center, extents }
    }
}

impl<T: Add<Output = T> + Copy> TyBounds<T> {
    /// The maximum corner, `center + extents`.
    pub fn max(&self) -> TyVector3<T> {
        self.center + self.extents
    }
}

impl<T: Sub<Output = T> + Copy> TyBounds<T> {
    /// The minimum corner, `center - extents`.
    pub fn min(&self) -> TyVector3<T> {
        self.center - self.extents
    }
}

impl<T: Copy + Mul<Output = T>> TyBounds<T> {
    /// This box scaled about the origin by `value`.
    pub fn scale(&self, value: T) -> Self {
        Self {
            center: self.center * value,
            extents: self.extents * value,
        }
    }
}

/// Implements the float-only box operations for a concrete floating-point
/// component type.
macro_rules! impl_ty_bounds_float {
    ($t:ty) => {
        impl TyBounds<$t> {
            /// The smallest box containing every point in `points`, or `None`
            /// when the iterator is empty.
            pub fn from_points(points: impl IntoIterator<Item = TyVector3<$t>>) -> Option<Self> {
                let mut points = points.into_iter();
                let first = points.next()?;
                let (min, max) = points.fold((first, first), |(min, max), point| {
                    (
                        min.component_min_with(&point),
                        max.component_max_with(&point),
                    )
                });

                Some(Self {
                    center: (min + max) * 0.5,
                    extents: (max - min) * 0.5,
                })
            }

            /// A box with minimum corner `min` and full `size` on each axis.
            pub fn from_min_size(min: TyVector3<$t>, size: TyVector3<$t>) -> Self {
                let extents = size * 0.5;
                Self {
                    center: min + extents,
                    extents,
                }
            }

            /// The full size on each axis, `extents * 2`.
            pub fn size(&self) -> TyVector3<$t> {
                self.extents * 2.0
            }

            /// The smallest box containing both `self` and `other`.
            pub fn encapsulate(&self, other: &Self) -> Self {
                let min = self.min().component_min_with(&other.min());
                let max = self.max().component_max_with(&other.max());

                Self {
                    center: (min + max) * 0.5,
                    extents: (max - min) * 0.5,
                }
            }
        }
    };
}

impl_ty_bounds_float!(f32);
impl_ty_bounds_float!(f64);

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
