use crate::{
    VoxValue,
    ext::{Result, VoxExt, VoxExtBlockCodec},
};
use std::any::Any;

/// An ordered set of key/value pairs: the object form of a [`VoxValue`].
///
/// Insertion order is preserved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoxMap(pub Vec<(String, VoxValue)>);

/// A block kept verbatim, whichever format owns it. It cannot follow a hook,
/// so a block loaded this way goes stale under a mutation that moves a
/// listing.
impl VoxExt for VoxMap {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        Ok(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn VoxExt> {
        Box::new(self.clone())
    }
}

/// A block kept verbatim, whichever format owns it. An absent block loads as
/// an empty map.
impl VoxExtBlockCodec for VoxMap {
    fn from_vox_ext_block(ext: Option<&VoxMap>) -> Result<Self> {
        Ok(ext.cloned().unwrap_or_default())
    }
}
