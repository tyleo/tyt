use crate::ext::{VoxconvExt, VoxconvVoxMain};
use voxcore::{TakenExt, VoxMain};

/// Boxes the ext a format's typed loader returns.
pub fn box_ext<E: VoxconvExt>(main: VoxMain<E>) -> VoxconvVoxMain {
    let TakenExt { main, ext } = main.take_ext();

    main.put_ext(Box::new(ext) as Box<dyn VoxconvExt>)
}
