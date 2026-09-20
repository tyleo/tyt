use crate::{dependencies::mesh::PngChannels, operations::mesh::Transfer};

/// An 8-bit image for the PNG encoder, its texels row-major from the
/// top-left.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PngImage {
    /// Width in texels.
    pub width: u32,

    /// Height in texels.
    pub height: u32,

    pub channels: PngChannels,

    /// The transfer the color chunks declare.
    pub transfer: Transfer,

    /// `width * height * channels.count()` samples.
    pub samples: Vec<u8>,
}
