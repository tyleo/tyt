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
