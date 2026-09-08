//! The typed path: a state carrying its format's ext as a boxed
//! [`VoxExt`](voxcore::ext::VoxExt), so a same-format write rebuilds the
//! file exactly and a Voxel Json write keeps the ext in its `ext` block.
//!
//! [`read_with_ext`] boxes the ext the format's bridge loads.
//! [`write_with_ext`] downcasts the box to the format's ext. Failing that, it
//! takes the format's entry from the block the box encodes to. An ext one
//! format loaded rides through a Voxel Json document and back. A box with no
//! entry for the format writes the synthesized file the bare pair writes.
//! [`load_with_ext`] and [`save_with_ext`] start and end at a path. A Voxel
//! Json document's block decodes into voxcore's
//! [`CompositeVoxExt`](voxcore::ext::CompositeVoxExt), with each enabled
//! format's entry as that format's ext and the rest verbatim.
//!
//! The `ext` feature, on by default, opens this module and enables every
//! enabled bridge's `ext`.

mod load_with_ext;
mod read_with_ext;
mod save_with_ext;
mod write_with_ext;

pub use load_with_ext::*;
pub use read_with_ext::*;
pub use save_with_ext::*;
pub use write_with_ext::*;
