use crate::{Result, VoxjExtCodec};
use voxj::VoxjMap;

/// A document `ext` block carried verbatim, without interpreting which format
/// owns it. Load a document with this ext type to re-encode it with the block
/// intact when the owning format is not known up front.
#[derive(Clone, Debug, PartialEq)]
pub struct VoxjRawExt(pub VoxjMap);

impl VoxjExtCodec for VoxjRawExt {
    fn to_voxj_ext(&self) -> Result<VoxjMap> {
        Ok(self.0.clone())
    }

    fn from_voxj_ext(ext: &VoxjMap) -> Result<Option<Self>> {
        Ok(Some(VoxjRawExt(ext.clone())))
    }
}
