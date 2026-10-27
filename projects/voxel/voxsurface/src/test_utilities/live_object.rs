use ty_math::TyVector3U32;
use voxcore::VoxObject;

/// An object of `bounds` with the `live` cells filled and no layers.
pub fn live_object(bounds: [u32; 3], live: &[[u32; 3]]) -> VoxObject {
    let mut object = VoxObject::new("o".to_owned(), TyVector3U32::from_array(bounds)).unwrap();

    for &[x, y, z] in live {
        let voxel_id = object.voxel_id(TyVector3U32::new(x, y, z)).unwrap();
        object.retain_voxel(voxel_id, &[]).unwrap();
    }

    object
}
