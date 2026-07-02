use crate::{TyBounds, TyQuaternion, TyUniformTrs, TyVector3};

/// A rigid pose with component type `T`: a rotation and position, no scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TyPose<T = f64> {
    /// The position.
    pub position: TyVector3<T>,

    /// The rotation, a unit quaternion.
    pub rotation: TyQuaternion<T>,
}

impl<T> TyPose<T> {
    /// Creates a pose from a `position` and a `rotation`.
    pub fn new(position: TyVector3<T>, rotation: TyQuaternion<T>) -> Self {
        Self { position, rotation }
    }
}

/// Implements the float-only pose operations for a concrete floating-point
/// component type.
macro_rules! impl_ty_pose_float {
    ($t:ty) => {
        impl TyPose<$t> {
            /// The identity pose: zero position, identity rotation.
            pub fn identity() -> Self {
                Self {
                    position: TyVector3::<$t>::ZERO,
                    rotation: TyQuaternion::<$t>::identity(),
                }
            }

            /// The pose of `target` expressed in the local space of `self`.
            pub fn calculate_relative_pose(&self, target: &Self) -> Self {
                let inverse_rotation = self.rotation.inverse();
                let relative_position = inverse_rotation.rotate(target.position - self.position);
                let relative_rotation = inverse_rotation * target.rotation;

                Self::new(relative_position, relative_rotation)
            }

            /// Transforms `aabb` by this pose, growing it to the axis-aligned
            /// bound of the rotated box.
            pub fn transform_aabb_conservative(&self, aabb: &TyBounds<$t>) -> TyBounds<$t> {
                let center = self.position + self.rotation.rotate(aabb.center);
                let extents = self.rotation.rotate_extents_abs(aabb.extents);

                TyBounds::new(center, extents)
            }

            /// This pose with a uniform `scale`, as a [`TyUniformTrs`].
            pub fn with_uniform_scale(&self, scale: $t) -> TyUniformTrs<$t> {
                TyUniformTrs::new(self.position, self.rotation, scale)
            }
        }

        impl Default for TyPose<$t> {
            fn default() -> Self {
                Self::identity()
            }
        }
    };
}

impl_ty_pose_float!(f32);
impl_ty_pose_float!(f64);

#[cfg(test)]
mod tests {
    use crate::{TyPoseF64, TyQuaternionF64, TyVector3F64};
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
                .is_approximately_equal(TyQuaternionF64::identity(), 1e-9)
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
        let target = TyPoseF64::new(
            TyVector3F64::new(2.0, 0.0, 0.0),
            TyQuaternionF64::identity(),
        );
        let relative = parent.calculate_relative_pose(&target);
        assert!(
            close(relative.position.x, 0.0)
                && close(relative.position.y, -1.0)
                && close(relative.position.z, 0.0)
        );
    }
}
