/// The RLE marker: a stream integer whose high mask byte holds this value is a
/// run header carrying the run length in its low byte.
pub const RLE_MASK: u8 = 2;
