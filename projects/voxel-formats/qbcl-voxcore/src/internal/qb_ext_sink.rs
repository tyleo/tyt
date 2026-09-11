use crate::ext::{QbExt, QbExtMatrix};
use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbZAxisOrientation};
use voxcore::{VoxExt, VoxMain, VoxObject};

/// Puts the ext a read yields on the bare state. [`QbExt`] keeps it. `()`
/// drops it.
pub trait QbExtSink: VoxExt + Sized {
    /// Puts on `state` the ext of a read of `file`. The objects
    /// pair with the file's matrices in listing order.
    fn record_file(state: VoxMain<()>, file: &QbFile) -> VoxMain<Self>;
}

impl QbExtSink for QbExt {
    fn record_file(state: VoxMain<()>, file: &QbFile) -> VoxMain<Self> {
        let matrices = state
            .iter_objects()
            .zip(&file.matrices)
            .map(|((_, object), matrix)| {
                Some(QbExtMatrix {
                    name: matrix.name.clone(),
                    position: matrix.position,
                    visibility: visibility_of(object, matrix),
                })
            })
            .collect();

        state.put_ext(QbExt {
            version: file.version,
            bgra: matches!(file.color_format, QbColorFormat::Bgra),
            right_handed: matches!(file.z_axis_orientation, QbZAxisOrientation::RightHanded),
            compressed: file.compressed,
            visibility_mask_encoded: file.visibility_mask_encoded,
            matrices,
        })
    }
}

impl QbExtSink for () {
    fn record_file(state: VoxMain<()>, _file: &QbFile) -> VoxMain<Self> {
        state
    }
}

/// The visibility bytes of an object's solid voxels, in live-voxel raster
/// order, read back from the matrix the object was built from.
fn visibility_of(object: &VoxObject, matrix: &QbMatrix) -> Vec<u8> {
    object
        .iter_live()
        .map(|voxel_id| {
            let position = object
                .voxel_position(voxel_id)
                .expect("a live voxel is within the grid");
            matrix
                .voxel(position.x, position.y, position.z)
                .map_or(0, |voxel| voxel.visibility)
        })
        .collect()
}
