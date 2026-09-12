use crate::{GoxlExtLayer, GoxlExtPlacement};
use branded_id::U32Id;
use goxl::{GoxlLayer, GoxlShape};

/// The ext entry for one layer. A block index becomes the id of the object
/// the loader retained for it. The loader checked every index against the
/// block list.
pub fn layer_provenance(layer: &GoxlLayer) -> GoxlExtLayer {
    GoxlExtLayer {
        id: layer.id,
        base_id: layer.base_id,
        material: layer.material,
        mode: layer.mode,
        visible: layer.visible,
        transform: layer.transform,
        bounding_box: layer.bounding_box,
        image_path: layer.image_path.clone(),
        shape: layer.shape.map(shape_token),
        color: layer.color,
        placements: layer
            .blocks
            .iter()
            .map(|block| GoxlExtPlacement {
                object_id: U32Id::from_u32(
                    u32::try_from(block.block_index)
                        .expect("the loader checked every placement against the block list"),
                ),
                position: block.position,
            })
            .collect(),
        extra: layer.extra.0.clone(),
    }
}

/// The on-disk shape name for a procedural shape.
fn shape_token(shape: GoxlShape) -> String {
    match shape {
        GoxlShape::Sphere => "sphere".to_owned(),
        GoxlShape::Cube => "cube".to_owned(),
        GoxlShape::Cylinder => "cylinder".to_owned(),
    }
}
