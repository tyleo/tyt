use crate::{VoxValue, ext::Result};

/// One format's ext as its entry in a document's ext block: the value under
/// the format's vendor key. A format owns one key. A block with several
/// entries comes only from an ext that holds several formats' exts.
/// [`encode_entry`](crate::ext::encode_entry) builds the one-entry block and
/// [`decode_entry`](crate::ext::decode_entry) finds the entry in a block. An
/// absent key means the block belongs to another format, so decoding takes
/// the entry itself and never sees absence.
pub trait VoxExtEntryCodec: Sized {
    /// The vendor key the entry sits under.
    const KEY: &'static str;

    /// Encodes the ext as its entry value.
    fn to_vox_ext_entry(&self) -> Result<VoxValue>;

    /// Decodes the ext from its entry value. An entry under this format's
    /// key that does not decode is an error.
    fn from_vox_ext_entry(value: &VoxValue) -> Result<Self>;
}
