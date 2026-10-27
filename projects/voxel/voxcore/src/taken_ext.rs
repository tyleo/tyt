use crate::VoxMain;

/// A state split from its ext by [`take_ext`](VoxMain::take_ext).
#[derive(Debug)]
pub struct TakenExt<T> {
    /// The scene as a bare main.
    pub main: VoxMain<()>,

    /// The ext the state carried.
    pub ext: T,
}
