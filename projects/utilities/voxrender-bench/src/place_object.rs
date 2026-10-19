use branded_id::U32Id;
use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64};
use voxcore::BVoxObject;
use voxrender::{RenderPlacement, RenderScene};

/// Places `object_id` in `scene` with its grid's origin at `position`,
/// turned by `rotation`, at `scale` meters per voxel.
pub fn place_object(
    scene: &mut RenderScene,
    object_id: U32Id<BVoxObject>,
    position: TyVector3F64,
    rotation: TyQuaternionF64,
    scale: f64,
) {
    scene
        .retain_placement(RenderPlacement {
            object_id,
            transform: TyTransformF64::new(position, rotation, TyVector3F64::splat(scale)),
        })
        .expect("the object is the scene's");
}
