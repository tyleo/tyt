use crate::{
    InstalledFormat, ReadFormat, ReadFormatVisitor, Result,
    ext::VoxconvExt,
    voxj::ext::{CompositeVoxExt, InertVoxExt},
};
use voxcore::VoxMapEntry;
use voxj_voxcore::VoxjVoxExt;

/// Decodes a document's `ext` block into a [`CompositeVoxExt`]. Each slot
/// under an installed format's key becomes that format's ext, and every
/// other slot an [`InertVoxExt`], in the block's order. Errors when a slot
/// under a format's key does not decode.
pub fn composite_vox_ext_from_voxj_vox_ext(ext: VoxjVoxExt) -> Result<CompositeVoxExt> {
    let slots = ext.into_slots();

    let mut exts = Vec::with_capacity(slots.len());

    for slot in slots {
        exts.push(slot_ext(slot)?);
    }

    Ok(CompositeVoxExt { exts })
}

/// The ext for one slot: the installed format's ext under its key, else the
/// slot kept inert.
fn slot_ext(slot: VoxMapEntry) -> Result<Box<dyn VoxconvExt>> {
    for &format in ReadFormat::ALL {
        if let Some(ext) = format.with(DecodeSlot(&slot))? {
            return Ok(ext);
        }
    }

    Ok(Box::new(InertVoxExt {
        key: slot.key,
        value: slot.value,
    }))
}

/// The ext a slot decodes to as one format's.
struct DecodeSlot<'a>(&'a VoxMapEntry);

impl ReadFormatVisitor for DecodeSlot<'_> {
    type Output = Result<Option<Box<dyn VoxconvExt>>>;

    fn visit<F: InstalledFormat>(self) -> Self::Output {
        F::decode_slot(self.0)
    }
}
