use voxcore::{VoxMain, VoxMap};

/// The state whose ext slot carries the document's `ext` block verbatim as a
/// voxcore value tree, through [`VoxMap`]'s identity
/// [`VoxExtCodec`](voxcore::ext::VoxExtCodec) impl.
pub type VoxjVoxMain = VoxMain<Option<VoxMap>>;
