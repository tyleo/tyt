use crate::{TyFloatExt, TyQuaternion, ty_array_conversions};
use std::ops::{Add, Div, Mul, Neg, Sub};

/// A 3D vector with component type `T`.
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

ty_array_conversions!(TyVector3, 3, x, y, z);

impl<T: Copy> TyVector3<T> {
    /// A vector with every component set to `value`.
    pub fn splat(value: T) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
        }
    }

    /// The component on `axis`, where `0` is `x`, `1` is `y`, and `2` is `z`.
    ///
    /// # Panics
    /// Panics when `axis` is not `0`, `1`, or `2`.
    pub fn component(&self, axis: usize) -> T {
        self.to_array()[axis]
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

    /// The unnormalized normal of the triangle `a`, `b`, `c`: the cross product
    /// `(b - a) x (c - a)`. Its length is twice the triangle area, and its
    /// direction follows the right-hand rule for the winding.
    pub fn triangle_normal(a: Self, b: Self, c: Self) -> Self {
        (b - a).cross(&(c - a))
    }
}

impl<T: Copy + Div<Output = T>> TyVector3<T> {
    /// Returns the component-wise quotient of `self` and `other`, the companion
    /// to [`componentwise_multiply`](Self::componentwise_multiply).
    pub fn componentwise_divide(&self, other: &Self) -> Self {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }
}

impl<T: Copy + Neg<Output = T>> TyVector3<T> {
    /// This vector rotated from Z-up to Y-up axes, `(x, y, z) -> (x, z, -y)`: a
    /// +90 degree rotation about X. The inverse of
    /// [`yup_to_zup`](Self::yup_to_zup).
    pub fn zup_to_yup(self) -> Self {
        Self::new(self.x, self.z, -self.y)
    }

    /// This vector rotated from Y-up to Z-up axes, `(x, y, z) -> (x, -z, y)`: a
    /// -90 degree rotation about X. The inverse of
    /// [`zup_to_yup`](Self::zup_to_yup).
    pub fn yup_to_zup(self) -> Self {
        Self::new(self.x, -self.z, self.y)
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

impl<T: Copy + Div<Output = T>> Div<T> for TyVector3<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl<T: Neg<Output = T>> Neg for TyVector3<T> {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

/// Implements the float-only vector operations (magnitude and scalar-on-the-left
/// multiplication) for a concrete floating-point component type.
macro_rules! impl_ty_vector3_float {
    ($t:ty) => {
        impl TyVector3<$t> {
            /// A vector with every component positive infinity.
            pub const INFINITY: Self = Self {
                x: <$t>::INFINITY,
                y: <$t>::INFINITY,
                z: <$t>::INFINITY,
            };

            /// A vector with every component negative infinity.
            pub const NEG_INFINITY: Self = Self {
                x: <$t>::NEG_INFINITY,
                y: <$t>::NEG_INFINITY,
                z: <$t>::NEG_INFINITY,
            };

            /// Returns the Euclidean length of this vector.
            pub fn magnitude(&self) -> $t {
                self.magnitude_squared().sqrt()
            }

            /// Returns the squared Euclidean length.
            pub fn magnitude_squared(&self) -> $t {
                self.x * self.x + self.y * self.y + self.z * self.z
            }

            /// Returns the component-wise minimum of `self` and `other`.
            pub fn component_min_with(&self, other: &Self) -> Self {
                Self {
                    x: self.x.min(other.x),
                    y: self.y.min(other.y),
                    z: self.z.min(other.z),
                }
            }

            /// Returns the component-wise maximum of `self` and `other`.
            pub fn component_max_with(&self, other: &Self) -> Self {
                Self {
                    x: self.x.max(other.x),
                    y: self.y.max(other.y),
                    z: self.z.max(other.z),
                }
            }

            /// Quantizes each component into `[0, buckets)` by its position
            /// between the matching components of `low` and `high`, one bucket
            /// index per axis.
            pub fn quantize(&self, low: Self, high: Self, buckets: u32) -> TyVector3<u32> {
                TyVector3::new(
                    self.x.quantize(low.x, high.x, buckets),
                    self.y.quantize(low.y, high.y, buckets),
                    self.z.quantize(low.z, high.z, buckets),
                )
            }

            /// Returns the component-wise absolute value of this vector.
            pub fn abs(&self) -> Self {
                Self {
                    x: self.x.abs(),
                    y: self.y.abs(),
                    z: self.z.abs(),
                }
            }

            /// Rounds each component to the nearest integer, rounding a half away
            /// from zero.
            pub fn round(self) -> Self {
                Self {
                    x: self.x.round(),
                    y: self.y.round(),
                    z: self.z.round(),
                }
            }

            /// Each component cast to `i32`, truncating toward zero. Compose with
            /// [`round`](Self::round) as `v.round().to_i32()` to round to the
            /// nearest integer vector.
            pub fn to_i32(self) -> TyVector3<i32> {
                TyVector3::new(self.x as i32, self.y as i32, self.z as i32)
            }

            /// The zero vector.
            pub const ZERO: Self = Self {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };

            /// The vector with every component `1`.
            pub const ONE: Self = Self {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            };

            /// The unit vector along the `x` axis.
            pub const UNIT_X: Self = Self {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            };

            /// The unit vector along the `y` axis.
            pub const UNIT_Y: Self = Self {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            };

            /// The unit vector along the `z` axis.
            pub const UNIT_Z: Self = Self {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            };

            /// The unit vector along the negative `x` axis.
            pub const UNIT_NEG_X: Self = Self {
                x: -1.0,
                y: 0.0,
                z: 0.0,
            };

            /// The unit vector along the negative `y` axis.
            pub const UNIT_NEG_Y: Self = Self {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            };

            /// The unit vector along the negative `z` axis.
            pub const UNIT_NEG_Z: Self = Self {
                x: 0.0,
                y: 0.0,
                z: -1.0,
            };

            /// A vector with the given `x` and zero `y` and `z`.
            pub fn from_x(x: $t) -> Self {
                Self { x, y: 0.0, z: 0.0 }
            }

            /// A vector with the given `y` and zero `x` and `z`.
            pub fn from_y(y: $t) -> Self {
                Self { x: 0.0, y, z: 0.0 }
            }

            /// A vector with the given `z` and zero `x` and `y`.
            pub fn from_z(z: $t) -> Self {
                Self { x: 0.0, y: 0.0, z }
            }

            /// Returns this vector scaled to unit length. A zero vector yields
            /// non-finite components.
            pub fn normalized(&self) -> Self {
                *self * (1.0 / self.magnitude())
            }

            /// Linearly interpolates from `self` towards `other` by `t`, with `t`
            /// of `0` returning `self` and `1` returning `other`.
            pub fn lerp(self, other: Self, t: $t) -> Self {
                self + (other - self) * t
            }

            /// This vector as a pure quaternion `(x, y, z, 0)`.
            pub fn to_pure_quaternion(self) -> TyQuaternion<$t> {
                TyQuaternion::new(self.x, self.y, self.z, 0.0)
            }

            /// A rotation of `angle` radians about this vector as the axis.
            pub fn rotation_around(self, angle: $t) -> TyQuaternion<$t> {
                TyQuaternion::<$t>::from_axis_angle(self, angle)
            }

            /// This vector's `x`, taken as a uniform scale factor. Assumes the
            /// components are equal.
            pub fn to_scale(self) -> $t {
                self.x
            }

            /// The shortest-arc rotation carrying `self` onto `target`, both taken
            /// to be unit length.
            pub fn rotate_towards(self, target: Self) -> TyQuaternion<$t> {
                const TOLERANCE: $t = 1.0e-5;

                let dot = self.dot(&target);

                // Already aligned: no rotation.
                if dot >= 1.0 - TOLERANCE {
                    return TyQuaternion::<$t>::identity();
                }

                // Nearly opposite: a half turn about any orthogonal axis.
                if dot <= -1.0 + TOLERANCE {
                    let axis = if self.x.abs() <= self.y.abs() && self.x.abs() <= self.z.abs() {
                        Self::UNIT_X
                    } else if self.y.abs() <= self.z.abs() {
                        Self::UNIT_Y
                    } else {
                        Self::UNIT_Z
                    };
                    let axis = self.cross(&axis).normalized();
                    return TyQuaternion::<$t>::from_axis_angle(axis, core::f64::consts::PI as $t);
                }

                let cross = self.cross(&target);
                TyQuaternion::new(cross.x, cross.y, cross.z, 1.0 + dot).normalized()
            }

            /// True when `self` and `other` point in approximately the same
            /// direction, within `tolerance` on the cosine of the angle between
            /// them. Works for non-unit vectors; a near-zero vector is never
            /// approximately equal.
            pub fn is_approximately_equal(&self, other: &Self, tolerance: $t) -> bool {
                const DEGENERATE: $t = 1.0e-8;

                let dot = self.dot(other);
                if dot <= 0.0 {
                    return false;
                }

                let length_squared_a = self.magnitude_squared();
                let length_squared_b = other.magnitude_squared();
                if length_squared_a <= DEGENERATE || length_squared_b <= DEGENERATE {
                    return false;
                }

                let cos_min = 1.0 - tolerance;
                dot * dot >= length_squared_a * length_squared_b * (cos_min * cos_min)
            }

            /// True when `self` and `other`, both taken to be unit length, point
            /// in approximately the same direction, within `tolerance` on their
            /// dot product.
            pub fn is_normalized_approximately_equal(&self, other: &Self, tolerance: $t) -> bool {
                self.dot(other) >= 1.0 - tolerance
            }

            /// The position on the uniform Catmull-Rom spline between `p1` and
            /// `p2`, with neighbours `p0` and `p3`, at parameter `t` in `[0, 1]`.
            pub fn catmull_rom_position(p0: Self, p1: Self, p2: Self, p3: Self, t: $t) -> Self {
                let t2 = t * t;
                let t3 = t2 * t;

                (p1 * 2.0
                    + (p2 - p0) * t
                    + (p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3) * t2
                    + (p1 * 3.0 - p0 - p2 * 3.0 + p3) * t3)
                    * 0.5
            }

            /// The tangent of the uniform Catmull-Rom spline between `p1` and
            /// `p2`, with neighbours `p0` and `p3`, at parameter `t` in `[0, 1]`.
            pub fn catmull_rom_tangent(p0: Self, p1: Self, p2: Self, p3: Self, t: $t) -> Self {
                let t2 = t * t;

                ((p2 - p0)
                    + (p0 * 4.0 - p1 * 10.0 + p2 * 8.0 - p3 * 2.0) * t
                    + (p1 * 9.0 - p0 * 3.0 - p2 * 9.0 + p3 * 3.0) * t2)
                    * 0.5
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

        impl Div<TyVector3<$t>> for $t {
            type Output = TyVector3<$t>;

            /// The component-wise reciprocal scale `self / rhs`, one scalar over
            /// each component.
            fn div(self, rhs: TyVector3<$t>) -> TyVector3<$t> {
                TyVector3 {
                    x: self / rhs.x,
                    y: self / rhs.y,
                    z: self / rhs.z,
                }
            }
        }
    };
}

impl_ty_vector3_float!(f32);
impl_ty_vector3_float!(f64);

/// Implements the ordered-integer vector operations for a concrete integer
/// component type. The component min/max mirror the float macro's, but live on
/// the concrete integer types rather than a generic `impl<T: Ord>`: the compiler
/// cannot rule out a future `Ord` impl for `f64`, so a generic version would be
/// treated as a duplicate of the float methods (E0592).
macro_rules! impl_ty_vector3_int {
    ($t:ty) => {
        impl TyVector3<$t> {
            /// Returns the component-wise minimum of `self` and `other`.
            pub fn component_min_with(&self, other: &Self) -> Self {
                Self {
                    x: self.x.min(other.x),
                    y: self.y.min(other.y),
                    z: self.z.min(other.z),
                }
            }

            /// Returns the component-wise maximum of `self` and `other`.
            pub fn component_max_with(&self, other: &Self) -> Self {
                Self {
                    x: self.x.max(other.x),
                    y: self.y.max(other.y),
                    z: self.z.max(other.z),
                }
            }
        }
    };
}

impl_ty_vector3_int!(i32);
impl_ty_vector3_int!(u32);

impl TyVector3<i32> {
    /// Each component widened to `f64`. The lossless inverse of the truncating
    /// `to_i32` cast on a float vector.
    pub fn to_f64(self) -> TyVector3<f64> {
        TyVector3::new(self.x as f64, self.y as f64, self.z as f64)
    }

    /// Each component cast to `u32` with `as`, wrapping negative values.
    pub fn to_u32(self) -> TyVector3<u32> {
        TyVector3::new(self.x as u32, self.y as u32, self.z as u32)
    }
}

impl TyVector3<u32> {
    /// Each component widened to `f64`, losslessly.
    pub fn to_f64(self) -> TyVector3<f64> {
        TyVector3::new(self.x as f64, self.y as f64, self.z as f64)
    }

    /// Each component cast to `i32` with `as`, wrapping values above `i32::MAX`.
    pub fn to_i32(self) -> TyVector3<i32> {
        TyVector3::new(self.x as i32, self.y as i32, self.z as i32)
    }
}

#[cfg(test)]
mod tests {
    use crate::{TyVector3F64, TyVector3I32, TyVector3U32};

    #[test]
    fn round_then_to_i32_rounds_to_the_nearest_integer_vector() {
        // round is half-away-from-zero, then to_i32 truncates the exact integers.
        let rounded = TyVector3F64::new(1.4, 2.5, -0.6).round().to_i32();
        assert_eq!(rounded, TyVector3I32::new(1, 3, -1));
    }

    #[test]
    fn to_i32_truncates_toward_zero() {
        let truncated = TyVector3F64::new(1.9, -1.9, 0.5).to_i32();
        assert_eq!(truncated, TyVector3I32::new(1, -1, 0));
    }

    #[test]
    fn to_f64_widens_each_component() {
        let widened = TyVector3I32::new(1, -2, 3).to_f64();
        assert_eq!(widened, TyVector3F64::new(1.0, -2.0, 3.0));
    }

    #[test]
    fn u32_to_f64_widens_each_component() {
        let widened = TyVector3U32::new(1, 2, 3).to_f64();
        assert_eq!(widened, TyVector3F64::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn zup_yup_axis_rotations_are_inverses() {
        let v = TyVector3F64::new(1.0, 2.0, 3.0);
        assert_eq!(v.zup_to_yup(), TyVector3F64::new(1.0, 3.0, -2.0));
        assert_eq!(v.yup_to_zup(), TyVector3F64::new(1.0, -3.0, 2.0));
        assert_eq!(v.zup_to_yup().yup_to_zup(), v);
    }

    #[test]
    fn triangle_normal_follows_the_winding() {
        // A counter-clockwise triangle in the z = 0 plane has a +z normal whose
        // length is twice its area.
        let normal = TyVector3F64::triangle_normal(
            TyVector3F64::new(0.0, 0.0, 0.0),
            TyVector3F64::new(2.0, 0.0, 0.0),
            TyVector3F64::new(0.0, 2.0, 0.0),
        );
        assert_eq!(normal, TyVector3F64::new(0.0, 0.0, 4.0));
    }

    #[test]
    fn componentwise_multiply_is_the_hadamard_product() {
        let product = TyVector3F64::new(2.0, 3.0, 4.0)
            .componentwise_multiply(&TyVector3F64::new(5.0, 6.0, 7.0));
        assert_eq!(product, TyVector3F64::new(10.0, 18.0, 28.0));
    }

    #[test]
    fn componentwise_divide_is_the_per_axis_quotient() {
        let quotient = TyVector3F64::new(10.0, 18.0, 28.0)
            .componentwise_divide(&TyVector3F64::new(5.0, 6.0, 7.0));
        assert_eq!(quotient, TyVector3F64::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn abs_takes_each_component() {
        let abs = TyVector3F64::new(-1.0, 2.0, -3.0).abs();
        assert_eq!(abs, TyVector3F64::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn integer_component_min_and_max_with_fold_each_axis() {
        let a = TyVector3U32::new(3, 1, 4);
        let b = TyVector3U32::new(1, 5, 9);
        assert_eq!(a.component_min_with(&b), TyVector3U32::new(1, 1, 4));
        assert_eq!(a.component_max_with(&b), TyVector3U32::new(3, 5, 9));

        // Signed integers fold the same way, including across zero.
        let c = TyVector3I32::new(-2, 7, -5);
        let d = TyVector3I32::new(3, -1, -9);
        assert_eq!(c.component_min_with(&d), TyVector3I32::new(-2, -1, -9));
        assert_eq!(c.component_max_with(&d), TyVector3I32::new(3, 7, -5));
    }

    #[test]
    fn integer_casts_between_i32_and_u32_are_componentwise() {
        assert_eq!(
            TyVector3U32::new(1, 2, 3).to_i32(),
            TyVector3I32::new(1, 2, 3)
        );
        assert_eq!(
            TyVector3I32::new(4, 5, 6).to_u32(),
            TyVector3U32::new(4, 5, 6)
        );
        // `as` wraps, matching the hand-rolled casts these replace.
        assert_eq!(TyVector3I32::new(-1, 0, 7).to_u32().x, u32::MAX);
    }

    #[test]
    fn scalar_over_vector_is_the_component_wise_reciprocal() {
        // The inverse-scale of a per-axis scale vector.
        let inverse = 1.0 / TyVector3F64::new(2.0, 4.0, 8.0);
        assert_eq!(inverse, TyVector3F64::new(0.5, 0.25, 0.125));
    }
}
