use crate::{TyQuaternion, TyVector3};

/// A node transform generic over its component type `T`, composing as
/// `Translation * Rotation * Scale`.
///
/// See `TyTransformF32` and `TyTransformF64` for the common instantiations.
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
