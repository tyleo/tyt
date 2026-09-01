use crate::{VoxMap, ext::Result};

/// A state ext with a value-tree form that a document write persists in an
/// ext block and a load reads back. The typed loads and writes handle an
/// `Option` slot of this ext through
/// [`VoxExtSlot`](crate::ext::VoxExtSlot)'s blanket impl.
///
/// Each format keeps its ext under its vendor key of the block. The key says
/// which format owns the ext, and a loader expecting another format sees a
/// foreign block instead of a decode error.
pub trait VoxExtCodec: Sized {
    /// Encodes the ext into an ext block.
    fn to_vox_ext(&self) -> Result<VoxMap>;

    /// Decodes the ext this codec owns from an ext block, or `None` when the
    /// block belongs to another format. A block this codec owns but cannot
    /// decode is an error.
    fn from_vox_ext(ext: &VoxMap) -> Result<Option<Self>>;
}

/// Keeps a block verbatim, whichever format owns it. A re-encode through an
/// untyped state then preserves the block without interpreting it.
impl VoxExtCodec for VoxMap {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        Ok(self.clone())
    }

    fn from_vox_ext(ext: &VoxMap) -> Result<Option<Self>> {
        Ok(Some(ext.clone()))
    }
}
