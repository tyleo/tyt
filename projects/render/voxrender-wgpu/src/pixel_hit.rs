/// A pixel's first hit and the corner shadow bits of its face.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PixelHit {
    /// The placement's index in the scene's placement order.
    pub placement: u32,

    /// The voxel id of the hit cell.
    pub voxel: u32,

    /// The face's axis times two, plus one when its normal points up the
    /// axis.
    pub face: u32,

    /// Per light that casts shadows: bit `c` is set when corner `c`'s shadow
    /// ray meets no surface. Bit 1 of `c` picks the face's far edge on `u`
    /// and bit 2 on `v`.
    pub lit: Vec<u8>,
}
