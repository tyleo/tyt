/// Resolved `--show-bounds` or `--show-extents` arguments: the space and decimal
/// precision to render each object's grid box in.
#[derive(Clone, Copy, Debug)]
pub struct BoundsView {
    /// Render in world space rather than local.
    pub world: bool,

    /// Decimal places for each component.
    pub precision: usize,
}
