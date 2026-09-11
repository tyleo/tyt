//! The `ext` block as a state's ext. [`VoxjVoxExt`] keeps the block as it
//! was parsed, so a [`VoxjVoxMain`] writes back exactly. The block is not
//! understood: it follows no hook and goes stale under a mutation that moves
//! a listing. [`from_voxj_file_with_ext`] loads it and
//! [`to_voxj_file_with_ext`] writes it back. A crate that knows the block's
//! entries takes it off with [`VoxMain::take_ext`](voxcore::VoxMain::take_ext)
//! and puts on an ext that follows.

mod from_voxj_file_with_ext;
mod to_voxj_file_with_ext;
mod voxj_vox_ext;
mod voxj_vox_main;

pub use from_voxj_file_with_ext::*;
pub use to_voxj_file_with_ext::*;
pub use voxj_vox_ext::*;
pub use voxj_vox_main::*;
