use branded_id::U32Id;
use voxcore::BVoxObject;
use voxrender::{RenderObject, RenderScene};

/// Adds `object` to `scene` under the id after the scene's last.
pub fn retain_next_object(scene: &mut RenderScene, object: RenderObject) -> U32Id<BVoxObject> {
    let object_id = U32Id::from_u32(
        u32::try_from(scene.object_count()).expect("a bench scene's objects number under 2^32"),
    );

    scene
        .retain_object(object_id, object)
        .expect("a bench scene's objects take the ids in order");

    object_id
}
