use crate::{TyPoseF64, TyUniformTrsF64};
use glam::{DQuat, DVec3};

/// A node transform with `f64` components, composing as
/// `Translation * Rotation * Scale`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TyTransformF64 {
    /// The translation.
    pub position: DVec3,

    /// The rotation, a unit quaternion.
    pub rotation: DQuat,

    /// The per-axis scale.
    pub scale: DVec3,
}

impl TyTransformF64 {
    /// The identity transform: zero position, identity rotation, unit scale.
    pub const IDENTITY: Self = Self {
        position: DVec3::ZERO,
        rotation: DQuat::IDENTITY,
        scale: DVec3::ONE,
    };

    /// Creates a new transform from a `position`, `rotation`, and `scale`.
    pub fn new(position: DVec3, rotation: DQuat, scale: DVec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// A translation-only transform: `position` with identity rotation and unit
    /// scale.
    pub fn from_translation(position: DVec3) -> Self {
        Self {
            position,
            ..Self::IDENTITY
        }
    }

    /// Transforms `point` by this transform: scale, then rotate, then translate,
    /// matching the `Translation * Rotation * Scale` order.
    pub fn transform_point(&self, point: DVec3) -> DVec3 {
        self.position + self.rotation * (self.scale * point)
    }

    /// Composes `self`, a parent world transform, with `child`, a local transform,
    /// returning the child's world transform. Rotation composes as the Hamilton
    /// product `parent * child`; scale is the lossy component-wise product,
    /// dropping the shear that a rotation between non-uniform scales introduces.
    pub fn compose(&self, child: &Self) -> Self {
        Self {
            position: self.transform_point(child.position),
            rotation: self.rotation * child.rotation,
            scale: self.scale * child.scale,
        }
    }

    /// This transform as a [`TyPoseF64`], dropping the scale.
    pub fn to_pose(&self) -> TyPoseF64 {
        TyPoseF64::new(self.position, self.rotation)
    }

    /// This transform as a [`TyUniformTrsF64`], taking `scale.x` as the uniform
    /// factor. Assumes the scale is uniform.
    pub fn to_uniform_trs(&self) -> TyUniformTrsF64 {
        TyUniformTrsF64::new(self.position, self.rotation, self.scale.x)
    }
}

impl Default for TyTransformF64 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use crate::{TyQuaternionF64, TyTransformF64, TyVector3F64};
    use std::f64::consts::PI;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn from_translation_places_position_with_identity_rotation_and_unit_scale() {
        let transform = TyTransformF64::from_translation(TyVector3F64::new(1.0, 2.0, 3.0));
        assert_eq!(
            transform,
            TyTransformF64::new(
                TyVector3F64::new(1.0, 2.0, 3.0),
                TyQuaternionF64::IDENTITY,
                TyVector3F64::new(1.0, 1.0, 1.0),
            )
        );
    }

    #[test]
    fn transform_point_scales_then_rotates_then_translates() {
        let transform = TyTransformF64::new(
            TyVector3F64::new(1.0, 0.0, 0.0),
            TyQuaternionF64::IDENTITY,
            TyVector3F64::new(2.0, 2.0, 2.0),
        );
        // 2 * (1, 1, 1) then + (1, 0, 0).
        let point = transform.transform_point(TyVector3F64::new(1.0, 1.0, 1.0));
        assert!(close(point.x, 3.0) && close(point.y, 2.0) && close(point.z, 2.0));
    }

    #[test]
    fn compose_with_the_identity_returns_the_child() {
        let child = TyTransformF64::new(
            TyVector3F64::new(1.0, 2.0, 3.0),
            TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.0, 0.0, 1.0), 0.5),
            TyVector3F64::new(2.0, 3.0, 4.0),
        );
        let world = TyTransformF64::default().compose(&child);
        assert!(close(world.position.x, 1.0) && close(world.position.y, 2.0));
        assert!(
            close(world.scale.x, 2.0) && close(world.scale.y, 3.0) && close(world.scale.z, 4.0)
        );
        assert!(
            close(world.rotation.z, child.rotation.z) && close(world.rotation.w, child.rotation.w)
        );
    }

    #[test]
    fn compose_places_a_child_in_the_parent_frame() {
        // The parent rotates a quarter turn about z and sits at +x. The child at
        // local +x lands one unit along world +y from the parent, at (1, 1, 0).
        let parent = TyTransformF64::new(
            TyVector3F64::new(1.0, 0.0, 0.0),
            TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.0, 0.0, 1.0), PI / 2.0),
            TyVector3F64::new(1.0, 1.0, 1.0),
        );
        let child = TyTransformF64::new(
            TyVector3F64::new(1.0, 0.0, 0.0),
            TyQuaternionF64::IDENTITY,
            TyVector3F64::new(1.0, 1.0, 1.0),
        );
        let world = parent.compose(&child);
        assert!(
            close(world.position.x, 1.0)
                && close(world.position.y, 1.0)
                && close(world.position.z, 0.0)
        );
    }
}
