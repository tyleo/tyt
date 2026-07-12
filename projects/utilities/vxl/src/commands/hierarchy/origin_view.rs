/// Resolved `--show-edit-origins` or `--show-runtime-origins` arguments: the
/// space and decimal precision an object's grid-min corner renders in.
#[derive(Clone, Copy, Debug)]
pub struct OriginView {
    /// Render in world space rather than local.
    pub world: bool,

    /// Decimal places for each component.
    pub precision: usize,
}
