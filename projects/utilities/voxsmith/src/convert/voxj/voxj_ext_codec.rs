use crate::Result;
use voxj::VoxjMap;

/// A state ext with a Voxel Json representation, so the document writers can
/// persist it in the `ext` block and the loaders can read it back. The
/// document fns handle an `Option` slot of this ext through
/// [`VoxjExtSlot`](crate::VoxjExtSlot)'s blanket impl.
///
/// Each format keeps its ext under its vendor key of the block. The key says
/// which format owns the ext, and a loader expecting another format sees a
/// foreign block instead of a decode error.
pub trait VoxjExtCodec: Sized {
    /// Encodes the ext into a document's `ext` block.
    fn to_voxj_ext(&self) -> Result<VoxjMap>;

    /// Decodes the ext this codec owns from a document's `ext` block, or
    /// `None` when the block belongs to another format. A block this codec
    /// owns but cannot decode is an error.
    fn from_voxj_ext(ext: &VoxjMap) -> Result<Option<Self>>;
}
