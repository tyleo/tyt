/// One spec validation check, the unit a failure is attributed to and the unit
/// [`check_voxj_file`](crate::check_voxj_file()) reports a result for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    /// The version is recognized.
    Version,

    /// Every value pool has non-empty `values`.
    ValuePools,

    /// Per palette:
    /// 1. every property has a non-empty name, distinct within the palette,
    ///    and an in-range value pool;
    /// 2. materials hold at least one row, one per material, each of
    ///    exactly one in-range value-index per property.
    Palettes,

    /// Object layers, node children, child objects, and roots resolve; node
    /// children, child objects, and roots are each listed at most once.
    Indices,

    /// Each object's position and sample blocks decode: recognized structure,
    /// canonical base64, exact bitmap and packed byte counts with zero pad
    /// bits, well-formed run streams and varints, the Hilbert bits cap, and one
    /// channel per layer with one value per voxel.
    Blocks,

    /// Voxel positions within an object are unique.
    UniquePositions,

    /// Positions lie within bounds and bounds are exactly tight around them.
    Bounds,

    /// Each sample indexes a real material of its layer's palette.
    SampleMaterials,

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
            Check::ValuePools => "value-pools",
            Check::Palettes => "palettes",
            Check::Indices => "indices",
            Check::Blocks => "blocks",
            Check::UniquePositions => "unique-positions",
            Check::Bounds => "bounds",
            Check::SampleMaterials => "sample-materials",
            Check::Acyclic => "acyclic",
            Check::Scale => "scale",
            Check::Rotation => "rotation",
            Check::EditState => "edit-state",
            Check::SampleOrder => "sample-order",
        }
    }
}
