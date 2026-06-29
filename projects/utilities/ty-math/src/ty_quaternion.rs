use crate::TyVector3;

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
        }

        impl Default for TyQuaternion<$t> {
            fn default() -> Self {
                Self::identity()
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
}
