/// The encoding used for a voxel-sample block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampleEncoding {
    /// One channel of cell indices per palette.
    RawJson,

    /// One run-length-encoded channel per palette.
    RleJson,

    /// One bit-packed, base64-encoded channel per palette.
    PackedBase64,
}
