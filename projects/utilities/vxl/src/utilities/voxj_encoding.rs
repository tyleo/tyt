use crate::{VoxjPositionEncoding, VoxjSampleEncoding};

/// The resolved per-block encoding choice for a voxj document, a codec-free
/// choice handed to the implementation.
#[derive(Clone, Copy, Debug)]
pub struct VoxjEncoding {
    /// Position-block encoding, or `Smallest` to search.
    pub position: VoxjPositionEncoding,
    /// Sample-block encoding, or `Smallest` to search.
    pub sample: VoxjSampleEncoding,
}
