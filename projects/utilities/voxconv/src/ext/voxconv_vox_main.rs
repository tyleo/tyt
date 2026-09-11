use crate::ext::VoxconvExt;
use voxcore::VoxMain;

/// The state the typed pairs exchange: a [`VoxMain`] carrying whichever
/// format's ext its document held, boxed.
/// [`read_with_ext`](crate::ext::read_with_ext) loads one and
/// [`write_with_ext`](crate::ext::write_with_ext) writes it.
pub type VoxconvVoxMain = VoxMain<Box<dyn VoxconvExt>>;
