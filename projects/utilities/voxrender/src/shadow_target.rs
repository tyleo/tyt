use ty_math::TyVector3F64;

/// Where a shadow ray heads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShadowTarget {
    /// Toward a directional light, along a unit direction, without end.
    Direction(TyVector3F64),

    /// Toward a point or spot light at a position, ending there.
    Position(TyVector3F64),
}
