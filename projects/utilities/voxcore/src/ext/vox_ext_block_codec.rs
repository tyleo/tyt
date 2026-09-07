use crate::{
    VoxMap,
    ext::{Result, VoxExtEntryCodec, decode_entry},
};

/// Builds the ext a [`VoxMain`](crate::VoxMain) carries from a document's ext
/// block. A loader takes it as the bound on the state's `T`. A document may
/// carry no block. Writing goes through
/// [`VoxExt::to_vox_ext`](crate::ext::VoxExt::to_vox_ext).
///
/// The `Option` impls live here because a format crate cannot implement a
/// foreign trait for an `Option` of its ext.
pub trait VoxExtBlockCodec: Sized {
    /// Builds the ext from a document's ext block, or from `None` when the
    /// document carries no block.
    fn from_vox_ext_block(ext: Option<&VoxMap>) -> Result<Self>;
}

/// No ext. A load drops any block the document carries.
impl VoxExtBlockCodec for () {
    fn from_vox_ext_block(_ext: Option<&VoxMap>) -> Result<Self> {
        Ok(())
    }
}

/// An optional format ext. Absent stays absent, and present goes through the
/// ext's entry, so a block another format owns loads as `None`.
impl<T: VoxExtEntryCodec> VoxExtBlockCodec for Option<T> {
    fn from_vox_ext_block(ext: Option<&VoxMap>) -> Result<Self> {
        match ext {
            Some(block) => decode_entry(block),
            None => Ok(None),
        }
    }
}

/// A block kept verbatim, whichever format owns it. Absent stays absent.
impl VoxExtBlockCodec for Option<VoxMap> {
    fn from_vox_ext_block(ext: Option<&VoxMap>) -> Result<Self> {
        Ok(ext.cloned())
    }
}

#[cfg(test)]
mod tests {
    use crate::{VoxMap, VoxValue, ext::VoxExtBlockCodec};

    fn block() -> VoxMap {
        VoxMap(vec![("any".to_owned(), VoxValue::Bool(true))])
    }

    #[test]
    fn the_unit_ext_drops_any_block() {
        <()>::from_vox_ext_block(Some(&block())).unwrap();

        <()>::from_vox_ext_block(None).unwrap();
    }

    #[test]
    fn a_verbatim_block_loads_as_it_was() {
        assert_eq!(
            Option::<VoxMap>::from_vox_ext_block(Some(&block())).unwrap(),
            Some(block())
        );

        assert_eq!(Option::<VoxMap>::from_vox_ext_block(None).unwrap(), None);

        assert_eq!(VoxMap::from_vox_ext_block(Some(&block())).unwrap(), block());

        assert_eq!(VoxMap::from_vox_ext_block(None).unwrap(), VoxMap::default());
    }
}
