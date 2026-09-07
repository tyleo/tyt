use crate::{
    VoxMap,
    ext::{Result, VoxExtCodec},
};

/// A document's whole ext block, as the ext a [`VoxMain`](crate::VoxMain)
/// carries. The loaders and writers take this trait as the bound on the
/// state's `T`. [`VoxExtCodec`] is the other half: one format's entry inside
/// the block. A state may carry no ext and a document may carry no block, so
/// both directions are optional here.
///
/// The `Option` impl is a blanket here because a format crate cannot
/// implement a foreign trait for an `Option` of its ext. It also covers
/// `Option<VoxMap>`, a block kept verbatim.
pub trait VoxExtBlockCodec: Sized {
    /// Encodes the ext as a document's ext block, or `None` when the state
    /// has no ext to persist.
    fn to_vox_ext_block(&self) -> Result<Option<VoxMap>>;

    /// Builds the ext from a document's ext block, or from `None` when the
    /// document carries no block.
    fn from_vox_ext_block(ext: Option<&VoxMap>) -> Result<Self>;
}

/// No ext. A load drops any block the document carries, and a write emits
/// none.
impl VoxExtBlockCodec for () {
    fn to_vox_ext_block(&self) -> Result<Option<VoxMap>> {
        Ok(None)
    }

    fn from_vox_ext_block(_ext: Option<&VoxMap>) -> Result<Self> {
        Ok(())
    }
}

/// An optional format ext. Absent stays absent, and present goes through the
/// ext's [`VoxExtCodec`], so a block another format owns loads as `None`.
impl<T: VoxExtCodec> VoxExtBlockCodec for Option<T> {
    fn to_vox_ext_block(&self) -> Result<Option<VoxMap>> {
        self.as_ref().map(|ext| ext.to_vox_ext()).transpose()
    }

    fn from_vox_ext_block(ext: Option<&VoxMap>) -> Result<Self> {
        match ext {
            Some(ext) => T::from_vox_ext(ext),
            None => Ok(None),
        }
    }
}
