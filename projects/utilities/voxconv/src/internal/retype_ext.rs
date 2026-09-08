use crate::Result;
use voxcore::{
    VoxMain,
    ext::{VoxExt, VoxExtEntryCodec, decode_entry},
};

/// Moves a boxed ext into the format's ext type for its writer. A box holding
/// the format's ext is taken as it is. Any other encodes to its block. The
/// format's entry in it decodes to `Some`. A block with no entry for the
/// format, such as another format's, gives `None`.
pub fn retype_ext<E: VoxExt + VoxExtEntryCodec + Clone>(
    state: VoxMain<Box<dyn VoxExt>>,
) -> Result<VoxMain<Option<E>>> {
    if let Some(ext) = state.ext().as_any().downcast_ref::<E>() {
        let ext = ext.clone();

        return Ok(state.map_ext(|_| Some(ext)));
    }

    let ext = decode_entry(&state.ext().to_vox_ext()?)?;

    Ok(state.map_ext(|_| ext))
}
