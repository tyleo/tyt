use crate::{BRenderMaterial, RenderObject};
use branded_id::U32Id;
use ty_math::TyVector3U32;

/// An object of `bounds` named `name`, filled with `material_id` at every
/// cell `is_live` accepts.
pub fn solid_object(
    name: &str,
    bounds: TyVector3U32,
    material_id: U32Id<BRenderMaterial>,
    is_live: impl Fn(TyVector3U32) -> bool,
) -> RenderObject {
    let mut object = RenderObject::new(name.to_owned(), bounds).unwrap();

    for x in 0..bounds.x {
        for y in 0..bounds.y {
            for z in 0..bounds.z {
                let position = TyVector3U32::new(x, y, z);

                if is_live(position) {
                    let voxel_id = object.voxel_id(position).unwrap();

                    object
                        .set_voxel_material(voxel_id, Some(material_id))
                        .unwrap();
                }
            }
        }
    }

    object
}
