/// Largest Hilbert `bits` per axis the format allows: the reference decoder
/// assembles an index in a JS double, exact only while `3 * bits <= 53`, so
/// `hilbert-delta-varint-base64` requires `bits <= 17` (every bounds
/// dimension `<= 131072`). A larger grid must use `bitmap-base64` or
/// `raw-json`.
pub const MAX_HILBERT_BITS: u32 = 17;
