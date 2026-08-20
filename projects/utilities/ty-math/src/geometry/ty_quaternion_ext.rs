use crate::{TyMatrix4x4F64, TyQuaternionF64, TyVector3F64};
use glam::{DMat3, EulerRot};

/// The tyt-specific quaternion operations glam does not provide directly, carried
/// on [`TyQuaternionF64`] so callers keep `q.foo()` with `use ty_math::TyQuaternionExt`.
pub trait TyQuaternionExt {
    /// Tait-Bryan euler angles in radians as `(roll, pitch, yaw)` about the
    /// `(x, y, z)` axes, the rotation `Rz(z) * Ry(y) * Rx(x)`. A rotation about a
    /// single axis reads on that component alone.
    fn to_euler_radians(self) -> TyVector3F64;

    /// A rotation from the upper-left 3x3 of `matrix`, taking its first three
    /// columns as the right, up, and forward basis vectors and normalizing each to
    /// strip embedded scale. `None` for a degenerate matrix with a near-zero
    /// column that cannot form a basis.
    fn from_rotation_matrix(matrix: TyMatrix4x4F64) -> Option<TyQuaternionF64>;

    /// A rotation from orthonormal `right`, `up`, and `forward` basis vectors.
    fn from_basis_vectors(
        right: TyVector3F64,
        up: TyVector3F64,
        forward: TyVector3F64,
    ) -> TyQuaternionF64;

    /// A rotation from `right` and `forward`, deriving `up` in a left-handed frame.
    fn from_right_forward(right: TyVector3F64, forward: TyVector3F64) -> TyQuaternionF64;

    /// A rotation from `right` and `up`, deriving `forward` in a left-handed frame.
    fn from_right_up(right: TyVector3F64, up: TyVector3F64) -> TyQuaternionF64;

    /// This rotation with a further rotation of `angle` radians about `axis`, which
    /// is normalized first, applied after it.
    fn rotate_around_axis(self, axis: TyVector3F64, angle: f64) -> TyQuaternionF64;

    /// `extents` rotated by this quaternion using the absolute values of the
    /// rotation matrix, so the result stays a positive extents vector: the
    /// half-extents of the rotated box's axis-aligned bound.
    fn rotate_extents_abs(self, extents: TyVector3F64) -> TyVector3F64;

    /// True when `self` and `other` are approximately the same rotation, within
    /// `tolerance` on the absolute dot product. Both are taken to be unit length;
    /// the sign is ignored since `q` and `-q` are one rotation.
    // `self` by value matches glam's Copy convention; the trait cannot express
    // `Self: Copy` for the lint to see it.
    #[allow(clippy::wrong_self_convention)]
    fn is_approximately_equal(self, other: TyQuaternionF64, tolerance: f64) -> bool;

    /// True when the squared length is within `tolerance` of `1`. A custom
    /// tolerance where glam's `is_normalized` fixes its own.
    #[allow(clippy::wrong_self_convention)]
    fn is_normalized_within(self, tolerance: f64) -> bool;

    /// The canonical form, with non-negative `w`.
    fn canonicalized(self) -> TyQuaternionF64;
}

impl TyQuaternionExt for TyQuaternionF64 {
    fn to_euler_radians(self) -> TyVector3F64 {
        let (x, y, z) = self.to_euler(EulerRot::XYZEx);
        TyVector3F64::new(x, y, z)
    }

    fn from_rotation_matrix(matrix: TyMatrix4x4F64) -> Option<TyQuaternionF64> {
        const DEGENERATE: f64 = 1e-12;

        let right = matrix.x_axis.truncate();
        let up = matrix.y_axis.truncate();
        let forward = matrix.z_axis.truncate();

        if right.length() < DEGENERATE || up.length() < DEGENERATE || forward.length() < DEGENERATE
        {
            return None;
        }

        Some(TyQuaternionF64::from_rotation_axes(
            right.normalize(),
            up.normalize(),
            forward.normalize(),
        ))
    }

    fn from_basis_vectors(
        right: TyVector3F64,
        up: TyVector3F64,
        forward: TyVector3F64,
    ) -> TyQuaternionF64 {
        TyQuaternionF64::from_rotation_axes(right, up, forward)
    }

    fn from_right_forward(right: TyVector3F64, forward: TyVector3F64) -> TyQuaternionF64 {
        let up = forward.cross(right).normalize();
        TyQuaternionF64::from_rotation_axes(right, up, forward)
    }

    fn from_right_up(right: TyVector3F64, up: TyVector3F64) -> TyQuaternionF64 {
        let forward = right.cross(up).normalize();
        TyQuaternionF64::from_rotation_axes(right, up, forward)
    }

    fn rotate_around_axis(self, axis: TyVector3F64, angle: f64) -> TyQuaternionF64 {
        TyQuaternionF64::from_axis_angle(axis.normalize(), angle) * self
    }

    fn rotate_extents_abs(self, extents: TyVector3F64) -> TyVector3F64 {
        let m = DMat3::from_quat(self);
        DMat3::from_cols(m.x_axis.abs(), m.y_axis.abs(), m.z_axis.abs()) * extents
    }

    fn is_approximately_equal(self, other: TyQuaternionF64, tolerance: f64) -> bool {
        self.dot(other).abs() >= 1.0 - tolerance
    }

    fn is_normalized_within(self, tolerance: f64) -> bool {
        (self.length_squared() - 1.0).abs() <= tolerance
    }

    fn canonicalized(self) -> TyQuaternionF64 {
        if self.w >= 0.0 { self } else { -self }
    }
}

#[cfg(test)]
mod tests {
    use crate::{TyMatrix4x4F64, TyQuaternionExt, TyQuaternionF64, TyVector3F64};
    use std::f64::consts::PI;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn close_vec(a: TyVector3F64, b: TyVector3F64) -> bool {
        close(a.x, b.x) && close(a.y, b.y) && close(a.z, b.z)
    }

    /// The hand-rolled Tait-Bryan formula this crate used before glam, kept as the
    /// convention oracle so the wire that reads these angles stays stable.
    fn euler_reference(q: TyQuaternionF64) -> TyVector3F64 {
        let (x, y, z, w) = (q.x, q.y, q.z, q.w);
        let roll = (2.0 * (w * x + y * z)).atan2(1.0 - 2.0 * (x * x + y * y));
        let pitch = (2.0 * (w * y - z * x)).clamp(-1.0, 1.0).asin();
        let yaw = (2.0 * (w * z + x * y)).atan2(1.0 - 2.0 * (y * y + z * z));
        TyVector3F64::new(roll, pitch, yaw)
    }

    #[test]
    fn to_euler_reads_a_single_axis_on_its_own_component() {
        let x = TyQuaternionF64::from_axis_angle(TyVector3F64::X, PI / 2.0).to_euler_radians();
        assert!(close(x.x, PI / 2.0) && close(x.y, 0.0) && close(x.z, 0.0));

        let y = TyQuaternionF64::from_axis_angle(TyVector3F64::Y, PI / 2.0).to_euler_radians();
        assert!(close(y.x, 0.0) && close(y.y, PI / 2.0) && close(y.z, 0.0));

        let z = TyQuaternionF64::from_axis_angle(TyVector3F64::Z, PI / 2.0).to_euler_radians();
        assert!(close(z.x, 0.0) && close(z.y, 0.0) && close(z.z, PI / 2.0));

        let identity = TyQuaternionF64::IDENTITY.to_euler_radians();
        assert!(close(identity.x, 0.0) && close(identity.y, 0.0) && close(identity.z, 0.0));
    }

    #[test]
    fn to_euler_radians_matches_the_tait_bryan_reference() {
        // Arbitrary rotations well away from gimbal lock: glam's XYZEx must match
        // the old formula so the vmax euler wire does not move.
        for (axis, angle) in [
            (TyVector3F64::new(1.0, 2.0, 3.0), 0.9),
            (TyVector3F64::new(-2.0, 1.0, 0.5), 1.7),
            (TyVector3F64::new(0.3, -0.4, 0.2), 2.4),
            (TyVector3F64::new(1.0, 0.0, 1.0), 0.5),
        ] {
            let q = TyQuaternionF64::from_axis_angle(axis.normalize(), angle);
            let got = q.to_euler_radians();
            let want = euler_reference(q);
            assert!(
                close_vec(got, want),
                "euler mismatch: got {got:?} want {want:?}"
            );
        }
    }

    #[test]
    fn from_rotation_matrix_inverts_from_quat() {
        let q = TyQuaternionF64::from_axis_angle(TyVector3F64::new(1.0, 2.0, 3.0).normalize(), 0.9);
        let matrix = TyMatrix4x4F64::from_quat(q);
        let recovered = TyQuaternionF64::from_rotation_matrix(matrix).expect("a proper rotation");
        // q and -q are the same rotation.
        assert!(close(recovered.dot(q).abs(), 1.0));
    }

    #[test]
    fn from_rotation_matrix_of_a_degenerate_matrix_is_none() {
        let zero = TyMatrix4x4F64::from_cols_array_2d(&[[0.0; 4]; 4]);
        assert_eq!(TyQuaternionF64::from_rotation_matrix(zero), None);
    }

    #[test]
    fn from_basis_vectors_recovers_a_rotations_own_axes() {
        let q = TyQuaternionF64::from_axis_angle(TyVector3F64::new(1.0, 2.0, 3.0).normalize(), 0.9);
        let rebuilt = TyQuaternionF64::from_basis_vectors(
            q * TyVector3F64::X,
            q * TyVector3F64::Y,
            q * TyVector3F64::Z,
        );
        assert!(close(rebuilt.dot(q).abs(), 1.0));
    }

    #[test]
    fn rotate_around_axis_composes_after_the_rotation() {
        // The identity rotated a quarter turn about z carries x onto y.
        let q = TyQuaternionF64::IDENTITY
            .rotate_around_axis(TyVector3F64::new(0.0, 0.0, 2.0), PI / 2.0);
        assert!(close_vec(q * TyVector3F64::X, TyVector3F64::Y));
    }

    #[test]
    fn rotate_extents_abs_swaps_axes_on_a_quarter_turn() {
        let q = TyQuaternionF64::from_axis_angle(TyVector3F64::Z, PI / 2.0);
        let extents = q.rotate_extents_abs(TyVector3F64::new(2.0, 3.0, 4.0));
        assert!(close(extents.x, 3.0) && close(extents.y, 2.0) && close(extents.z, 4.0));

        let same = TyQuaternionF64::IDENTITY.rotate_extents_abs(TyVector3F64::new(2.0, 3.0, 4.0));
        assert!(close(same.x, 2.0) && close(same.y, 3.0) && close(same.z, 4.0));
    }

    #[test]
    fn is_approximately_equal_ignores_sign_and_canonicalized_makes_w_non_negative() {
        let q = TyQuaternionF64::from_axis_angle(TyVector3F64::Z, 0.7);
        assert!(q.is_approximately_equal(-q, 1e-9));
        assert!((-q).w < 0.0);
        assert!((-q).canonicalized().w >= 0.0);
    }

    #[test]
    fn is_normalized_within_gates_on_squared_length() {
        let unit = TyQuaternionF64::from_axis_angle(TyVector3F64::Z, 0.7);
        assert!(unit.is_normalized_within(1e-9));
        // length_squared = 1 + 4 + 9 + 16 = 30, so the deviation is 29.
        let scaled = TyQuaternionF64::from_xyzw(1.0, 2.0, 3.0, 4.0);
        assert!(!scaled.is_normalized_within(1e-6));
        assert!(scaled.is_normalized_within(29.0));
    }
}
