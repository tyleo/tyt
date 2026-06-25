/// The encoding used for a voxel-sample block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampleEncoding {
    /// One row of cell indices per voxel.
    RawJson,

    /// One run-length-encoded channel per palette.
    RleJson,

    /// One bit-packed, base64-encoded channel per palette.
    PackedBase64,
}
