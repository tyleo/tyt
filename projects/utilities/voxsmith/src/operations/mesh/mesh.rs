use crate::{
    Error, Result,
    dependencies::mesh::EncodePng,
    operations::mesh::{
        MaterialMeshRequest, MaterialSlot, MeshGeometry, atlas_dimensions, bake_atlas_image,
        check_material_maps, material_scalar, max_emissive_strength, mesh_slices,
        object_to_mesh_geometry, resolve_used_materials, texel_center,
    },
    utilities::check_material_property_ranges,
};
use branded_id::U32Id;
use meshdoc::{
    MeshHierarchyNode, MeshImage, MeshImageMediaType, MeshImageSource, MeshMagFilter, MeshMain,
    MeshMaterial, MeshMinFilter, MeshObject, MeshPrimitive, MeshProperty, MeshPropertyValue,
    MeshTexture, MeshTextureRef, MeshTriangle, MeshWrap,
    material::{EMISSIVE_STRENGTH_DEFAULT, IOR, TRANSMISSION},
};
use ty_math::{TyLinSrgbF64, TyVector2F64, TyVector3F64};
use voxcore::{VoxExt, VoxMain, VoxObject};

/// Meshes `object` into a mesh document of one object placed by one root
/// node, both named as `object` is. With no maps in `request` the object is
/// pure geometry: one primitive with positions and normals, no material and
/// no images. Otherwise the object's flattened layer materials bake into the
/// requested maps, each an image the primitive's one UV stream samples, and
/// one material references them through their slots. The material's flat
/// factors take `ior` and `transmission` from the first used material and
/// `emissiveStrength` from the mesh's greatest. `dependencies` encodes the
/// atlas images as PNG. Positions are in meters, `request.scale` per voxel,
/// on the grid's Z-up axes. Errors if a layer references a palette `main`
/// does not hold, if a vocabulary property carries a value outside its
/// range, or if a map reads a property as a kind its layer does not bind.
pub fn mesh<D: EncodePng, T: VoxExt>(
    dependencies: &D,
    main: &VoxMain<T>,
    object: &VoxObject,
    request: &MaterialMeshRequest,
) -> Result<MeshMain<()>> {
    let mut document = MeshMain::default();

    if request.maps.is_empty() {
        let geometry = object_to_mesh_geometry(object, request.method);

        let primitive = primitive_of(&geometry, request.scale, None)?;

        return place(document, object.name(), primitive);
    }

    // Nothing out of range reaches a mesh material: the vocabulary range
    // check gates the export before anything is built.
    check_material_property_ranges(main)?;

    // Every map reads its properties as the kinds the object's layers bind
    // them to, checked once here before the bake reads texel by texel.
    check_material_maps(main, object, &request.maps)?;

    let used = resolve_used_materials(main, object)?;

    let geometry = mesh_slices(
        object,
        request.method,
        &|voxel_id| {
            used.material_index(voxel_id)
                .expect("the sweep keys only the live voxels the used set indexed")
        },
        true,
    );

    let (atlas_width, atlas_height) = atlas_dimensions(used.len(), request.shape)?;

    let mut material = MeshMaterial::default();

    // Bake each map into an image and a texture, and wire it into its slot.
    for map in &request.maps {
        let image = bake_atlas_image(&used, &map.bake, atlas_width, atlas_height)?;

        let png = dependencies.encode_png(&image).map_err(Error::Png)?;

        let image_id = document.retain_image(MeshImage {
            name: map.name.clone(),
            media_type: MeshImageMediaType::Png,
            source: MeshImageSource::Bytes(png),
        })?;

        // Nearest filtering and clamped wraps, so a face reads exactly its
        // texel.
        let texture_id = document.retain_texture(MeshTexture {
            image_id,
            mag_filter: Some(MeshMagFilter::Nearest),
            min_filter: Some(MeshMinFilter::Nearest),
            wrap_s: MeshWrap::ClampToEdge,
            wrap_t: MeshWrap::ClampToEdge,
        })?;

        let texture_ref = MeshTextureRef {
            texture_id,
            uv_stream_id: U32Id::from_u32(0),
        };

        match map.slot {
            MaterialSlot::BaseColor => material.base_color_texture = Some(texture_ref),

            MaterialSlot::MetallicRoughness => {
                material.metallic_roughness_texture = Some(texture_ref)
            }

            MaterialSlot::Occlusion => material.occlusion_texture = Some(texture_ref),

            MaterialSlot::OcclusionMetallicRoughness => {
                material.occlusion_texture = Some(texture_ref);
                material.metallic_roughness_texture = Some(texture_ref);
            }

            MaterialSlot::Emissive => {
                material.emissive_texture = Some(texture_ref);
                material.emissive_factor = TyLinSrgbF64::new(1.0, 1.0, 1.0);
            }

            MaterialSlot::None => material.properties.push(MeshProperty {
                name: map.name.clone(),
                value: MeshPropertyValue::Texture(texture_ref),
            }),
        }
    }

    // The emissive strength is the one the emissive texels were normalized
    // by, or the default when the mesh emits nothing.
    if used.len() > 0 {
        material.ior = material_scalar(&used, 0, IOR)?;
        material.transmission_factor = material_scalar(&used, 0, TRANSMISSION)?;
    }

    let max_strength = max_emissive_strength(&used)?;
    material.emissive_strength = if max_strength > 0.0 {
        max_strength
    } else {
        EMISSIVE_STRENGTH_DEFAULT
    };

    let material_id = document.retain_material(material)?;

    let mut primitive = primitive_of(&geometry, request.scale, Some((atlas_width, atlas_height)))?;

    primitive.set_material_id(Some(material_id));

    place(document, object.name(), primitive)
}

/// The primitive of `geometry` scaled to `scale` meters per voxel, with its
/// normals and, given the atlas dimensions, one UV stream sampling each
/// vertex's material texel.
fn primitive_of(
    geometry: &MeshGeometry,
    scale: f64,
    atlas: Option<(u32, u32)>,
) -> Result<MeshPrimitive> {
    let positions = geometry
        .positions
        .iter()
        .map(|position| TyVector3F64::from(position.as_dvec3() * scale))
        .collect();

    let triangles = geometry
        .indices
        .chunks_exact(3)
        .map(|corners| MeshTriangle {
            vertex_ids: [
                U32Id::from_u32(corners[0]),
                U32Id::from_u32(corners[1]),
                U32Id::from_u32(corners[2]),
            ],
        })
        .collect();

    let mut primitive = MeshPrimitive::new(positions, triangles)?;

    primitive.set_normals(Some(
        geometry
            .normals
            .iter()
            .map(|normal| normal.as_dvec3())
            .collect(),
    ))?;

    if let Some((width, height)) = atlas {
        primitive.push_uv_stream(
            geometry
                .material_indices
                .iter()
                .map(|&index| {
                    let [u, v] = texel_center(index, width, height);
                    TyVector2F64::new(f64::from(u), f64::from(v))
                })
                .collect(),
        )?;
    }

    Ok(primitive)
}

/// `document` with `primitive` in one object named `name`, placed by one
/// root node of the same name.
fn place(mut document: MeshMain<()>, name: &str, primitive: MeshPrimitive) -> Result<MeshMain<()>> {
    let mut object = MeshObject::new(name.to_owned());

    object.retain_primitive(primitive);

    let object_id = document.retain_object(object)?;

    let node_id = document.retain_hierarchy_node(MeshHierarchyNode {
        name: name.to_owned(),
        child_object_ids: vec![object_id],
        ..Default::default()
    })?;

    document.set_root_hierarchy_node_ids(vec![node_id])?;

    Ok(document)
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        dependencies::DependenciesImpl,
        operations::mesh::{
            AtlasShape, MaterialBake, MaterialChannel, MaterialMap, MaterialMeshRequest,
            MaterialSlot, MeshMethod, mesh,
        },
    };
    use branded_id::U32Id;
    use meshdoc::{
        BMeshObject, MeshImageMediaType, MeshMain, MeshPropertyValue,
        material::{EMISSIVE_STRENGTH, IOR, TRANSMISSION},
    };
    use ty_math::{TyVector3F64, TyVector3U32};
    use voxcore::{
        VoxMain, VoxObject, VoxPalette, VoxValuePool,
        material::{BASE_COLOR, METALLIC},
    };

    /// The 8-byte PNG signature.
    const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    /// A 2x1x1 bar whose two voxels are red and blue through one `baseColor`
    /// palette, with `ior` and `transmission` bound too.
    fn red_blue_bar() -> (VoxMain, U32Id<voxcore::BVoxObject>) {
        let mut main: VoxMain = VoxMain::default();

        let base_value_pool_id = main.retain_value_pool(
            VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]]).unwrap(),
        );
        let ior_value_pool_id = main.retain_value_pool(VoxValuePool::float(vec![1.4]).unwrap());
        let transmission_value_pool_id =
            main.retain_value_pool(VoxValuePool::float(vec![0.5]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .retain_property(
                BASE_COLOR.to_owned(),
                base_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette
            .retain_property(IOR.to_owned(), ior_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette
            .retain_property(
                TRANSMISSION.to_owned(),
                transmission_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        let red_id = palette
            .retain_material(vec![
                U32Id::from_u32(0),
                U32Id::from_u32(0),
                U32Id::from_u32(0),
            ])
            .unwrap();
        let blue_id = palette
            .retain_material(vec![
                U32Id::from_u32(1),
                U32Id::from_u32(0),
                U32Id::from_u32(0),
            ])
            .unwrap();
        let palette_id = main.retain_palette(palette).unwrap();

        let mut object = VoxObject::new("bar".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        object.retain_layer(palette_id, red_id);
        for (x, material_id) in [(0, red_id), (1, blue_id)] {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel_id, &[material_id]).unwrap();
        }
        let object_id = main.retain_object(object).unwrap();

        (main, object_id)
    }

    /// A request baking `maps` under the fit atlas at `scale`.
    fn request(maps: Vec<MaterialMap>, scale: f64) -> MaterialMeshRequest {
        MaterialMeshRequest {
            method: MeshMethod::Greedy,
            scale,
            maps,
            shape: AtlasShape::Fit,
        }
    }

    /// The one object of `document`.
    fn the_object(document: &MeshMain<()>) -> (U32Id<BMeshObject>, &meshdoc::MeshObject) {
        document.iter_objects().next().unwrap()
    }

    #[test]
    fn pure_geometry_is_one_unmaterialed_primitive_under_one_root() {
        let (main, object_id) = red_blue_bar();

        let document = mesh(
            &DependenciesImpl,
            &main,
            main.object(object_id).unwrap(),
            &request(Vec::new(), 2.0),
        )
        .unwrap();
        document.validate().unwrap();

        assert_eq!(document.image_count(), 0);
        assert_eq!(document.material_count(), 0);
        assert_eq!(document.root_hierarchy_node_ids().len(), 1);

        let (_, object) = the_object(&document);
        assert_eq!(object.name(), "bar");
        let (_, primitive) = object.iter_primitives().next().unwrap();
        assert_eq!(primitive.material_id(), None);
        assert_eq!(primitive.uv_stream_count(), 0);
        assert!(primitive.normals().is_some());

        // Greedy on a same-key bar: six faces, four vertices and two
        // triangles each, scaled to two meters per voxel and kept Z-up.
        assert_eq!(primitive.vertex_count(), 24);
        assert_eq!(primitive.triangle_count(), 12);
        let max = primitive
            .positions()
            .iter()
            .fold(TyVector3F64::NEG_INFINITY, |max, position| {
                max.max(*position)
            });
        assert_eq!(max, TyVector3F64::new(4.0, 2.0, 2.0));
    }

    #[test]
    fn maps_bake_into_images_textures_and_material_slots() {
        let (main, object_id) = red_blue_bar();

        let maps = vec![
            MaterialMap {
                name: "bar-albedo.png".to_owned(),
                slot: MaterialSlot::BaseColor,
                bake: MaterialBake::RgbaColor,
            },
            MaterialMap {
                name: "bar-mse.png".to_owned(),
                slot: MaterialSlot::None,
                bake: MaterialBake::Packing(vec![MaterialChannel::Property {
                    key: METALLIC.to_owned(),
                    component: None,
                    invert: false,
                }]),
            },
        ];

        let document = mesh(
            &DependenciesImpl,
            &main,
            main.object(object_id).unwrap(),
            &request(maps, 1.0),
        )
        .unwrap();
        document.validate().unwrap();

        // One image and texture per map, each a PNG named as the map.
        assert_eq!(document.image_count(), 2);
        assert_eq!(document.texture_count(), 2);
        let images: Vec<_> = document.iter_images().collect();
        assert_eq!(images[0].1.name, "bar-albedo.png");
        assert_eq!(images[0].1.media_type, MeshImageMediaType::Png);
        assert!(
            document
                .image_bytes(images[0].0)
                .unwrap()
                .starts_with(&PNG_MAGIC)
        );

        // The base color takes its slot; the slotless map is a texture
        // property named after it; the flat factors ride on the material.
        assert_eq!(document.material_count(), 1);
        let (material_id, material) = document.iter_materials().next().unwrap();
        assert!(material.base_color_texture.is_some());
        assert!(matches!(
            material
                .property("bar-mse.png")
                .map(|property| &property.value),
            Some(MeshPropertyValue::Texture(_))
        ));
        assert_eq!(material.scalar(IOR), Some(1.4));
        assert_eq!(material.scalar(TRANSMISSION), Some(0.5));
        assert_eq!(material.scalar(EMISSIVE_STRENGTH), Some(1.0));

        // The primitive draws the material through one UV stream, one
        // coordinate per vertex.
        let (_, object) = the_object(&document);
        let (_, primitive) = object.iter_primitives().next().unwrap();
        assert_eq!(primitive.material_id(), Some(material_id));
        assert_eq!(primitive.uv_stream_count(), 1);
        assert_eq!(
            primitive.uv_stream(U32Id::from_u32(0)).unwrap().len(),
            primitive.vertex_count()
        );
    }

    #[test]
    fn an_out_of_range_vocabulary_value_fails_the_export() {
        // A metallic outside `[0, 1]` trips the vocabulary range check
        // before anything is built.
        let mut main: VoxMain = VoxMain::default();
        let base_value_pool_id =
            main.retain_value_pool(VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap());
        let metallic_value_pool_id =
            main.retain_value_pool(VoxValuePool::float(vec![1.5]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .retain_property(
                BASE_COLOR.to_owned(),
                base_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette
            .retain_property(
                METALLIC.to_owned(),
                metallic_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        let material_id = palette
            .retain_material(vec![U32Id::from_u32(0), U32Id::from_u32(0)])
            .unwrap();
        let palette_id = main.retain_palette(palette).unwrap();

        let mut object = VoxObject::new("bar".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.retain_layer(palette_id, material_id);
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel_id, &[material_id]).unwrap();
        let object_id = main.retain_object(object).unwrap();

        let maps = vec![MaterialMap {
            name: "bar-albedo.png".to_owned(),
            slot: MaterialSlot::BaseColor,
            bake: MaterialBake::RgbaColor,
        }];

        let message = mesh(
            &DependenciesImpl,
            &main,
            main.object(object_id).unwrap(),
            &request(maps, 1.0),
        )
        .expect_err("the export rejects an out-of-range metallic")
        .to_string();
        assert!(message.contains(METALLIC), "{message}");
        assert!(message.contains("1.5"), "{message}");
    }

    #[test]
    fn an_empty_object_meshes_to_an_object_with_no_primitive() {
        let main: VoxMain = VoxMain::default();
        let empty = VoxObject::new("empty".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();

        let document = mesh(&DependenciesImpl, &main, &empty, &request(Vec::new(), 1.0)).unwrap();

        let (_, object) = the_object(&document);
        assert_eq!(object.name(), "empty");
        let (_, primitive) = object.iter_primitives().next().unwrap();
        assert_eq!(primitive.vertex_count(), 0);
    }
}
