use crate::{VoxjPositionEncoding, VoxjSampleEncoding};

/// The resolved per-block encoding choice for a `to voxj` document. The command
/// resolves its `--position-encoding`, `--sample-encoding`, and
/// `--encoding-preset` flags into this codec-free choice before handing it to
/// the implementation.
#[derive(Clone, Copy, Debug)]
pub struct VoxjEncoding {
    /// Position-block encoding, or `Smallest` to search.
    pub position: VoxjPositionEncoding,
    /// Sample-block encoding, or `Smallest` to search.
    pub sample: VoxjSampleEncoding,
}
