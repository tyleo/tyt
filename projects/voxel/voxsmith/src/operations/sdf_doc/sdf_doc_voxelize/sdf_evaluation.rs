use ty_math::TyVector3F64;

/// A 3D shape's value at a point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfEvaluation {
    /// The signed distance in meters, which reads zero or less inside the
    /// shape.
    pub distance: f64,

    /// The point carried back to the frame of the primitive that decides the
    /// distance. A pattern reads its coordinates there.
    pub frame_position: TyVector3F64,
}
