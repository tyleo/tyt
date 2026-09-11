use crate::{
    Result,
    ext::{VoxconvExt, push_slots},
};
use voxj_voxcore::ext::VoxjVoxExt;

/// Encodes an ext as a document's `ext` block. `()` encodes an empty ext. An
/// installed format's ext encodes as its slots, which for Voxel Json's exts
/// means an [`InertVoxExt`](crate::voxj::ext::InertVoxExt) as the slot it kept,
/// a [`CompositeVoxExt`](crate::voxj::ext::CompositeVoxExt) as every slot it
/// holds, and a [`VoxjVoxExt`] as it is. Errors on a repeated key and on an ext
/// outside those.
pub fn voxj_vox_ext_from_ext(ext: &dyn VoxconvExt) -> Result<VoxjVoxExt> {
    let mut slots = Vec::new();

    push_slots(ext, &mut slots)?;

    Ok(VoxjVoxExt::new(slots))
}
