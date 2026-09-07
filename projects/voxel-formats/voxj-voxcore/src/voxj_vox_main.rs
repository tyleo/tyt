use voxcore::{VoxMain, VoxMap};

/// The state whose ext carries the document's `ext` block verbatim as a
/// voxcore value tree. It loads through `Option<VoxMap>`'s
/// [`VoxExtBlockCodec`](voxcore::ext::VoxExtBlockCodec) impl and writes back
/// through [`VoxExt`](voxcore::ext::VoxExt).
pub type VoxjVoxMain = VoxMain<Option<VoxMap>>;
