use branded_id::U32Id;
use ty_math::TyVector3U32;
use voxrender::{BRenderMaterial, RenderObject};

/// An object of `bounds` named `name` whose cells hold what `paint` gives
/// each position.
pub fn painted_object(
    name: &str,
    bounds: TyVector3U32,
    paint: impl Fn(TyVector3U32) -> Option<U32Id<BRenderMaterial>>,
) -> RenderObject {
    let mut object =
        RenderObject::new(name.to_owned(), bounds).expect("a bench object fits the cell cap");

    for x in 0..bounds.x {
        for y in 0..bounds.y {
            for z in 0..bounds.z {
                let position = TyVector3U32::new(x, y, z);

                if let Some(material_id) = paint(position) {
                    let voxel_id = object.voxel_id(position).expect("the cell is in the grid");

                    object
                        .set_voxel_material(voxel_id, Some(material_id))
                        .expect("the cell is in the grid");
                }
            }
        }
    }

    object
}
