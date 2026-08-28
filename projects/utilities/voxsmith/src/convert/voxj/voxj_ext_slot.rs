use crate::{Result, VoxjExtCodec};
use voxj::VoxjMap;

/// A whole state ext slot with a Voxel Json representation: the `ext` block a
/// document writer persists, if any, and how a loader fills the slot back.
/// The [`Default`] value is the slot a loader starts from before it reads the
/// document's block.
///
/// A format's slot is an `Option` of its ext, covered by the blanket impl
/// over the ext's [`VoxjExtCodec`]. The unit slot carries nothing.
pub trait VoxjExtSlot: Default {
    /// Encodes the slot into a document's `ext` block, or `None` when the
    /// slot has nothing to persist.
    fn to_voxj_ext(&self) -> Result<Option<VoxjMap>>;

    /// Fills the slot from a document's `ext` block, absent when the document
    /// carries none.
    fn from_voxj_ext(ext: Option<&VoxjMap>) -> Result<Self>;
}

/// The unit slot declines the block: loading a document drops any block it
/// carries, and writing one emits none.
impl VoxjExtSlot for () {
    fn to_voxj_ext(&self) -> Result<Option<VoxjMap>> {
        Ok(None)
    }

    fn from_voxj_ext(_ext: Option<&VoxjMap>) -> Result<Self> {
        Ok(())
    }
}

/// An optional value slot: absent stays absent, present goes through the
/// value's [`VoxjExtCodec`].
impl<T: VoxjExtCodec> VoxjExtSlot for Option<T> {
    fn to_voxj_ext(&self) -> Result<Option<VoxjMap>> {
        self.as_ref().map(|ext| ext.to_voxj_ext()).transpose()
    }

    fn from_voxj_ext(ext: Option<&VoxjMap>) -> Result<Self> {
        match ext {
            Some(ext) => T::from_voxj_ext(ext),
            None => Ok(None),
        }
    }
}
