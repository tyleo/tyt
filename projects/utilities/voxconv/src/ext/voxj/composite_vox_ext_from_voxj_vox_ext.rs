use crate::{
    Result,
    ext::{CompositeVoxExt, InertVoxExt, VoxconvExt},
};
#[cfg(any(feature = "goxl", feature = "mvox", feature = "qbcl", feature = "vmax"))]
use crate::{VoxjEntry, ext_from_vox_value};
#[cfg(feature = "goxl")]
use goxl_voxcore::ext::GoxlExt;
#[cfg(feature = "mvox")]
use mvox_voxcore::ext::MVoxExt;
#[cfg(feature = "qbcl")]
use qbcl_voxcore::ext::{QbExt, QbclExt, QbtExt};
#[cfg(feature = "vmax")]
use vmax_voxcore::ext::VMaxExt;
use voxcore::VoxValue;
use voxj_voxcore::ext::VoxjVoxExt;

/// Decodes a document's `ext` block into a [`CompositeVoxExt`]. Each entry
/// under an enabled format's key becomes that format's ext, and every other
/// entry an [`InertVoxExt`], in the block's order. Errors when an entry
/// under a format's key does not decode.
pub fn composite_vox_ext_from_voxj_vox_ext(ext: VoxjVoxExt) -> Result<CompositeVoxExt> {
    let slots = ext.into_slots();
    let mut exts: Vec<Box<dyn VoxconvExt>> = Vec::with_capacity(slots.len());
    for slot in slots {
        exts.push(entry_ext(slot.key, slot.value)?);
    }
    Ok(CompositeVoxExt { exts })
}

/// The ext for one entry: the enabled format's ext under its key, else the
/// entry kept inert.
fn entry_ext(key: String, value: VoxValue) -> Result<Box<dyn VoxconvExt>> {
    #[cfg(feature = "goxl")]
    if key == GoxlExt::KEY {
        return Ok(Box::new(ext_from_vox_value::<GoxlExt>(&value)?));
    }

    #[cfg(feature = "mvox")]
    if key == MVoxExt::KEY {
        return Ok(Box::new(ext_from_vox_value::<MVoxExt>(&value)?));
    }

    #[cfg(feature = "qbcl")]
    if key == QbExt::KEY {
        return Ok(Box::new(ext_from_vox_value::<QbExt>(&value)?));
    }

    #[cfg(feature = "qbcl")]
    if key == QbtExt::KEY {
        return Ok(Box::new(ext_from_vox_value::<QbtExt>(&value)?));
    }

    #[cfg(feature = "qbcl")]
    if key == QbclExt::KEY {
        return Ok(Box::new(ext_from_vox_value::<QbclExt>(&value)?));
    }

    #[cfg(feature = "vmax")]
    if key == VMaxExt::KEY {
        return Ok(Box::new(ext_from_vox_value::<VMaxExt>(&value)?));
    }
    Ok(Box::new(InertVoxExt { key, value }))
}
