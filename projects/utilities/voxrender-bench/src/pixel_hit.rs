/// A pixel's first hit, as the B1 id buffer stores it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PixelHit {
    /// The placement's index in the scene's placement order.
    pub placement: u32,

    /// The voxel id of the hit cell.
    pub voxel: u32,

    /// The face's axis times two, plus one when its normal points up the
    /// axis.
    pub face: u32,
}
