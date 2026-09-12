use crate::ext::{VoxconvExt, VoxconvVoxMain};
use voxcore::{TakenExt, VoxMain};

/// Boxes the ext a format's typed loader returns.
pub fn box_ext<E: VoxconvExt>(state: VoxMain<E>) -> VoxconvVoxMain {
    let TakenExt { state, ext } = state.take_ext();

    state.put_ext(Box::new(ext) as Box<dyn VoxconvExt>)
}
