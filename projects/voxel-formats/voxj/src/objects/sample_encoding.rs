/// The encoding used for a voxel-sample block. Every encoding lays out one
/// channel per layer, in `layers` order, each channel a material index into
/// that layer's palette for every voxel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampleEncoding {
    /// One channel of material indices per layer.
    RawJson,

    /// One run-length-encoded channel per layer.
    RleJson,

    /// One bit-packed, base64-encoded channel per layer.
    PackedBase64,
}
