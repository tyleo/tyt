use ty_math::TyVector3F64;

/// A ray in world space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderRay {
    /// Where the ray starts, in meters.
    pub origin: TyVector3F64,

    /// The unit direction.
    pub direction: TyVector3F64,
}
