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

/// Implements the float-only quaternion operations (the identity and its
/// [`Default`]) for a concrete floating-point component type.
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
