use crate::{Error, InstalledFormat, ReadFormat, ReadFormatVisitor, Result, ext::VoxconvExt};
use voxcore::VoxMapEntry;

/// Appends the Voxel Json `ext` block slots `ext` encodes to `slots`. `()`
/// encodes none. An installed format's ext encodes as its slots. Errors on
/// a repeated key and on an ext no installed format claims.
pub fn push_slots(ext: &dyn VoxconvExt, slots: &mut Vec<VoxMapEntry>) -> Result<()> {
    if ext.is::<()>() {
        return Ok(());
    }

    for &format in ReadFormat::ALL {
        if format.with(EncodeSlots { ext, slots })? {
            return Ok(());
        }
    }

    Err(Error::Ext(
        "the ext is not one voxconv writes to Voxel Json".to_owned(),
    ))
}

/// One format's attempt to encode an ext.
struct EncodeSlots<'a> {
    ext: &'a dyn VoxconvExt,
    slots: &'a mut Vec<VoxMapEntry>,
}

impl ReadFormatVisitor for EncodeSlots<'_> {
    type Output = Result<bool>;

    fn visit<F: InstalledFormat>(self) -> Self::Output {
        F::encode_slots(self.ext, self.slots)
    }
}
