use crate::{VoxjPositionEncoding, VoxjSampleEncoding};

/// The resolved block-encoding strategy for a `to voxj` document: either fixed
/// position/sample encodings or the smallest-deflated search across encodings.
/// The command resolves its `--position-encoding`/`--sample-encoding`/
/// `--optimize` flags into this codec-free choice before handing it to the
/// implementation.
#[derive(Clone, Copy, Debug)]
pub enum VoxjEncoding {
    /// Fixed position and sample block encodings.
    Fixed {
        /// Position-block encoding.
        position: VoxjPositionEncoding,
        /// Sample-block encoding.
        sample: VoxjSampleEncoding,
    },
    /// Search every non-raw pairing and keep the smallest deflated result.
    Smallest,
}
