use ty_math::TyAxis3;

/// One rotation of a part about an axis through its pivot: an entry of
/// [`SdfNode::rotation`](crate::SdfNode::rotation).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfAxisRotation {
    /// The axis of the parent's frame the part rotates about.
    pub axis: TyAxis3,

    /// How far the part rotates.
    pub degrees: f64,
}
