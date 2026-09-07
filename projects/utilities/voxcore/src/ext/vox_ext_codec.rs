use crate::{VoxMap, ext::Result};

/// One format's ext as its entry in a document's ext block. A format's ext
/// type implements this to encode itself under the format's vendor key and to
/// find itself in a block again. The other half is
/// [`VoxExtBlockCodec`](crate::ext::VoxExtBlockCodec), the whole block as the
/// ext a state carries, which may be nothing. Its blanket impl makes an
/// `Option` of this ext a whole-block codec.
///
/// The vendor key says which format owns the entry, so a loader expecting
/// another format sees a foreign block instead of a decode error.
pub trait VoxExtCodec: Sized {
    /// Encodes the ext as a block holding one entry under the format's key.
    fn to_vox_ext(&self) -> Result<VoxMap>;

    /// Decodes the ext from its entry in `ext`, or `None` when the block
    /// belongs to another format. An entry this codec owns but cannot decode
    /// is an error.
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
