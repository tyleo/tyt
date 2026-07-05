use crate::{TyPose, TyQuaternion, TyUniformTrs, TyVector3};

/// A node transform with component type `T`, composing as
/// `Translation * Rotation * Scale`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TyTransform<T> {
    /// The translation, which may be fractional.
    pub position: TyVector3<T>,

    /// The rotation, a unit quaternion.
    pub rotation: TyQuaternion<T>,

    /// The per-axis scale.
    pub scale: TyVector3<T>,
}

impl<T> TyTransform<T> {
    /// Creates a new transform from a `position`, `rotation`, and `scale`.
    pub fn new(position: TyVector3<T>, rotation: TyQuaternion<T>, scale: TyVector3<T>) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }
}

/// Implements the float-only transform [`Default`] (zero position, identity
/// rotation, unit scale) for a concrete floating-point component type.
macro_rules! impl_ty_transform_float {
    ($t:ty) => {
        impl TyTransform<$t> {
            /// A translation-only transform: `position` with identity rotation
            /// and unit scale.
            pub fn from_translation(position: TyVector3<$t>) -> Self {
                Self {
                    position,
                    ..Self::default()
                }
            }

            /// Transforms `point` by this transform: scale, then rotate, then
            /// translate, matching the `Translation * Rotation * Scale` order.
            pub fn transform_point(&self, point: TyVector3<$t>) -> TyVector3<$t> {
                self.position
                    + self
                        .rotation
                        .rotate(self.scale.componentwise_multiply(&point))
            }

            /// Composes `self`, a parent world transform, with `child`, a local
            /// transform, returning the child's world transform. Rotation
            /// composes as the Hamilton product `parent * child`; scale is the
            /// lossy component-wise product, dropping the shear that a rotation
            /// between non-uniform scales introduces.
            pub fn compose(&self, child: &Self) -> Self {
                Self {
                    position: self.transform_point(child.position),
                    rotation: self.rotation * child.rotation,
                    scale: self.scale.componentwise_multiply(&child.scale),
                }
            }

            /// This transform as a [`TyPose`], dropping the scale.
            pub fn to_pose(&self) -> TyPose<$t> {
                TyPose::new(self.position, self.rotation)
            }

            /// This transform as a [`TyUniformTrs`], taking `scale.x` as the
            /// uniform factor. Assumes the scale is uniform.
            pub fn to_uniform_trs(&self) -> TyUniformTrs<$t> {
                TyUniformTrs::new(self.position, self.rotation, self.scale.x)
            }
        }

        impl Default for TyTransform<$t> {
            fn default() -> Self {
                Self {
                    position: TyVector3::new(0.0, 0.0, 0.0),
                    rotation: TyQuaternion::<$t>::identity(),
                    scale: TyVector3::new(1.0, 1.0, 1.0),
                }
            }
        }
    };
}

impl_ty_transform_float!(f32);
impl_ty_transform_float!(f64);

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
                TyQuaternionF64::identity(),
                TyVector3F64::new(1.0, 1.0, 1.0),
            )
        );
    }

    #[test]
    fn transform_point_scales_then_rotates_then_translates() {
        let transform = TyTransformF64::new(
            TyVector3F64::new(1.0, 0.0, 0.0),
            TyQuaternionF64::identity(),
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
        // Parent: a quarter turn about z, then translate +x. Its own +x child
        // sits one unit along the parent's rotated x, which is world +y, plus
        // the parent translation.
        let parent = TyTransformF64::new(
            TyVector3F64::new(1.0, 0.0, 0.0),
            TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.0, 0.0, 1.0), PI / 2.0),
            TyVector3F64::new(1.0, 1.0, 1.0),
        );
        let child = TyTransformF64::new(
            TyVector3F64::new(1.0, 0.0, 0.0),
            TyQuaternionF64::identity(),
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
