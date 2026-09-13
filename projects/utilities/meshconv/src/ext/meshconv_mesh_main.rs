use crate::ext::MeshconvExt;
use meshdoc::MeshMain;

/// The state the typed pairs exchange: a [`MeshMain`] carrying whichever
/// format's ext its document held, boxed.
/// [`read_with_ext`](crate::ext::read_with_ext) loads one and
/// [`write_with_ext`](crate::ext::write_with_ext) writes it.
pub type MeshconvMeshMain = MeshMain<Box<dyn MeshconvExt>>;
