/// The on-disk channel order for `.qb` voxel colors, from the header's
/// `colorFormat` field. Colors are normalized to `RGB` in memory regardless;
/// this records the on-disk layout to read and write back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum QbColorFormat {
    /// `colorFormat == 0`: red, green, blue, then the visibility byte.
    #[default]
    Rgba,

    /// `colorFormat == 1`: blue, green, red, then the visibility byte.
    Bgra,
}
