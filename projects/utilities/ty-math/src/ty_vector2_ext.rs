use glam::{DVec2, DVec3, Vec2, Vec3};

/// The tyt-specific 2D vector operations glam does not name directly, carried on
/// the glam vector aliases so callers keep `v.foo()` with `use ty_math::TyVector2Ext`.
pub trait TyVector2Ext {
    /// The matching 3D vector type.
    type Vector3;

    /// This vector as a 3D vector with a `z` of `0`.
    fn to_vector3(self) -> Self::Vector3;
}

/// Implements [`TyVector2Ext`] for a concrete glam 2D vector and its 3D form.
macro_rules! impl_ty_vector2_ext {
    ($vec2:ty, $vec3:ty) => {
        impl TyVector2Ext for $vec2 {
            type Vector3 = $vec3;

            fn to_vector3(self) -> $vec3 {
                self.extend(0.0)
            }
        }
    };
}

impl_ty_vector2_ext!(DVec2, DVec3);
impl_ty_vector2_ext!(Vec2, Vec3);

#[cfg(test)]
mod tests {
    use crate::{TyVector2Ext, TyVector2F64, TyVector3F64};

    #[test]
    fn to_vector3_zeroes_z() {
        assert_eq!(
            TyVector2F64::new(1.0, 2.0).to_vector3(),
            TyVector3F64::new(1.0, 2.0, 0.0)
        );
    }
}
