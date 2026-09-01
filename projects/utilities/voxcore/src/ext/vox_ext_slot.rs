use crate::{
    VoxMap,
    ext::{Result, VoxExtCodec},
};

/// A whole state ext slot with a value-tree form: the ext block a document
/// writer persists, if any, and how a loader fills the slot back.
///
/// A format's slot is an `Option` of its ext, covered by the blanket impl
/// over the ext's [`VoxExtCodec`]. The unit slot carries nothing.
pub trait VoxExtSlot: Sized {
    /// Encodes the slot into an ext block, or `None` when the slot has
    /// nothing to persist.
    fn to_vox_ext(&self) -> Result<Option<VoxMap>>;

    /// Fills the slot from a document's ext block, absent when the document
    /// carries none.
    fn from_vox_ext(ext: Option<&VoxMap>) -> Result<Self>;
}

/// The unit slot declines the block: loading a document drops any block it
/// carries, and writing one emits none.
impl VoxExtSlot for () {
    fn to_vox_ext(&self) -> Result<Option<VoxMap>> {
        Ok(None)
    }

    fn from_vox_ext(_ext: Option<&VoxMap>) -> Result<Self> {
        Ok(())
    }
}

/// An optional value slot: absent stays absent, present goes through the
/// value's [`VoxExtCodec`].
impl<T: VoxExtCodec> VoxExtSlot for Option<T> {
    fn to_vox_ext(&self) -> Result<Option<VoxMap>> {
        self.as_ref().map(|ext| ext.to_vox_ext()).transpose()
    }

    fn from_vox_ext(ext: Option<&VoxMap>) -> Result<Self> {
        match ext {
            Some(ext) => T::from_vox_ext(ext),
            None => Ok(None),
        }
    }
}
