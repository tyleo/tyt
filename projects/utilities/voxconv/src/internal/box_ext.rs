use voxcore::{VoxMain, ext::VoxExt};

/// Boxes the ext a format's typed loader returns. A loaded file always
/// carries one.
pub fn box_ext<E: VoxExt>(state: VoxMain<Option<E>>) -> VoxMain<Box<dyn VoxExt>> {
    state.map_ext(|ext| Box::new(ext.expect("a loaded file carries its ext")) as Box<dyn VoxExt>)
}
