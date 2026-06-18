/// An encoded voxel-sample block, in the position block's voxel order. Each
/// voxel carries one cell index per referenced palette.
#[derive(Clone, Debug, PartialEq)]
pub enum SampleBlock {
    /// One row per voxel: that voxel's cell index per palette, in order.
    RawJson(Vec<Vec<u32>>),
    /// One channel per palette: a flat run stream `[value1, count1, ...]`.
    RleJson(Vec<Vec<u32>>),
    /// One channel per palette: each voxel's cell index bit-packed at width
    /// `max(1, bitLength(cellCount - 1))`, MSB-first, base64-encoded.
    PackedBase64(Vec<String>),
}
