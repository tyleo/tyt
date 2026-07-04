/// One spec validation check, the unit a failure is attributed to and the unit
/// [`check_voxj_file`](crate::check_voxj_file()) reports a result for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    /// The version is recognized.
    Version,
    /// Palettes are rectangular, have distinct attribute keys, and any `rgba`
    /// value is a `#RRGGBBAA` string.
    Palettes,
    /// Palette refs, node children, and roots resolve and are listed at most
    /// once.
    Indices,
    /// Each object's position and sample blocks decode, with matching arity and
    /// per-channel lengths.
    Blocks,
    /// Voxel positions within an object are unique.
    UniquePositions,
    /// Positions lie within bounds and bounds are exactly tight around them.
    Bounds,
    /// Each sample indexes a real cell of the palette it samples.
    SampleCells,
    /// The hierarchy is acyclic.
    Acyclic,
    /// No transform scale component is zero.
    Scale,
    /// Every transform rotation is a unit quaternion.
    Rotation,
    /// Each edit grid contains its object's runtime grid.
    EditState,
    /// Sample order matches the position block's voxel order: an authoring
    /// invariant no document can witness.
    SampleOrder,
}

impl Check {
    /// The short stable identifier reported as [`VoxjCheck::name`](crate::VoxjCheck::name).
    pub fn name(self) -> &'static str {
        match self {
            Check::Version => "version",
            Check::Palettes => "palettes",
            Check::Indices => "indices",
            Check::Blocks => "blocks",
            Check::UniquePositions => "unique-positions",
            Check::Bounds => "bounds",
            Check::SampleCells => "sample-cells",
            Check::Acyclic => "acyclic",
            Check::Scale => "scale",
            Check::Rotation => "rotation",
            Check::EditState => "edit-state",
            Check::SampleOrder => "sample-order",
        }
    }
}
