/// How a 4-component color's alpha takes part in quantizing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlphaMode {
    /// Alpha adds a fourth clustering coordinate.
    Distance,

    /// Alpha stays out of clustering, so a merged voxel takes its
    /// representative's alpha.
    Ignore,

    /// Materials merge only when their alpha matches exactly.
    Partition,
}
