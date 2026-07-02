use crate::{TyBounds, TyPose, TyQuaternion, TyVector3};

/// A transform with component type `T` and a single uniform scale factor. The
/// scalar-scale companion to [`TyTransform`](crate::TyTransform).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TyUniformTrs<T = f64> {
    /// The translation.
    pub translation: TyVector3<T>,

    /// The rotation, a unit quaternion.
    pub rotation: TyQuaternion<T>,

    /// The uniform scale factor.
    pub scale: T,
}

impl<T> TyUniformTrs<T> {
    /// Creates a transform from a `translation`, `rotation`, and uniform `scale`.
    pub fn new(translation: TyVector3<T>, rotation: TyQuaternion<T>, scale: T) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }
}

/// Implements the float-only transform operations for a concrete floating-point
/// component type.
macro_rules! impl_ty_uniform_trs_float {
    ($t:ty) => {
        impl TyUniformTrs<$t> {
            /// The identity transform: zero translation, identity rotation, unit
            /// scale.
            pub fn identity() -> Self {
                Self {
                    translation: TyVector3::<$t>::ZERO,
                    rotation: TyQuaternion::<$t>::identity(),
                    scale: 1.0,
                }
            }

            /// The transform of `target` expressed in the local space of `self`.
            /// Assumes a positive scale.
            pub fn calculate_relative_trs(&self, target: &Self) -> Self {
                let inverse_rotation = self.rotation.inverse();
                let inverse_scale = 1.0 / self.scale;

                let delta = target.translation - self.translation;
                let relative_translation = inverse_rotation.rotate(delta * inverse_scale);
                let relative_rotation = inverse_rotation * target.rotation;
                let relative_scale = target.scale * inverse_scale;

                Self::new(relative_translation, relative_rotation, relative_scale)
            }

            /// This transform as a [`TyPose`], dropping the scale.
            pub fn to_pose(&self) -> TyPose<$t> {
                TyPose::new(self.translation, self.rotation)
            }

            /// Transforms `aabb` by this transform, growing it to the axis-aligned
            /// bound of the scaled and rotated box. Assumes a positive scale.
            pub fn transform_aabb_conservative(&self, aabb: &TyBounds<$t>) -> TyBounds<$t> {
                let center = self.translation + self.rotation.rotate(aabb.center * self.scale);
                let extents = self.rotation.rotate_extents_abs(aabb.extents * self.scale);

                TyBounds::new(center, extents)
            }
        }

        impl Default for TyUniformTrs<$t> {
            fn default() -> Self {
                Self::identity()
            }
        }
    };
}

impl_ty_uniform_trs_float!(f32);
impl_ty_uniform_trs_float!(f64);

#[cfg(test)]
mod tests {
    use crate::{TyQuaternionF64, TyUniformTrsF64, TyVector3F64};

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
                .is_approximately_equal(TyQuaternionF64::identity(), 1e-9)
        );
    }

    #[test]
    fn relative_scale_is_the_ratio() {
        let parent = TyUniformTrsF64::new(TyVector3F64::ZERO, TyQuaternionF64::identity(), 2.0);
        let child = TyUniformTrsF64::new(TyVector3F64::ZERO, TyQuaternionF64::identity(), 6.0);
        assert!(close(parent.calculate_relative_trs(&child).scale, 3.0));
    }
}
