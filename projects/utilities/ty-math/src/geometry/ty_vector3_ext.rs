use crate::{TyFloatExt, TyVector3U32};
use glam::{DQuat, DVec3, Quat, Vec3};

/// The tyt-specific 3D vector operations glam does not provide, carried on the
/// glam vector aliases so callers keep `v.foo()` with `use ty_math::TyVector3Ext`.
pub trait TyVector3Ext {
    /// The scalar component type, `f64` or `f32`.
    type Scalar;

    /// The matching quaternion type, [`DQuat`](glam::DQuat) or [`Quat`](glam::Quat).
    type Quaternion;

    /// The unnormalized normal of triangle `a`, `b`, `c`, the cross product
    /// `(b - a) x (c - a)`. Its length is twice the triangle area and its
    /// direction follows the right-hand rule for the winding.
    fn triangle_normal(a: Self, b: Self, c: Self) -> Self;

    /// This vector rotated from Z-up to Y-up axes, `(x, y, z) -> (x, z, -y)`: a
    /// +90 degree rotation about X. The inverse of [`yup_to_zup`](Self::yup_to_zup).
    fn zup_to_yup(self) -> Self;

    /// This vector rotated from Y-up to Z-up axes, `(x, y, z) -> (x, -z, y)`: a
    /// -90 degree rotation about X. The inverse of [`zup_to_yup`](Self::zup_to_yup).
    fn yup_to_zup(self) -> Self;

    /// A vector with the given `x` and zero `y` and `z`.
    fn from_x(x: Self::Scalar) -> Self;

    /// A vector with the given `y` and zero `x` and `z`.
    fn from_y(y: Self::Scalar) -> Self;

    /// A vector with the given `z` and zero `x` and `y`.
    fn from_z(z: Self::Scalar) -> Self;

    /// This vector's `x`, taken as a uniform scale factor. Assumes the components
    /// are equal.
    fn to_scale(self) -> Self::Scalar;

    /// This vector as a pure quaternion `(x, y, z, 0)`.
    fn to_pure_quaternion(self) -> Self::Quaternion;

    /// A rotation of `angle` radians about this vector as the axis, which is
    /// normalized first.
    fn rotation_around(self, angle: Self::Scalar) -> Self::Quaternion;

    /// The shortest-arc rotation carrying `self` onto `target`, both taken to be
    /// unit length. Named to avoid glam's inherent `rotate_towards`, which rotates
    /// a vector toward another by a capped angle.
    fn rotation_towards(self, target: Self) -> Self::Quaternion;

    /// True when `self` and `other` point in approximately the same direction,
    /// within `tolerance` on the cosine of the angle between them. Works for
    /// non-unit vectors; a near-zero vector is never approximately equal.
    // `self` by value matches glam's Copy convention; the trait cannot express
    // `Self: Copy` for the lint to see it.
    #[allow(clippy::wrong_self_convention)]
    fn is_approximately_equal(self, other: Self, tolerance: Self::Scalar) -> bool;

    /// True when `self` and `other`, both taken to be unit length, point in
    /// approximately the same direction, within `tolerance` on their dot product.
    #[allow(clippy::wrong_self_convention)]
    fn is_normalized_approximately_equal(self, other: Self, tolerance: Self::Scalar) -> bool;

    /// Each component quantized into `[0, buckets)` by its position between the
    /// matching components of `low` and `high`, one bucket index per axis.
    fn quantize(self, low: Self, high: Self, buckets: u32) -> TyVector3U32;

    /// The position on the uniform Catmull-Rom spline between `p1` and `p2`, with
    /// neighbours `p0` and `p3`, at parameter `t` in `[0, 1]`.
    fn catmull_rom_position(p0: Self, p1: Self, p2: Self, p3: Self, t: Self::Scalar) -> Self;

    /// The tangent of the uniform Catmull-Rom spline between `p1` and `p2`, with
    /// neighbours `p0` and `p3`, at parameter `t` in `[0, 1]`.
    fn catmull_rom_tangent(p0: Self, p1: Self, p2: Self, p3: Self, t: Self::Scalar) -> Self;
}

/// Implements [`TyVector3Ext`] for a concrete glam vector, scalar, and quaternion.
macro_rules! impl_ty_vector3_ext {
    ($vec:ty, $scalar:ty, $quat:ty) => {
        impl TyVector3Ext for $vec {
            type Scalar = $scalar;
            type Quaternion = $quat;

            fn triangle_normal(a: Self, b: Self, c: Self) -> Self {
                (b - a).cross(c - a)
            }

            fn zup_to_yup(self) -> Self {
                Self::new(self.x, self.z, -self.y)
            }

            fn yup_to_zup(self) -> Self {
                Self::new(self.x, -self.z, self.y)
            }

            fn from_x(x: $scalar) -> Self {
                Self::new(x, 0.0, 0.0)
            }

            fn from_y(y: $scalar) -> Self {
                Self::new(0.0, y, 0.0)
            }

            fn from_z(z: $scalar) -> Self {
                Self::new(0.0, 0.0, z)
            }

            fn to_scale(self) -> $scalar {
                self.x
            }

            fn to_pure_quaternion(self) -> $quat {
                <$quat>::from_xyzw(self.x, self.y, self.z, 0.0)
            }

            fn rotation_around(self, angle: $scalar) -> $quat {
                <$quat>::from_axis_angle(self.normalize(), angle)
            }

            fn rotation_towards(self, target: Self) -> $quat {
                <$quat>::from_rotation_arc(self, target)
            }

            fn is_approximately_equal(self, other: Self, tolerance: $scalar) -> bool {
                const DEGENERATE: $scalar = 1.0e-8;

                let dot = self.dot(other);
                if dot <= 0.0 {
                    return false;
                }

                let length_squared_a = self.length_squared();
                let length_squared_b = other.length_squared();
                if length_squared_a <= DEGENERATE || length_squared_b <= DEGENERATE {
                    return false;
                }

                let cos_min = 1.0 - tolerance;
                dot * dot >= length_squared_a * length_squared_b * (cos_min * cos_min)
            }

            fn is_normalized_approximately_equal(self, other: Self, tolerance: $scalar) -> bool {
                self.dot(other) >= 1.0 - tolerance
            }

            fn quantize(self, low: Self, high: Self, buckets: u32) -> TyVector3U32 {
                TyVector3U32::new(
                    self.x.quantize(low.x, high.x, buckets),
                    self.y.quantize(low.y, high.y, buckets),
                    self.z.quantize(low.z, high.z, buckets),
                )
            }

            fn catmull_rom_position(p0: Self, p1: Self, p2: Self, p3: Self, t: $scalar) -> Self {
                let t2 = t * t;
                let t3 = t2 * t;

                (p1 * 2.0
                    + (p2 - p0) * t
                    + (p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3) * t2
                    + (p1 * 3.0 - p0 - p2 * 3.0 + p3) * t3)
                    * 0.5
            }

            fn catmull_rom_tangent(p0: Self, p1: Self, p2: Self, p3: Self, t: $scalar) -> Self {
                let t2 = t * t;

                ((p2 - p0)
                    + (p0 * 4.0 - p1 * 10.0 + p2 * 8.0 - p3 * 2.0) * t
                    + (p1 * 9.0 - p0 * 3.0 - p2 * 9.0 + p3 * 3.0) * t2)
                    * 0.5
            }
        }
    };
}

impl_ty_vector3_ext!(DVec3, f64, DQuat);
impl_ty_vector3_ext!(Vec3, f32, Quat);

#[cfg(test)]
mod tests {
    use crate::{TyQuaternionF64, TyVector3Ext, TyVector3F64, TyVector3U32};
    use std::f64::consts::PI;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn close_vec(a: TyVector3F64, b: TyVector3F64) -> bool {
        close(a.x, b.x) && close(a.y, b.y) && close(a.z, b.z)
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
    fn from_axis_helpers_place_one_component() {
        assert_eq!(TyVector3F64::from_x(2.0), TyVector3F64::new(2.0, 0.0, 0.0));
        assert_eq!(TyVector3F64::from_y(2.0), TyVector3F64::new(0.0, 2.0, 0.0));
        assert_eq!(TyVector3F64::from_z(2.0), TyVector3F64::new(0.0, 0.0, 2.0));
        assert_eq!(TyVector3F64::splat(3.0).to_scale(), 3.0);
    }

    #[test]
    fn to_pure_quaternion_zeroes_w() {
        let quaternion = TyVector3F64::new(1.0, 2.0, 3.0).to_pure_quaternion();
        assert_eq!(quaternion, TyQuaternionF64::from_xyzw(1.0, 2.0, 3.0, 0.0));
    }

    #[test]
    fn rotation_around_normalizes_the_axis() {
        // A non-unit z axis and a quarter turn rotate x onto y.
        let quaternion = TyVector3F64::new(0.0, 0.0, 5.0).rotation_around(PI / 2.0);
        assert!(close_vec(quaternion * TyVector3F64::X, TyVector3F64::Y));
    }

    #[test]
    fn rotation_towards_gives_the_shortest_arc() {
        let quaternion = TyVector3F64::X.rotation_towards(TyVector3F64::Y);
        assert!(close_vec(quaternion * TyVector3F64::X, TyVector3F64::Y));
    }

    #[test]
    fn is_approximately_equal_gates_on_the_cosine() {
        let a = TyVector3F64::new(1.0, 0.0, 0.0);
        assert!(a.is_approximately_equal(TyVector3F64::new(2.0, 0.0, 0.0), 1e-6));
        assert!(!a.is_approximately_equal(TyVector3F64::new(0.0, 1.0, 0.0), 1e-6));
        assert!(!a.is_approximately_equal(TyVector3F64::ZERO, 1e-6));
        assert!(a.is_normalized_approximately_equal(TyVector3F64::X, 1e-9));
    }

    #[test]
    fn quantize_buckets_each_component() {
        // The midpoint of [0, 10] lands in the middle bucket; the high end clamps
        // into the last bucket.
        let bucketed = TyVector3F64::new(5.0, 0.0, 10.0).quantize(
            TyVector3F64::ZERO,
            TyVector3F64::splat(10.0),
            10,
        );
        assert_eq!(bucketed, TyVector3U32::new(5, 0, 9));
    }

    #[test]
    fn catmull_rom_passes_through_its_inner_control_points() {
        let p0 = TyVector3F64::new(0.0, 0.0, 0.0);
        let p1 = TyVector3F64::new(1.0, 0.0, 0.0);
        let p2 = TyVector3F64::new(2.0, 1.0, 0.0);
        let p3 = TyVector3F64::new(3.0, 0.0, 0.0);
        assert_eq!(TyVector3F64::catmull_rom_position(p0, p1, p2, p3, 0.0), p1);
        assert_eq!(TyVector3F64::catmull_rom_position(p0, p1, p2, p3, 1.0), p2);
    }
}
