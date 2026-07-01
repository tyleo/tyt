use crate::TyVector3;
use std::ops::Mul;

/// A quaternion `(x, y, z, w)` generic over its component type `T`.
///
/// See `TyQuaternionF32` and `TyQuaternionF64` for the common instantiations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TyQuaternion<T> {
    /// The `x` component.
    pub x: T,

    /// The `y` component.
    pub y: T,

    /// The `z` component.
    pub z: T,

    /// The `w` (scalar) component.
    pub w: T,
}

impl<T> TyQuaternion<T> {
    /// Creates a new quaternion from `x`, `y`, `z`, and `w` components.
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { x, y, z, w }
    }
}

/// Implements the float-only quaternion operations for a concrete
/// floating-point component type.
macro_rules! impl_ty_quaternion_float {
    ($t:ty) => {
        impl TyQuaternion<$t> {
            /// Returns the identity quaternion `(0, 0, 0, 1)`.
            pub fn identity() -> Self {
                Self {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                }
            }

            /// Builds a unit quaternion that rotates by `angle` radians about
            /// `axis`. A degenerate axis or a zero angle yields the identity.
            pub fn from_axis_angle(axis: TyVector3<$t>, angle: $t) -> Self {
                let length = axis.magnitude();
                if length < 1e-12 || angle == 0.0 {
                    return Self::identity();
                }
                let half = angle / 2.0;
                let scale = half.sin() / length;
                Self {
                    x: axis.x * scale,
                    y: axis.y * scale,
                    z: axis.z * scale,
                    w: half.cos(),
                }
            }

            /// Decomposes this unit quaternion into a rotation `axis` of unit
            /// length and an `angle` in radians, the inverse of
            /// [`from_axis_angle`](Self::from_axis_angle). A near-zero rotation has
            /// no defined axis, so it yields the `x` axis with a zero angle.
            pub fn to_axis_angle(self) -> (TyVector3<$t>, $t) {
                let axis = TyVector3::new(self.x, self.y, self.z);
                let length = axis.magnitude();
                if length < 1e-12 {
                    return (TyVector3::new(1.0, 0.0, 0.0), 0.0);
                }
                let angle = 2.0 * length.atan2(self.w);
                (axis * (1.0 / length), angle)
            }

            /// Rotates `v` by this quaternion, taken to be unit length.
            pub fn rotate(self, v: TyVector3<$t>) -> TyVector3<$t> {
                let axis = TyVector3::new(self.x, self.y, self.z);
                let t = axis.cross(&v) * 2.0;
                v + t * self.w + axis.cross(&t)
            }

            /// Decomposes this unit quaternion into Tait-Bryan euler angles in
            /// radians, returned as `(x, y, z)`. The rotation is
            /// `Rz(z) * Ry(y) * Rx(x)`, so a rotation about a single axis reads
            /// on that component alone. The `y` (pitch) term is clamped into
            /// `[-1, 1]` before the arcsine, so a near-gimbal value stays finite.
            pub fn to_euler_radians(self) -> TyVector3<$t> {
                let (x, y, z, w) = (self.x, self.y, self.z, self.w);
                let roll = (2.0 * (w * x + y * z)).atan2(1.0 - 2.0 * (x * x + y * y));
                let pitch = (2.0 * (w * y - z * x)).clamp(-1.0, 1.0).asin();
                let yaw = (2.0 * (w * z + x * y)).atan2(1.0 - 2.0 * (y * y + z * z));
                TyVector3::new(roll, pitch, yaw)
            }

            /// Rotates `extents` by this quaternion using the absolute values of
            /// the rotation matrix, so the result stays a positive extents
            /// vector. This is the half-extents of the rotated box's
            /// axis-aligned bound.
            pub fn rotate_extents_abs(self, extents: TyVector3<$t>) -> TyVector3<$t> {
                let (x, y, z, w) = (self.x, self.y, self.z, self.w);
                // Rows of the rotation matrix applied as `R * v`.
                let r00 = 1.0 - 2.0 * (y * y + z * z);
                let r01 = 2.0 * (x * y - w * z);
                let r02 = 2.0 * (x * z + w * y);
                let r10 = 2.0 * (x * y + w * z);
                let r11 = 1.0 - 2.0 * (x * x + z * z);
                let r12 = 2.0 * (y * z - w * x);
                let r20 = 2.0 * (x * z - w * y);
                let r21 = 2.0 * (y * z + w * x);
                let r22 = 1.0 - 2.0 * (x * x + y * y);
                TyVector3::new(
                    r00.abs() * extents.x + r01.abs() * extents.y + r02.abs() * extents.z,
                    r10.abs() * extents.x + r11.abs() * extents.y + r12.abs() * extents.z,
                    r20.abs() * extents.x + r21.abs() * extents.y + r22.abs() * extents.z,
                )
            }
        }

        impl Default for TyQuaternion<$t> {
            fn default() -> Self {
                Self::identity()
            }
        }

        impl Mul for TyQuaternion<$t> {
            type Output = Self;

            /// The Hamilton product. Composing `self * other` yields the rotation
            /// that applies `other` first, then `self`.
            fn mul(self, other: Self) -> Self {
                Self {
                    x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
                    y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
                    z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
                    w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
                }
            }
        }
    };
}

impl_ty_quaternion_float!(f32);
impl_ty_quaternion_float!(f64);

#[cfg(test)]
mod tests {
    use crate::{TyQuaternionF64, TyVector3F64};
    use std::f64::consts::{FRAC_1_SQRT_2, PI};

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn from_axis_angle_is_identity_for_a_degenerate_axis_or_zero_angle() {
        assert_eq!(
            TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.0, 0.0, 0.0), 1.0),
            TyQuaternionF64::identity()
        );
        assert_eq!(
            TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.0, 0.0, 1.0), 0.0),
            TyQuaternionF64::identity()
        );
    }

    #[test]
    fn from_axis_angle_normalizes_the_axis() {
        // A non-unit axis and a quarter turn give the unit quaternion for that turn.
        let quaternion =
            TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.0, 0.0, 5.0), PI / 2.0);
        assert!(close(quaternion.x, 0.0) && close(quaternion.y, 0.0));
        assert!(close(quaternion.z, FRAC_1_SQRT_2) && close(quaternion.w, FRAC_1_SQRT_2));
    }

    #[test]
    fn to_axis_angle_inverts_from_axis_angle() {
        let axis = TyVector3F64::new(1.0, 2.0, 3.0);
        let quaternion = TyQuaternionF64::from_axis_angle(axis, 1.2);
        let (recovered, angle) = quaternion.to_axis_angle();
        // The axis returns unit length along the original direction, the angle exact.
        let unit = axis * (1.0 / axis.magnitude());
        assert!(close(recovered.x, unit.x) && close(recovered.y, unit.y));
        assert!(close(recovered.z, unit.z) && close(angle, 1.2));
        // Re-encoding reproduces the same quaternion.
        let again = TyQuaternionF64::from_axis_angle(recovered, angle);
        assert!(close(again.x, quaternion.x) && close(again.y, quaternion.y));
        assert!(close(again.z, quaternion.z) && close(again.w, quaternion.w));
    }

    #[test]
    fn to_axis_angle_of_the_identity_is_a_zero_angle() {
        let (_, angle) = TyQuaternionF64::identity().to_axis_angle();
        assert!(close(angle, 0.0));
    }

    #[test]
    fn rotate_turns_x_into_y_about_z() {
        let quaternion =
            TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.0, 0.0, 1.0), PI / 2.0);
        let rotated = quaternion.rotate(TyVector3F64::new(1.0, 0.0, 0.0));
        assert!(close(rotated.x, 0.0) && close(rotated.y, 1.0) && close(rotated.z, 0.0));
    }

    #[test]
    fn rotate_by_the_identity_leaves_a_vector_unchanged() {
        let rotated = TyQuaternionF64::identity().rotate(TyVector3F64::new(1.0, 2.0, 3.0));
        assert!(close(rotated.x, 1.0) && close(rotated.y, 2.0) && close(rotated.z, 3.0));
    }

    /// A unit quaternion for `angle` radians about the `z` axis.
    fn about_z(angle: f64) -> TyQuaternionF64 {
        TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.0, 0.0, 1.0), angle)
    }

    #[test]
    fn mul_by_the_identity_is_neutral() {
        let q = about_z(0.7);
        let identity = TyQuaternionF64::identity();
        let left = identity * q;
        let right = q * identity;
        assert!(
            close(left.x, q.x) && close(left.y, q.y) && close(left.z, q.z) && close(left.w, q.w)
        );
        assert!(
            close(right.x, q.x)
                && close(right.y, q.y)
                && close(right.z, q.z)
                && close(right.w, q.w)
        );
    }

    #[test]
    fn mul_composes_two_rotations() {
        // Two quarter turns about z make a half turn: x maps to -x.
        let half = about_z(PI / 2.0) * about_z(PI / 2.0);
        let rotated = half.rotate(TyVector3F64::new(1.0, 0.0, 0.0));
        assert!(close(rotated.x, -1.0) && close(rotated.y, 0.0) && close(rotated.z, 0.0));
    }

    #[test]
    fn to_euler_reads_a_single_axis_on_its_own_component() {
        let x = TyQuaternionF64::from_axis_angle(TyVector3F64::new(1.0, 0.0, 0.0), PI / 2.0)
            .to_euler_radians();
        assert!(close(x.x, PI / 2.0) && close(x.y, 0.0) && close(x.z, 0.0));

        let y = TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.0, 1.0, 0.0), PI / 2.0)
            .to_euler_radians();
        assert!(close(y.x, 0.0) && close(y.y, PI / 2.0) && close(y.z, 0.0));

        let z = about_z(PI / 2.0).to_euler_radians();
        assert!(close(z.x, 0.0) && close(z.y, 0.0) && close(z.z, PI / 2.0));

        let identity = TyQuaternionF64::identity().to_euler_radians();
        assert!(close(identity.x, 0.0) && close(identity.y, 0.0) && close(identity.z, 0.0));
    }

    #[test]
    fn rotate_extents_abs_swaps_axes_on_a_quarter_turn() {
        // A quarter turn about z swaps the x and y extents, leaving z.
        let extents = about_z(PI / 2.0).rotate_extents_abs(TyVector3F64::new(2.0, 3.0, 4.0));
        assert!(close(extents.x, 3.0) && close(extents.y, 2.0) && close(extents.z, 4.0));

        // The identity leaves extents unchanged.
        let same = TyQuaternionF64::identity().rotate_extents_abs(TyVector3F64::new(2.0, 3.0, 4.0));
        assert!(close(same.x, 2.0) && close(same.y, 3.0) && close(same.z, 4.0));
    }
}
