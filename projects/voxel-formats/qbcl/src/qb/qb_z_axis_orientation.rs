/// The Z-axis handedness a `.qb` file was authored in, from the header's
/// `zAxisOrientation` field. It does not affect voxel storage; it records the
/// authoring convention so it round-trips.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QbZAxisOrientation {
    /// `zAxisOrientation == 0`: left-handed Z axis.
    LeftHanded,

    /// `zAxisOrientation == 1`: right-handed Z axis.
    RightHanded,
}
