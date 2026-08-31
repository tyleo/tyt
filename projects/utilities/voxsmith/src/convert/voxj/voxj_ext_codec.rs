use crate::Result;
use voxcore::VoxMap;

/// A state ext with a Voxel Json representation, so a document write can
/// persist it in the `ext` block and a load can read it back. The typed
/// loads and writes handle an `Option` slot of this ext through
/// [`VoxjExtSlot`](crate::VoxjExtSlot)'s blanket impl.
///
/// Each format keeps its ext under its vendor key of the block. The key says
/// which format owns the ext, and a loader expecting another format sees a
/// foreign block instead of a decode error.
pub trait VoxjExtCodec: Sized {
    /// Encodes the ext into a document's `ext` block.
    fn to_voxj_ext(&self) -> Result<VoxMap>;

    /// Decodes the ext this codec owns from a document's `ext` block, or
    /// `None` when the block belongs to another format. A block this codec
    /// owns but cannot decode is an error.
    fn from_voxj_ext(ext: &VoxMap) -> Result<Option<Self>>;
}

/// Keeps a block verbatim, whichever format owns it, so a re-encode through
/// [`VoxjVoxMain`](voxj_voxcore::VoxjVoxMain) preserves the block without
/// interpreting it.
impl VoxjExtCodec for VoxMap {
    fn to_voxj_ext(&self) -> Result<VoxMap> {
        Ok(self.clone())
    }

    fn from_voxj_ext(ext: &VoxMap) -> Result<Option<Self>> {
        Ok(Some(ext.clone()))
    }
}
