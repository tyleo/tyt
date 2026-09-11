use crate::{Result, ext::VoxconvExt};
#[cfg(feature = "voxj")]
use crate::{VoxjEntry, ext::voxj_vox_ext_from_ext, ext_from_vox_value};

/// The format ext `E` in a boxed ext, cloned out. A box holding an `E` gives
/// it as it is. Any other box encodes to its Voxel Json `ext` block, and the
/// entry under `E`'s key decodes. A box with no such entry, such as another
/// format's ext, gives `None`.
#[cfg(feature = "voxj")]
pub fn find_ext<E: VoxjEntry>(ext: &dyn VoxconvExt) -> Result<Option<E>> {
    if let Some(ext) = ext.downcast_ref::<E>() {
        return Ok(Some(ext.clone()));
    }
    let block = voxj_vox_ext_from_ext(ext)?;
    let Some(value) = block.slot(E::KEY) else {
        return Ok(None);
    };
    Ok(Some(ext_from_vox_value(value)?))
}

/// The format ext `E` in a boxed ext, cloned out. Without the `voxj`
/// feature no box encodes to an `ext` block, so only a box holding an `E`
/// gives one.
#[cfg(not(feature = "voxj"))]
pub fn find_ext<E: VoxconvExt + Clone>(ext: &dyn VoxconvExt) -> Result<Option<E>> {
    Ok(ext.downcast_ref::<E>().cloned())
}
