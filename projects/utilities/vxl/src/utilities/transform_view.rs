/// Resolved `--show-transforms` arguments: the space, angle unit, and decimal
/// precision to render each node's transform in.
#[derive(Clone, Copy, Debug)]
pub struct TransformView {
    /// Render in world space rather than local.
    pub world: bool,

    /// Render rotation in degrees rather than radians.
    pub degrees: bool,

    /// Decimal places for each component.
    pub precision: usize,
}
