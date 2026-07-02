use crate::{TyVector3, ty_array_conversions};
use std::ops::{Add, Div, Mul, Neg, Sub};

/// A 4D vector with component type `T`, the column type of
/// [`TyMatrix4x4`](crate::TyMatrix4x4).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TyVector4<T = f64> {
    /// The `x` component.
    pub x: T,

    /// The `y` component.
    pub y: T,

    /// The `z` component.
    pub z: T,

    /// The `w` component.
    pub w: T,
}

impl<T> TyVector4<T> {
    /// Creates a new vector from `x`, `y`, `z`, and `w` components.
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { x, y, z, w }
    }
}

ty_array_conversions!(TyVector4, 4, x, y, z, w);

impl<T: Copy> TyVector4<T> {
    /// A vector with every component set to `value`.
    pub fn splat(value: T) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
            w: value,
        }
    }

    /// A vector from a [`TyVector3`] and a `w` component.
    pub fn from_vector3(vector: TyVector3<T>, w: T) -> Self {
        Self {
            x: vector.x,
            y: vector.y,
            z: vector.z,
            w,
        }
    }

    /// The component on `axis`, where `0` is `x`, `1` is `y`, `2` is `z`, and `3`
    /// is `w`.
    ///
    /// # Panics
    /// Panics when `axis` is not in `0..4`.
    pub fn component(&self, axis: usize) -> T {
        self.to_array()[axis]
    }

    /// This vector's `x`, `y`, and `z` as a [`TyVector3`], dropping `w`.
    pub fn truncate(&self) -> TyVector3<T> {
        TyVector3::new(self.x, self.y, self.z)
    }
}

impl<T: Add<Output = T> + Copy + Mul<Output = T>> TyVector4<T> {
    /// Returns the dot product of `self` and `other`.
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    /// Returns the component-wise (Hadamard) product of `self` and `other`.
    pub fn componentwise_multiply(&self, other: &Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
            w: self.w * other.w,
        }
    }
}

impl<T: Add<Output = T>> Add for TyVector4<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl<T: Sub<Output = T>> Sub for TyVector4<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl<T: Neg<Output = T>> Neg for TyVector4<T> {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

impl<T: Copy + Mul<Output = T>> Mul<T> for TyVector4<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}

impl<T: Copy + Div<Output = T>> Div<T> for TyVector4<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            w: self.w / rhs,
        }
    }
}

/// Implements the float-only vector operations (magnitude, normalization, and
/// scalar-on-the-left multiplication) for a concrete floating-point component
/// type.
macro_rules! impl_ty_vector4_float {
    ($t:ty) => {
        impl TyVector4<$t> {
            /// The zero vector.
            pub const ZERO: Self = Self {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            };

            /// The vector with every component `1`.
            pub const ONE: Self = Self {
                x: 1.0,
                y: 1.0,
                z: 1.0,
                w: 1.0,
            };

            /// Returns the Euclidean length of this vector.
            pub fn magnitude(&self) -> $t {
                self.magnitude_squared().sqrt()
            }

            /// Returns the squared Euclidean length.
            pub fn magnitude_squared(&self) -> $t {
                self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
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
                    z: self.z.abs(),
                    w: self.w.abs(),
                }
            }
        }

        impl Mul<TyVector4<$t>> for $t {
            type Output = TyVector4<$t>;

            fn mul(self, rhs: TyVector4<$t>) -> TyVector4<$t> {
                rhs * self
            }
        }

        impl Div<TyVector4<$t>> for $t {
            type Output = TyVector4<$t>;

            /// The component-wise reciprocal scale `self / rhs`, one scalar over
            /// each component.
            fn div(self, rhs: TyVector4<$t>) -> TyVector4<$t> {
                TyVector4::new(self / rhs.x, self / rhs.y, self / rhs.z, self / rhs.w)
            }
        }
    };
}

impl_ty_vector4_float!(f32);
impl_ty_vector4_float!(f64);

#[cfg(test)]
mod tests {
    use crate::{TyVector3F64, TyVector4F64};

    #[test]
    fn truncate_drops_w() {
        let truncated = TyVector4F64::new(1.0, 2.0, 3.0, 4.0).truncate();
        assert_eq!(truncated, TyVector3F64::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn from_vector3_carries_w() {
        let vector = TyVector4F64::from_vector3(TyVector3F64::new(1.0, 2.0, 3.0), 4.0);
        assert_eq!(vector, TyVector4F64::new(1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn dot_sums_the_component_products() {
        let dot = TyVector4F64::new(1.0, 2.0, 3.0, 4.0).dot(&TyVector4F64::new(5.0, 6.0, 7.0, 8.0));
        assert_eq!(dot, 5.0 + 12.0 + 21.0 + 32.0);
    }

    #[test]
    fn scalar_multiplication_commutes() {
        let vector = TyVector4F64::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(vector * 2.0, 2.0 * vector);
    }

    #[test]
    fn from_array_and_to_array_round_trip() {
        let array = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(TyVector4F64::from_array(array).to_array(), array);
    }

    #[test]
    fn from_slice_reads_the_first_components_and_write_to_slice_round_trips() {
        // A buffer longer than the vector: from_slice takes the leading four.
        let buffer = [1.0, 2.0, 3.0, 4.0, 5.0];
        let vector = TyVector4F64::from_slice(&buffer);
        assert_eq!(vector, TyVector4F64::new(1.0, 2.0, 3.0, 4.0));

        let mut out = [0.0; 4];
        vector.write_to_slice(&mut out);
        assert_eq!(out, [1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn array_converts_via_from_and_into() {
        let vector: TyVector4F64 = [1.0, 2.0, 3.0, 4.0].into();
        assert_eq!(vector, TyVector4F64::new(1.0, 2.0, 3.0, 4.0));
    }
}
