use crate::{TyVector3, ty_array_conversions};
use std::ops::{Add, Div, Mul, Neg, Sub};

/// A 2D vector with component type `T`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TyVector2<T = f64> {
    /// The `x` component.
    pub x: T,

    /// The `y` component.
    pub y: T,
}

impl<T> TyVector2<T> {
    /// Creates a new vector from `x` and `y` components.
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

ty_array_conversions!(TyVector2, 2, x, y);

impl<T: Copy> TyVector2<T> {
    /// A vector with both components set to `value`.
    pub fn splat(value: T) -> Self {
        Self { x: value, y: value }
    }

    /// The component on `axis`, where `0` is `x` and `1` is `y`.
    ///
    /// # Panics
    /// Panics when `axis` is not `0` or `1`.
    pub fn component(&self, axis: usize) -> T {
        self.to_array()[axis]
    }
}

impl<T: Add<Output = T> + Copy + Mul<Output = T> + Sub<Output = T>> TyVector2<T> {
    /// Returns the dot product of `self` and `other`.
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y
    }

    /// Returns the 2D cross product (perp-dot) of `self` and `other`, the signed
    /// area of the parallelogram they span.
    pub fn cross(&self, other: &Self) -> T {
        self.x * other.y - self.y * other.x
    }

    /// Returns the component-wise (Hadamard) product of `self` and `other`.
    pub fn componentwise_multiply(&self, other: &Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl<T: Add<Output = T>> Add for TyVector2<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: Sub<Output = T>> Sub for TyVector2<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Neg<Output = T>> Neg for TyVector2<T> {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T: Copy + Mul<Output = T>> Mul<T> for TyVector2<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T: Copy + Div<Output = T>> Div<T> for TyVector2<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

/// Implements the float-only vector operations for a concrete floating-point
/// component type.
macro_rules! impl_ty_vector2_float {
    ($t:ty) => {
        impl TyVector2<$t> {
            /// The zero vector.
            pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

            /// The vector with both components `1`.
            pub const ONE: Self = Self { x: 1.0, y: 1.0 };

            /// The unit vector along the `x` axis.
            pub const UNIT_X: Self = Self { x: 1.0, y: 0.0 };

            /// The unit vector along the `y` axis.
            pub const UNIT_Y: Self = Self { x: 0.0, y: 1.0 };

            /// Returns the Euclidean length of this vector.
            pub fn magnitude(&self) -> $t {
                self.magnitude_squared().sqrt()
            }

            /// Returns the squared Euclidean length.
            pub fn magnitude_squared(&self) -> $t {
                self.x * self.x + self.y * self.y
            }

            /// Returns this vector scaled to unit length. A zero vector yields
            /// non-finite components.
            pub fn normalized(&self) -> Self {
                *self * (1.0 / self.magnitude())
            }

            /// Returns the component-wise absolute value of this vector.
            pub fn abs(&self) -> Self {
                Self {
                    x: self.x.abs(),
                    y: self.y.abs(),
                }
            }

            /// Returns the component-wise minimum of `self` and `other`.
            pub fn component_min_with(&self, other: &Self) -> Self {
                Self {
                    x: self.x.min(other.x),
                    y: self.y.min(other.y),
                }
            }

            /// Returns the component-wise maximum of `self` and `other`.
            pub fn component_max_with(&self, other: &Self) -> Self {
                Self {
                    x: self.x.max(other.x),
                    y: self.y.max(other.y),
                }
            }

            /// This vector as a [`TyVector3`] with a `z` of `0`.
            pub fn to_vector3(&self) -> TyVector3<$t> {
                TyVector3::new(self.x, self.y, 0.0)
            }
        }

        impl Mul<TyVector2<$t>> for $t {
            type Output = TyVector2<$t>;

            fn mul(self, rhs: TyVector2<$t>) -> TyVector2<$t> {
                rhs * self
            }
        }

        impl Div<TyVector2<$t>> for $t {
            type Output = TyVector2<$t>;

            /// The component-wise reciprocal scale `self / rhs`, one scalar over
            /// each component.
            fn div(self, rhs: TyVector2<$t>) -> TyVector2<$t> {
                TyVector2::new(self / rhs.x, self / rhs.y)
            }
        }
    };
}

impl_ty_vector2_float!(f32);
impl_ty_vector2_float!(f64);

#[cfg(test)]
mod tests {
    use crate::{TyVector2F64, TyVector3F64};

    #[test]
    fn cross_is_the_signed_area() {
        // +x cross +y is +1; +y cross +x is -1.
        assert_eq!(TyVector2F64::UNIT_X.cross(&TyVector2F64::UNIT_Y), 1.0);
        assert_eq!(TyVector2F64::UNIT_Y.cross(&TyVector2F64::UNIT_X), -1.0);
    }

    #[test]
    fn dot_sums_the_component_products() {
        assert_eq!(
            TyVector2F64::new(1.0, 2.0).dot(&TyVector2F64::new(3.0, 4.0)),
            11.0
        );
    }

    #[test]
    fn to_vector3_zeroes_z() {
        assert_eq!(
            TyVector2F64::new(1.0, 2.0).to_vector3(),
            TyVector3F64::new(1.0, 2.0, 0.0)
        );
    }

    #[test]
    fn scalar_multiplication_commutes() {
        let vector = TyVector2F64::new(1.0, 2.0);
        assert_eq!(vector * 2.0, 2.0 * vector);
    }
}
