use crate::MVoxExtShapeModel;
use branded_id::U32Id;
use voxcore::BVoxObject;

/// The model a synthesized shape draws `object` with.
pub fn synthesized_shape_model(object: U32Id<BVoxObject>) -> MVoxExtShapeModel {
    MVoxExtShapeModel {
        object,
        frame_index: Some(0),
        extra: Vec::new(),
    }
}
