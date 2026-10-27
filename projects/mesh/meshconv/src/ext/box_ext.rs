use crate::ext::{MeshconvExt, MeshconvMeshMain};
use meshdoc::{MeshMain, TakenExt};

/// Boxes the ext a format's typed loader returns.
pub fn box_ext<E: MeshconvExt>(main: MeshMain<E>) -> MeshconvMeshMain {
    let TakenExt { main, ext } = main.take_ext();

    main.put_ext(Box::new(ext) as Box<dyn MeshconvExt>)
}
