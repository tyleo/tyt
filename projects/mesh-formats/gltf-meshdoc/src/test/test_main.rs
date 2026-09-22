use branded_id::U32Id;
use meshdoc::{
    MeshAlphaMode, MeshAttributeComponents, MeshFile, MeshHierarchyNode, MeshImage,
    MeshImageMediaType, MeshImageSource, MeshMagFilter, MeshMain, MeshMaterial, MeshMinFilter,
    MeshObject, MeshPrimitive, MeshProperty, MeshPropertyValue, MeshTexture, MeshTextureRef,
    MeshTriangle, MeshVertexAttribute, MeshWrap,
};
use ty_math::{
    TyLinSrgbF64, TyLinSrgbaF64, TyQuaternionF64, TyTransformF64, TyVector2F64, TyVector3F64,
    TyVector4F64,
};

/// The 8-byte PNG signature.
pub const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// A bare main exercising every modeled entity, with one image over its own
/// bytes and one over a file, every material factor away from its default,
/// and properties of every kind.
pub fn test_main() -> MeshMain<()> {
    let mut main = MeshMain::default();

    // The files, properties, and further attributes sit in name order, the
    // order glTF gives them back in.
    let detail_file_id = main
        .retain_file(MeshFile {
            name: "textures/detail.jpg".to_owned(),
            bytes: vec![0xFF, 0xD8, 0xFF, 0xE0, 9, 9],
        })
        .unwrap();

    let values_id = main
        .retain_file(MeshFile {
            name: "values.json".to_owned(),
            bytes: br#"{"albedo":[[1,0,0,1]]}"#.to_vec(),
        })
        .unwrap();

    let png_id = main
        .retain_image(MeshImage {
            name: "skin.png".to_owned(),
            media_type: MeshImageMediaType::Png,
            source: MeshImageSource::Bytes([PNG_MAGIC.as_slice(), &[1, 2, 3]].concat()),
        })
        .unwrap();

    let jpeg_id = main
        .retain_image(MeshImage {
            name: String::new(),
            media_type: MeshImageMediaType::Jpeg,
            source: MeshImageSource::File(detail_file_id),
        })
        .unwrap();

    let skin_id = main
        .retain_texture(MeshTexture {
            image_id: png_id,
            mag_filter: Some(MeshMagFilter::Nearest),
            min_filter: Some(MeshMinFilter::LinearMipmapLinear),
            wrap_s: MeshWrap::ClampToEdge,
            wrap_t: MeshWrap::MirroredRepeat,
        })
        .unwrap();

    let detail_id = main.retain_texture(MeshTexture::new(jpeg_id)).unwrap();

    let texture_ref = |texture_id, stream| MeshTextureRef {
        texture_id,
        uv_stream_id: U32Id::from_u32(stream),
    };

    let textured_id = main
        .retain_material(MeshMaterial {
            name: "textured".to_owned(),
            base_color_factor: TyLinSrgbaF64::new(0.5, 0.25, 0.125, 0.75),
            base_color_texture: Some(texture_ref(skin_id, 0)),
            metallic_factor: 0.25,
            roughness_factor: 0.5,
            metallic_roughness_texture: Some(texture_ref(detail_id, 1)),
            normal_texture: Some(texture_ref(detail_id, 0)),
            normal_scale: 0.5,
            occlusion_texture: Some(texture_ref(skin_id, 1)),
            occlusion_strength: 0.25,
            emissive_factor: TyLinSrgbF64::new(1.0, 0.5, 0.0),
            emissive_texture: Some(texture_ref(skin_id, 0)),
            emissive_strength: 4.0,
            alpha_mode: MeshAlphaMode::Mask,
            alpha_cutoff: 0.25,
            double_sided: true,
            ior: 1.25,
            transmission_factor: 0.5,
            transmission_texture: Some(texture_ref(detail_id, 1)),
            properties: vec![
                MeshProperty {
                    name: "accent".to_owned(),
                    value: MeshPropertyValue::Floats(vec![0.5, 0.25, 0.125]),
                },
                MeshProperty {
                    name: "flags".to_owned(),
                    value: MeshPropertyValue::Bools(vec![true, false]),
                },
                MeshProperty {
                    name: "flagRows".to_owned(),
                    value: MeshPropertyValue::BoolRows(vec![vec![true, false], vec![false, true]]),
                },
                MeshProperty {
                    name: "group".to_owned(),
                    value: MeshPropertyValue::Int(3),
                },
                MeshProperty {
                    name: "groups".to_owned(),
                    value: MeshPropertyValue::Ints(vec![1, 2]),
                },
                MeshProperty {
                    name: "groupRows".to_owned(),
                    value: MeshPropertyValue::IntRows(vec![vec![1, 2], vec![3, 4]]),
                },
                MeshProperty {
                    name: "heat".to_owned(),
                    value: MeshPropertyValue::Texture(texture_ref(detail_id, 1)),
                },
                MeshProperty {
                    name: "label".to_owned(),
                    value: MeshPropertyValue::Text("shell".to_owned()),
                },
                MeshProperty {
                    name: "labels".to_owned(),
                    value: MeshPropertyValue::Texts(vec!["a".to_owned(), "b".to_owned()]),
                },
                MeshProperty {
                    name: "labelRows".to_owned(),
                    value: MeshPropertyValue::TextRows(vec![
                        vec!["a".to_owned(), "b".to_owned()],
                        vec!["c".to_owned()],
                    ]),
                },
                MeshProperty {
                    name: "subsurface".to_owned(),
                    value: MeshPropertyValue::Float(0.5),
                },
                MeshProperty {
                    name: "wear".to_owned(),
                    value: MeshPropertyValue::File(values_id),
                },
            ],
        })
        .unwrap();

    let plain_id = main
        .retain_material(MeshMaterial::new("plain".to_owned()))
        .unwrap();

    let triangle = |a, b, c| MeshTriangle {
        vertex_ids: [U32Id::from_u32(a), U32Id::from_u32(b), U32Id::from_u32(c)],
    };

    let mut textured = MeshPrimitive::new(
        vec![
            TyVector3F64::new(0.0, 0.0, 0.0),
            TyVector3F64::new(1.0, 0.0, 0.0),
            TyVector3F64::new(1.0, 2.0, 0.0),
            TyVector3F64::new(0.0, 2.0, 3.0),
        ],
        vec![triangle(0, 1, 2), triangle(0, 2, 3)],
    )
    .unwrap();

    textured
        .set_normals(Some(vec![TyVector3F64::Z; 4]))
        .unwrap();

    textured
        .set_tangents(Some(vec![TyVector4F64::new(1.0, 0.0, 0.0, -1.0); 4]))
        .unwrap();

    textured
        .push_uv_stream(vec![
            TyVector2F64::new(0.0, 0.0),
            TyVector2F64::new(1.0, 0.0),
            TyVector2F64::new(1.0, 1.0),
            TyVector2F64::new(0.0, 1.0),
        ])
        .unwrap();

    textured
        .push_uv_stream(vec![TyVector2F64::new(0.5, 0.5); 4])
        .unwrap();

    textured
        .set_colors(Some(vec![
            TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0),
            TyLinSrgbaF64::new(0.0, 1.0, 0.0, 1.0),
            TyLinSrgbaF64::new(0.0, 0.0, 1.0, 1.0),
            TyLinSrgbaF64::new(1.0, 1.0, 1.0, 0.5),
        ]))
        .unwrap();

    textured
        .push_vertex_attribute(MeshVertexAttribute {
            name: "_CELL".to_owned(),
            width: 2,
            components: MeshAttributeComponents::U16(vec![0, 1, 2, 3, 4, 5, 6, 7]),
        })
        .unwrap();

    textured
        .push_vertex_attribute(MeshVertexAttribute {
            name: "_PALETTE".to_owned(),
            width: 1,
            components: MeshAttributeComponents::U8(vec![0, 1, 1, 2]),
        })
        .unwrap();

    textured
        .push_vertex_attribute(MeshVertexAttribute {
            name: "_TEMPERATURE".to_owned(),
            width: 2,
            components: MeshAttributeComponents::F64(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]),
        })
        .unwrap();

    textured.set_material_id(Some(textured_id));
    textured.set_name("shell".to_owned());

    let mut plain = MeshPrimitive::new(
        vec![
            TyVector3F64::new(0.0, 0.0, 1.0),
            TyVector3F64::new(1.0, 0.0, 1.0),
            TyVector3F64::new(0.0, 1.0, 1.0),
        ],
        vec![triangle(0, 1, 2)],
    )
    .unwrap();

    plain.set_material_id(Some(plain_id));

    let mut object = MeshObject::new("body".to_owned());

    object.set_properties(vec![
        MeshProperty {
            name: "albedo".to_owned(),
            value: MeshPropertyValue::FloatRows(vec![
                vec![1.0, 0.0, 0.0, 1.0],
                vec![0.0, 0.0, 1.0, 1.0],
            ]),
        },
        MeshProperty {
            name: "key".to_owned(),
            value: MeshPropertyValue::Texture(texture_ref(skin_id, 0)),
        },
        MeshProperty {
            name: "rows".to_owned(),
            value: MeshPropertyValue::File(values_id),
        },
    ]);

    object.retain_primitive(textured);

    object.retain_primitive(plain);

    let object_id = main.retain_object(object).unwrap();

    let node_ids = main
        .retain_hierarchy_nodes(vec![
            MeshHierarchyNode {
                name: "root".to_owned(),
                transform: TyTransformF64 {
                    position: TyVector3F64::new(1.0, 2.0, 3.0),
                    rotation: TyQuaternionF64::from_axis_angle(
                        TyVector3F64::new(0.0, 0.0, 1.0),
                        0.5,
                    ),
                    scale: TyVector3F64::new(1.0, 2.0, 3.0),
                },
                child_node_ids: vec![U32Id::from_u32(1)],
                child_object_ids: Vec::new(),
            },
            MeshHierarchyNode {
                name: "leaf".to_owned(),
                transform: TyTransformF64::default(),
                child_node_ids: Vec::new(),
                child_object_ids: vec![object_id],
            },
        ])
        .unwrap();

    main.set_root_hierarchy_node_ids(vec![node_ids[0]]).unwrap();

    main.validate().unwrap();

    main
}
