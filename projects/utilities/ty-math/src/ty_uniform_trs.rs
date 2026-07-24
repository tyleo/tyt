use crate::{TyBounds, TyPose, TyQuaternionExt};
use glam::{DQuat, DVec3};

/// A transform with a single uniform scale factor, the scalar-scale companion to
/// [`TyTransform`](crate::TyTransform). Backed by glam; the bare name is the
/// `f64` form.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TyUniformTrs {
    /// The translation.
    pub translation: DVec3,

    /// The rotation, a unit quaternion.
    pub rotation: DQuat,

    /// The uniform scale factor.
    pub scale: f64,
}

impl TyUniformTrs {
    /// The identity transform: zero translation, identity rotation, unit scale.
    pub const IDENTITY: Self = Self {
        translation: DVec3::ZERO,
        rotation: DQuat::IDENTITY,
        scale: 1.0,
    };

    /// Creates a transform from a `translation`, `rotation`, and uniform `scale`.
    pub fn new(translation: DVec3, rotation: DQuat, scale: f64) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// The transform of `target` expressed in the local space of `self`. Assumes a
    /// positive scale.
    pub fn calculate_relative_trs(&self, target: &Self) -> Self {
        let inverse_rotation = self.rotation.inverse();
        let inverse_scale = 1.0 / self.scale;

        let delta = target.translation - self.translation;

        Self::new(
            inverse_rotation * (delta * inverse_scale),
            inverse_rotation * target.rotation,
            target.scale * inverse_scale,
        )
    }

    /// This transform as a [`TyPose`], dropping the scale.
    pub fn to_pose(&self) -> TyPose {
        TyPose::new(self.translation, self.rotation)
    }

    /// Transforms `aabb` by this transform, growing it to the axis-aligned bound of
    /// the scaled and rotated box. Assumes a positive scale.
    pub fn transform_aabb_conservative(&self, aabb: &TyBounds) -> TyBounds {
        let center = self.translation + self.rotation * (aabb.center * self.scale);
        let extents = self.rotation.rotate_extents_abs(aabb.extents * self.scale);

        TyBounds::new(center, extents)
    }
}

impl Default for TyUniformTrs {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use crate::{TyQuaternionExt, TyQuaternionF64, TyUniformTrsF64, TyVector3F64};

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn a_transform_relative_to_itself_is_the_identity() {
        let trs = TyUniformTrsF64::new(
            TyVector3F64::new(1.0, 2.0, 3.0),
            TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.0, 1.0, 0.0), 0.5),
            2.0,
        );
        let relative = trs.calculate_relative_trs(&trs);
        assert!(
            close(relative.translation.x, 0.0)
                && close(relative.translation.y, 0.0)
                && close(relative.translation.z, 0.0)
        );
        assert!(close(relative.scale, 1.0));
        assert!(
            relative
                .rotation
                .is_approximately_equal(TyQuaternionF64::IDENTITY, 1e-9)
        );
    }

    #[test]
    fn relative_scale_is_the_ratio() {
        let parent = TyUniformTrsF64::new(TyVector3F64::ZERO, TyQuaternionF64::IDENTITY, 2.0);
        let child = TyUniformTrsF64::new(TyVector3F64::ZERO, TyQuaternionF64::IDENTITY, 6.0);
        assert!(close(parent.calculate_relative_trs(&child).scale, 3.0));
    }
}
