/// The visibility mask written for every synthesized solid voxel. Qubicle reads
/// a zero mask as an empty cell and any non-zero mask as a solid voxel whose
/// bits are a per-face visibility set; the exact bits are cosmetic, so this
/// mirrors the codec's solid fixture.
pub(crate) const SOLID_MASK: u8 = 0x7e;
