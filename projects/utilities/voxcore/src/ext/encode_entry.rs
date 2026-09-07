use crate::{
    VoxMap,
    ext::{Result, VoxExtEntryCodec},
};

/// The one-entry block a format ext persists: its entry under its key.
pub fn encode_entry<E: VoxExtEntryCodec>(ext: &E) -> Result<VoxMap> {
    Ok(VoxMap(vec![(E::KEY.to_owned(), ext.to_vox_ext_entry()?)]))
}
