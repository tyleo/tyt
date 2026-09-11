use crate::{
    Error, Result,
    ext::{CompositeVoxExt, InertVoxExt, VoxconvExt},
};
#[cfg(any(feature = "goxl", feature = "mvox", feature = "qbcl", feature = "vmax"))]
use crate::{VoxjEntry, vox_value_from_ext};
#[cfg(feature = "goxl")]
use goxl_voxcore::ext::GoxlExt;
#[cfg(feature = "mvox")]
use mvox_voxcore::ext::MVoxExt;
#[cfg(feature = "qbcl")]
use qbcl_voxcore::ext::{QbExt, QbclExt, QbtExt};
#[cfg(feature = "vmax")]
use vmax_voxcore::ext::VMaxExt;
use voxcore::{VoxMapEntry, VoxValue};
use voxj_voxcore::ext::VoxjVoxExt;

/// Encodes an ext as a document's `ext` block. `()` encodes an empty ext. An
/// enabled format's ext encodes as its entry, an [`InertVoxExt`] as the
/// entry it kept, a [`CompositeVoxExt`] as every entry it holds, and a
/// [`VoxjVoxExt`] as it is. Errors on a repeated key and on an ext outside
/// those.
pub fn voxj_vox_ext_from_ext(ext: &dyn VoxconvExt) -> Result<VoxjVoxExt> {
    let mut slots = Vec::new();
    push_entries(ext, &mut slots)?;
    Ok(VoxjVoxExt::new(slots))
}

/// Appends the slots `ext` encodes to `slots`.
fn push_entries(ext: &dyn VoxconvExt, slots: &mut Vec<VoxMapEntry>) -> Result<()> {
    if ext.is::<()>() {
        return Ok(());
    }
    if let Some(voxj) = ext.downcast_ref::<VoxjVoxExt>() {
        for slot in voxj.slots() {
            push_entry(slots, slot.key.clone(), slot.value.clone())?;
        }
        return Ok(());
    }
    if let Some(composite) = ext.downcast_ref::<CompositeVoxExt>() {
        for ext in &composite.exts {
            push_entries(ext.as_ref(), slots)?;
        }
        return Ok(());
    }
    if let Some(inert) = ext.downcast_ref::<InertVoxExt>() {
        return push_entry(slots, inert.key.clone(), inert.value.clone());
    }
    #[cfg(feature = "goxl")]
    if let Some(goxlext) = ext.downcast_ref::<GoxlExt>() {
        return push_known(slots, goxlext);
    }

    #[cfg(feature = "mvox")]
    if let Some(mvoxext) = ext.downcast_ref::<MVoxExt>() {
        return push_known(slots, mvoxext);
    }

    #[cfg(feature = "qbcl")]
    if let Some(qbext) = ext.downcast_ref::<QbExt>() {
        return push_known(slots, qbext);
    }

    #[cfg(feature = "qbcl")]
    if let Some(qbtext) = ext.downcast_ref::<QbtExt>() {
        return push_known(slots, qbtext);
    }

    #[cfg(feature = "qbcl")]
    if let Some(qbclext) = ext.downcast_ref::<QbclExt>() {
        return push_known(slots, qbclext);
    }

    #[cfg(feature = "vmax")]
    if let Some(vmaxext) = ext.downcast_ref::<VMaxExt>() {
        return push_known(slots, vmaxext);
    }
    Err(Error::Ext(
        "the ext is not one voxconv writes to Voxel Json".to_owned(),
    ))
}

/// Appends an enabled format's ext as the slot under its key.
#[cfg(any(feature = "goxl", feature = "mvox", feature = "qbcl", feature = "vmax"))]
fn push_known<E: VoxjEntry>(slots: &mut Vec<VoxMapEntry>, ext: &E) -> Result<()> {
    push_entry(slots, E::KEY.to_owned(), vox_value_from_ext(ext)?)
}

/// Appends one slot. Errors when the block already holds its key.
fn push_entry(slots: &mut Vec<VoxMapEntry>, key: String, value: VoxValue) -> Result<()> {
    if slots.iter().any(|slot| slot.key == key) {
        return Err(Error::Ext(format!("the ext block holds `{key}` twice")));
    }
    slots.push(VoxMapEntry { key, value });
    Ok(())
}
