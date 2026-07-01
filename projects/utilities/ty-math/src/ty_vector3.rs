use std::ops::{Add, Mul, Sub};

/// A 3D vector generic over its component type `T`.
///
/// The component type defaults to `f64`, so `TyVector3` is the `f64` vector;
/// see `TyVector3F32`, `TyVector3F64`, `TyVector3I32`, and `TyVector3U32` for
/// the common instantiations.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TyVector3<T = f64> {
    /// The `x` component.
    pub x: T,

    /// The `y` component.
    pub y: T,

    /// The `z` component.
    pub z: T,
}

impl<T> TyVector3<T> {
    /// Creates a new vector from `x`, `y`, and `z` components.
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: Add<Output = T> + Copy + Mul<Output = T> + Sub<Output = T>> TyVector3<T> {
    /// Returns the cross product of `self` and `other`.
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Returns the dot product of `self` and `other`.
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Returns the component-wise (Hadamard) product of `self` and `other`.
    pub fn componentwise_multiply(&self, other: &Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl<T: Add<Output = T>> Add for TyVector3<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T: Sub<Output = T>> Sub for TyVector3<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T: Copy + Mul<Output = T>> Mul<T> for TyVector3<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

/// Implements the float-only vector operations (magnitude and scalar-on-the-left
/// multiplication) for a concrete floating-point component type.
macro_rules! impl_ty_vector3_float {
    ($t:ty) => {
        impl TyVector3<$t> {
            /// Returns the Euclidean length of this vector.
            pub fn magnitude(&self) -> $t {
                (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
            }

            /// Returns the component-wise absolute value of this vector.
            pub fn abs(&self) -> Self {
                Self {
                    x: self.x.abs(),
                    y: self.y.abs(),
                    z: self.z.abs(),
                }
            }
        }

        impl Mul<TyVector3<$t>> for $t {
            type Output = TyVector3<$t>;

            fn mul(self, rhs: TyVector3<$t>) -> TyVector3<$t> {
                TyVector3 {
                    x: self * rhs.x,
                    y: self * rhs.y,
                    z: self * rhs.z,
                }
            }
        }
    };
}

impl_ty_vector3_float!(f32);
impl_ty_vector3_float!(f64);

#[cfg(test)]
mod tests {
    use crate::TyVector3F64;

    #[test]
    fn componentwise_multiply_is_the_hadamard_product() {
        let product = TyVector3F64::new(2.0, 3.0, 4.0)
            .componentwise_multiply(&TyVector3F64::new(5.0, 6.0, 7.0));
        assert_eq!(product, TyVector3F64::new(10.0, 18.0, 28.0));
    }

    #[test]
    fn abs_takes_each_component() {
        let abs = TyVector3F64::new(-1.0, 2.0, -3.0).abs();
        assert_eq!(abs, TyVector3F64::new(1.0, 2.0, 3.0));
    }
}
