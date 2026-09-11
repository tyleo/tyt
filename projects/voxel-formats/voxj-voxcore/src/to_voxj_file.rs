use crate::{Result, VoxjWriteOptions, write_voxj};
use voxcore::VoxMain;
use voxj::{CostVoxjObject, EncodeBase64, VoxjFile};

/// Encodes a bare [`VoxMain`] into a [`VoxjFile`], the inverse of
/// [`from_voxj_file`](crate::from_voxj_file). The document gets no `ext`
/// block. [`ext::to_voxj_file_with_ext`](crate::ext::to_voxj_file_with_ext)
/// writes one.
///
/// Value pools, palettes, objects, and hierarchy nodes are emitted in listing
/// order, and ids are written as array indices. Call
/// [`VoxMain::gc`](voxcore::VoxMain::gc) first after any removal or move so
/// each id equals its listing index and the cross references land intact.
///
/// # Arguments
/// 1. `dependencies` - encodes the base64 blocks and costs a searched block's
///    candidates. A pinned pair is never costed.
pub fn to_voxj_file<D: EncodeBase64 + CostVoxjObject>(
    dependencies: &D,
    state: &VoxMain<()>,
    options: &VoxjWriteOptions,
) -> Result<VoxjFile> {
    write_voxj(dependencies, state, options)
}
