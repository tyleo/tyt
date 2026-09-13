use crate::{
    BMeshMaterial, BMeshObject, MeshHierarchyNode, MeshMain, MeshMaterial, MeshObject, MeshTexture,
    MeshTextureRef, png_image, unit_triangle,
};
use branded_id::U32Id;
use ty_math::TyVector2F64;

/// A main carrying `ext`: one image, one texture over it, one material
/// sampling the texture as its base color through UV stream `0`, and one
/// object of one triangle with that stream drawing the material, placed by
/// one root node. Returns the main with its object and material ids.
pub fn test_main<T>(ext: T) -> (MeshMain<T>, U32Id<BMeshObject>, U32Id<BMeshMaterial>) {
    let mut main = MeshMain::default();

    let image_id = main.retain_image(png_image()).unwrap();

    let texture_id = main.retain_texture(MeshTexture::new(image_id)).unwrap();

    let material_id = main
        .retain_material(MeshMaterial {
            base_color_texture: Some(MeshTextureRef {
                texture_id,
                uv_stream_id: U32Id::from_u32(0),
            }),
            ..MeshMaterial::new("skin".to_owned())
        })
        .unwrap();

    let mut primitive = unit_triangle();

    primitive
        .push_uv_stream(vec![TyVector2F64::ZERO; 3])
        .unwrap();

    primitive.set_material_id(Some(material_id));

    let mut object = MeshObject::new("body".to_owned());

    object.retain_primitive(primitive);

    let object_id = main.retain_object(object).unwrap();

    let node_id = main
        .retain_hierarchy_node(MeshHierarchyNode {
            name: "root".to_owned(),
            child_object_ids: vec![object_id],
            ..Default::default()
        })
        .unwrap();

    main.set_root_hierarchy_node_ids(vec![node_id]).unwrap();

    (main.put_ext(ext), object_id, material_id)
}
