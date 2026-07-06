use crate::{TyMatrix4x4, TyVector3, ty_array_conversions};
use std::ops::{Add, Mul, Neg, Sub};

/// A quaternion `(x, y, z, w)` with component type `T`.
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

ty_array_conversions!(TyQuaternion, 4, x, y, z, w);

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

            /// Returns the length of this quaternion.
            pub fn magnitude(&self) -> $t {
                self.magnitude_squared().sqrt()
            }

            /// Returns the squared length of this quaternion.
            pub fn magnitude_squared(&self) -> $t {
                self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
            }

            /// Returns this quaternion scaled to unit length.
            pub fn normalized(self) -> Self {
                self * (1.0 / self.magnitude())
            }

            /// True when this quaternion is within `tolerance` of unit length,
            /// testing the squared magnitude against `1` to avoid the square
            /// root.
            pub fn is_normalized(self, tolerance: $t) -> bool {
                (self.magnitude_squared() - 1.0).abs() <= tolerance
            }

            /// Returns the conjugate. For a unit quaternion this is the inverse
            /// rotation.
            pub fn conjugate(self) -> Self {
                Self::new(-self.x, -self.y, -self.z, self.w)
            }

            /// Returns the canonical form, with non-negative `w`.
            pub fn canonicalized(self) -> Self {
                if self.w >= 0.0 { self } else { -self }
            }

            /// Returns the inverse rotation, valid for a non-unit quaternion.
            pub fn inverse(self) -> Self {
                let inverse_squared = 1.0 / self.magnitude_squared();
                Self::new(
                    -self.x * inverse_squared,
                    -self.y * inverse_squared,
                    -self.z * inverse_squared,
                    self.w * inverse_squared,
                )
            }

            /// Returns the vector part.
            pub fn vector_part(self) -> TyVector3<$t> {
                TyVector3::new(self.x, self.y, self.z)
            }

            /// Returns the dot product of `self` and `other`.
            pub fn dot(self, other: Self) -> $t {
                self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
            }

            /// Builds a rotation from orthonormal `right`, `up`, and `forward`
            /// basis vectors.
            pub fn from_basis_vectors(
                right: TyVector3<$t>,
                up: TyVector3<$t>,
                forward: TyVector3<$t>,
            ) -> Self {
                let trace = right.x + up.y + forward.z;
                if trace > 0.0 {
                    let s = (trace + 1.0).sqrt() * 2.0;
                    Self::new(
                        (up.z - forward.y) / s,
                        (forward.x - right.z) / s,
                        (right.y - up.x) / s,
                        0.25 * s,
                    )
                } else if right.x > up.y && right.x > forward.z {
                    let s = (1.0 + right.x - up.y - forward.z).sqrt() * 2.0;
                    Self::new(
                        0.25 * s,
                        (right.y + up.x) / s,
                        (forward.x + right.z) / s,
                        (up.z - forward.y) / s,
                    )
                } else if up.y > forward.z {
                    let s = (1.0 + up.y - right.x - forward.z).sqrt() * 2.0;
                    Self::new(
                        (right.y + up.x) / s,
                        0.25 * s,
                        (up.z + forward.y) / s,
                        (forward.x - right.z) / s,
                    )
                } else {
                    let s = (1.0 + forward.z - right.x - up.y).sqrt() * 2.0;
                    Self::new(
                        (forward.x + right.z) / s,
                        (up.z + forward.y) / s,
                        0.25 * s,
                        (right.y - up.x) / s,
                    )
                }
            }

            /// A rotation from the upper-left 3x3 of `matrix`, taking its first
            /// three columns as the right, up, and forward basis vectors and
            /// normalizing each to strip any embedded scale. Returns `None` for a
            /// degenerate matrix, one with a near-zero column that cannot form a
            /// basis. Inverts
            /// [`TyMatrix4x4::from_quaternion`](crate::TyMatrix4x4::from_quaternion).
            pub fn from_rotation_matrix(matrix: TyMatrix4x4<$t>) -> Option<Self> {
                const DEGENERATE: $t = 1e-12;

                let right = matrix.columns[0].truncate();
                let up = matrix.columns[1].truncate();
                let forward = matrix.columns[2].truncate();

                if right.magnitude() < DEGENERATE
                    || up.magnitude() < DEGENERATE
                    || forward.magnitude() < DEGENERATE
                {
                    return None;
                }

                Some(Self::from_basis_vectors(
                    right.normalized(),
                    up.normalized(),
                    forward.normalized(),
                ))
            }

            /// Builds a rotation from `right` and `forward` basis vectors,
            /// deriving `up` in a left-handed frame.
            pub fn from_right_forward(right: TyVector3<$t>, forward: TyVector3<$t>) -> Self {
                let up = forward.cross(&right).normalized();
                Self::from_basis_vectors(right, up, forward)
            }

            /// Builds a rotation from `right` and `up` basis vectors, deriving
            /// `forward` in a left-handed frame.
            pub fn from_right_up(right: TyVector3<$t>, up: TyVector3<$t>) -> Self {
                let forward = right.cross(&up).normalized();
                Self::from_basis_vectors(right, up, forward)
            }

            /// Returns this rotation with a further rotation of `angle` radians
            /// about `axis` applied after it.
            pub fn rotate_around_axis(self, axis: TyVector3<$t>, angle: $t) -> Self {
                Self::from_axis_angle(axis, angle) * self
            }

            /// True when `self` and `other` are approximately the same rotation,
            /// within `tolerance` on the absolute dot product. Both are taken to
            /// be unit length; the sign is ignored since `q` and `-q` are one
            /// rotation.
            pub fn is_approximately_equal(self, other: Self, tolerance: $t) -> bool {
                self.dot(other).abs() >= 1.0 - tolerance
            }

            /// Spherically interpolates from `self` towards `to` by `t` in
            /// `[0, 1]`, taking the shortest arc. Falls back to a normalized
            /// linear blend when the rotations are nearly aligned.
            pub fn slerp_towards(self, to: Self, t: $t) -> Self {
                const DOT_THRESHOLD: $t = 0.9995;

                let mut to = to;
                let mut dot = self.dot(to);

                // q and -q are the same rotation; flip to take the shortest arc.
                if dot < 0.0 {
                    to = -to;
                    dot = -dot;
                }

                if dot > DOT_THRESHOLD {
                    return (self * (1.0 - t) + to * t).normalized();
                }

                let dot = dot.clamp(-1.0, 1.0);
                let theta_0 = dot.acos();
                let sin_theta_0 = theta_0.sin();

                // Guard the division for a degenerate angle between the rotations.
                if sin_theta_0 < 1.0e-5 {
                    return (self * (1.0 - t) + to * t).normalized();
                }

                let theta = theta_0 * t;
                let sin_theta = theta.sin();

                let s0 = theta.cos() - dot * sin_theta / sin_theta_0;
                let s1 = sin_theta / sin_theta_0;

                self * s0 + to * s1
            }

            /// Returns the rotation matrix of this unit quaternion.
            pub fn to_matrix4x4(self) -> TyMatrix4x4<$t> {
                TyMatrix4x4::<$t>::from_quaternion(self)
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

        impl Neg for TyQuaternion<$t> {
            type Output = Self;

            fn neg(self) -> Self {
                Self::new(-self.x, -self.y, -self.z, -self.w)
            }
        }

        impl Add for TyQuaternion<$t> {
            type Output = Self;

            fn add(self, rhs: Self) -> Self {
                Self::new(
                    self.x + rhs.x,
                    self.y + rhs.y,
                    self.z + rhs.z,
                    self.w + rhs.w,
                )
            }
        }

        impl Sub for TyQuaternion<$t> {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self {
                Self::new(
                    self.x - rhs.x,
                    self.y - rhs.y,
                    self.z - rhs.z,
                    self.w - rhs.w,
                )
            }
        }

        impl Mul<$t> for TyQuaternion<$t> {
            type Output = Self;

            fn mul(self, rhs: $t) -> Self {
                Self::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
            }
        }

        impl Mul<TyQuaternion<$t>> for $t {
            type Output = TyQuaternion<$t>;

            fn mul(self, rhs: TyQuaternion<$t>) -> TyQuaternion<$t> {
                rhs * self
            }
        }
    };
}

impl_ty_quaternion_float!(f32);
impl_ty_quaternion_float!(f64);

#[cfg(test)]
mod tests {
    use crate::{TyMatrix4x4F64, TyQuaternionF64, TyVector3F64};
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

    #[test]
    fn normalized_has_unit_length() {
        let quaternion = TyQuaternionF64::new(1.0, 2.0, 3.0, 4.0).normalized();
        assert!(close(quaternion.magnitude(), 1.0));
    }

    #[test]
    fn is_normalized_gates_on_squared_length() {
        assert!(about_z(0.7).is_normalized(1e-9));
        // magnitude_squared = 1 + 4 + 9 + 16 = 30, so the deviation is 29.
        let scaled = TyQuaternionF64::new(1.0, 2.0, 3.0, 4.0);
        assert!(!scaled.is_normalized(1e-6));
        assert!(scaled.is_normalized(29.0));
    }

    #[test]
    fn inverse_undoes_a_rotation() {
        // A rotation composed with its inverse is the identity, so it leaves a
        // vector unchanged.
        let quaternion = about_z(0.7);
        let composed = quaternion * quaternion.inverse();
        let point = TyVector3F64::new(1.0, 2.0, 3.0);
        let rotated = composed.rotate(point);
        assert!(close(rotated.x, 1.0) && close(rotated.y, 2.0) && close(rotated.z, 3.0));
    }

    #[test]
    fn conjugate_is_the_inverse_of_a_unit_quaternion() {
        let quaternion = about_z(0.7);
        let conjugate = quaternion.conjugate();
        let inverse = quaternion.inverse();
        assert!(
            close(conjugate.x, inverse.x)
                && close(conjugate.y, inverse.y)
                && close(conjugate.z, inverse.z)
                && close(conjugate.w, inverse.w)
        );
    }

    #[test]
    fn canonicalized_makes_w_non_negative() {
        let flipped = -about_z(0.7);
        assert!(flipped.w < 0.0);
        assert!(flipped.canonicalized().w >= 0.0);
    }

    #[test]
    fn slerp_hits_its_endpoints() {
        let from = about_z(0.2);
        let to = about_z(1.3);
        let start = from.slerp_towards(to, 0.0);
        let end = from.slerp_towards(to, 1.0);
        assert!(start.is_approximately_equal(from, 1e-9));
        assert!(end.is_approximately_equal(to, 1e-9));
    }

    #[test]
    fn slerp_midpoint_is_the_halfway_rotation() {
        // Halfway from 0 to a half turn about z is a quarter turn about z.
        let mid = about_z(0.0).slerp_towards(about_z(PI), 0.5);
        assert!(mid.is_approximately_equal(about_z(PI / 2.0), 1e-9));
    }

    #[test]
    fn from_basis_vectors_of_the_identity_frame_is_the_identity() {
        let quaternion = TyQuaternionF64::from_basis_vectors(
            TyVector3F64::new(1.0, 0.0, 0.0),
            TyVector3F64::new(0.0, 1.0, 0.0),
            TyVector3F64::new(0.0, 0.0, 1.0),
        );
        assert!(quaternion.is_approximately_equal(TyQuaternionF64::identity(), 1e-9));
    }

    #[test]
    fn from_rotation_matrix_inverts_from_quaternion() {
        let quaternion = TyQuaternionF64::from_axis_angle(TyVector3F64::new(1.0, 2.0, 3.0), 0.9);
        let matrix = TyMatrix4x4F64::from_quaternion(quaternion);
        let recovered = TyQuaternionF64::from_rotation_matrix(matrix).expect("a proper rotation");
        assert!(recovered.is_approximately_equal(quaternion, 1e-9));
    }

    #[test]
    fn from_rotation_matrix_of_a_degenerate_matrix_is_none() {
        let zero = TyMatrix4x4F64::from_column_arrays([[0.0; 4]; 4]);
        assert_eq!(TyQuaternionF64::from_rotation_matrix(zero), None);
    }

    #[test]
    fn from_basis_vectors_recovers_a_rotations_own_axes() {
        // Take a rotation, read its rotated axes off its matrix columns, and
        // rebuild it from those basis vectors.
        let quaternion = TyQuaternionF64::from_axis_angle(TyVector3F64::new(1.0, 2.0, 3.0), 0.9);
        let right = quaternion.rotate(TyVector3F64::new(1.0, 0.0, 0.0));
        let up = quaternion.rotate(TyVector3F64::new(0.0, 1.0, 0.0));
        let forward = quaternion.rotate(TyVector3F64::new(0.0, 0.0, 1.0));
        let rebuilt = TyQuaternionF64::from_basis_vectors(right, up, forward);
        assert!(rebuilt.is_approximately_equal(quaternion, 1e-9));
    }
}
