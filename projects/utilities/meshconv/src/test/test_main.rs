use branded_id::U32Id;
use meshdoc::{
    MeshHierarchyNode, MeshImage, MeshImageMediaType, MeshImageSource, MeshMain, MeshMaterial,
    MeshObject, MeshPrimitive, MeshTexture, MeshTextureRef, MeshTriangle,
};
use ty_math::{TyVector2F64, TyVector3F64};

/// A state carrying `ext`, with one textured material and one object drawing
/// it. The object sits under a root node so every format's writer emits it.
pub fn test_main<T>(ext: T) -> MeshMain<T> {
    let mut main = MeshMain::default();

    let image_id = main
        .retain_image(MeshImage {
            name: "atlas".to_owned(),
            media_type: MeshImageMediaType::Png,
            source: MeshImageSource::Bytes(vec![
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 1, 2, 3,
            ]),
        })
        .unwrap();

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

    let mut primitive = MeshPrimitive::new(
        vec![TyVector3F64::ZERO, TyVector3F64::X, TyVector3F64::Y],
        vec![MeshTriangle {
            vertex_ids: [U32Id::from_u32(0), U32Id::from_u32(1), U32Id::from_u32(2)],
        }],
    )
    .unwrap();

    primitive
        .push_uv_stream(vec![TyVector2F64::ZERO, TyVector2F64::X, TyVector2F64::Y])
        .unwrap();

    primitive.set_material_id(Some(material_id));

    let mut object = MeshObject::new("body".to_owned());

    object.retain_primitive(primitive);

    let object_id = main.retain_object(object).unwrap();

    let node_id = main
        .retain_hierarchy_node(MeshHierarchyNode {
            child_object_ids: vec![object_id],
            ..Default::default()
        })
        .unwrap();

    main.set_root_hierarchy_node_ids(vec![node_id]).unwrap();

    main.put_ext(ext)
}
