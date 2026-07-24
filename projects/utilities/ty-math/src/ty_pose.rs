use crate::{TyBounds, TyQuaternionExt, TyUniformTrs};
use glam::{DQuat, DVec3};

/// A rigid pose: a rotation and position, no scale. Backed by glam; the bare name
/// is the `f64` form.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TyPose {
    /// The position.
    pub position: DVec3,

    /// The rotation, a unit quaternion.
    pub rotation: DQuat,
}

impl TyPose {
    /// The identity pose: zero position, identity rotation.
    pub const IDENTITY: Self = Self {
        position: DVec3::ZERO,
        rotation: DQuat::IDENTITY,
    };

    /// Creates a pose from a `position` and a `rotation`.
    pub fn new(position: DVec3, rotation: DQuat) -> Self {
        Self { position, rotation }
    }

    /// The pose of `target` expressed in the local space of `self`.
    pub fn calculate_relative_pose(&self, target: &Self) -> Self {
        let inverse_rotation = self.rotation.inverse();

        Self::new(
            inverse_rotation * (target.position - self.position),
            inverse_rotation * target.rotation,
        )
    }

    /// Transforms `aabb` by this pose, growing it to the axis-aligned bound of the
    /// rotated box.
    pub fn transform_aabb_conservative(&self, aabb: &TyBounds) -> TyBounds {
        let center = self.position + self.rotation * aabb.center;
        let extents = self.rotation.rotate_extents_abs(aabb.extents);

        TyBounds::new(center, extents)
    }

    /// This pose with a uniform `scale`, as a [`TyUniformTrs`].
    pub fn with_uniform_scale(&self, scale: f64) -> TyUniformTrs {
        TyUniformTrs::new(self.position, self.rotation, scale)
    }
}

impl Default for TyPose {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use crate::{TyPoseF64, TyQuaternionExt, TyQuaternionF64, TyVector3F64};
    use std::f64::consts::PI;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn a_pose_relative_to_itself_is_the_identity() {
        let pose = TyPoseF64::new(
            TyVector3F64::new(1.0, 2.0, 3.0),
            TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.0, 0.0, 1.0), 0.6),
        );
        let relative = pose.calculate_relative_pose(&pose);
        assert!(
            close(relative.position.x, 0.0)
                && close(relative.position.y, 0.0)
                && close(relative.position.z, 0.0)
        );
        assert!(
            relative
                .rotation
                .is_approximately_equal(TyQuaternionF64::IDENTITY, 1e-9)
        );
    }

    #[test]
    fn relative_pose_undoes_the_parent_frame() {
        // A parent rotated a quarter turn about z at +x; a target one unit along
        // world +x sits, in the parent's frame, one unit along the parent's local
        // -y (world +x is the parent's local -y after a +90 z turn).
        let parent = TyPoseF64::new(
            TyVector3F64::new(1.0, 0.0, 0.0),
            TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.0, 0.0, 1.0), PI / 2.0),
        );
        let target = TyPoseF64::new(TyVector3F64::new(2.0, 0.0, 0.0), TyQuaternionF64::IDENTITY);
        let relative = parent.calculate_relative_pose(&target);
        assert!(
            close(relative.position.x, 0.0)
                && close(relative.position.y, -1.0)
                && close(relative.position.z, 0.0)
        );
    }
}
