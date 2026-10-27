use crate::MeshMain;

/// A state split from its ext by [`take_ext`](MeshMain::take_ext).
#[derive(Debug)]
pub struct TakenExt<T> {
    /// The document as a bare main.
    pub main: MeshMain<()>,

    /// The ext the state carried.
    pub ext: T,
}
