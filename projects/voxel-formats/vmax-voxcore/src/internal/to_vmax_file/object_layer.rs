use crate::{Error, Result};
use branded_id::U32Id;
use voxcore::{BVoxLayer, BVoxPalette, VoxObject};

/// The one layer `object` samples and the palette it draws, or `None` for an
/// object with no layer, which writes colorless. A Voxel Max object reads one
/// palette and this writer converts nothing, so a second layer errors.
pub(crate) fn object_layer(
    object: &VoxObject,
) -> Result<Option<(U32Id<BVoxLayer>, U32Id<BVoxPalette>)>> {
    let mut layers = object.iter_layers();
    match (layers.next(), layers.next()) {
        (layer, None) => Ok(layer),
        _ => Err(Error::invalid(format!(
            "object \"{}\" has {} layers but a Voxel Max object reads one palette",
            object.name(),
            object.layer_count()
        ))),
    }
}
