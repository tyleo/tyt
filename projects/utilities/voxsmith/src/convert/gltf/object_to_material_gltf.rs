use crate::{Error, MaterialMeshRequest, MeshFiles, MeshTarget, Result, build_material_document};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::Value;
use voxcore::{VoxMain, VoxObject};

/// Meshes `object` material-aware and writes it as a text glTF (`.gltf`) with a
/// material sampling the baked palette atlas, plus any loose image files
/// `request.storage` asks for. The geometry buffer embeds as a base64 data URI,
/// and embedded images embed as their own data URIs, so the document is a single
/// self-contained file. An object with no live voxels writes a valid glTF with
/// an empty scene.
pub fn object_to_material_gltf(
    state: &VoxMain,
    object: &VoxObject,
    request: &MaterialMeshRequest,
) -> Result<MeshFiles> {
    let mut built = build_material_document(state, object, request, MeshTarget::Gltf)?;

    if !built.blob.is_empty() {
        let uri = format!(
            "data:application/octet-stream;base64,{}",
            STANDARD.encode(&built.blob)
        );

        built.document["buffers"][0]["uri"] = Value::String(uri);
    }

    let mesh = serde_json::to_vec(&built.document).map_err(Error::invalid)?;

    Ok(MeshFiles {
        mesh,
        sidecars: built.sidecars,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        AtlasShape, BASE_COLOR_FACTOR, METALLIC_FACTOR, MaterialBake, MaterialChannel, MaterialMap,
        MaterialMeshRequest, MaterialSlot, MeshMethod, ResourceStorage, object_to_material_gltf,
    };
    use branded_id::U32Id;
    use gltf::Gltf;
    use serde_json::Value;
    use ty_math::TyVector3U32;
    use voxcore::{VoxMain, VoxObject, VoxPalette, VoxValuePool};

    /// A one-voxel red cube through a single `baseColorFactor` palette, and the
    /// request the test bakes; `maps` and `storage` vary per test.
    fn cube_gltf(maps: Vec<MaterialMap>, storage: ResourceStorage) -> Vec<u8> {
        let mut state = VoxMain::default();

        let base = state.add_value_pool(VoxValuePool::srgba(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .add_property(BASE_COLOR_FACTOR.to_owned(), base)
            .unwrap();
        let red = palette.add_material(vec![U32Id::from_u32(0)]).unwrap();
        let palette_id = state.add_palette(palette).unwrap();

        let mut object = VoxObject::new("cube".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        let layer = object.add_layer(palette_id, red);
        let voxel = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel, &[red]).unwrap();
        let object_id = state.add_object(object).unwrap();

        let request = MaterialMeshRequest {
            method: MeshMethod::Greedy,
            layer,
            scale: 1.0,
            maps,
            storage,
            shape: AtlasShape::Fit,
        };

        object_to_material_gltf(&state, state.object(object_id).unwrap(), &request)
            .unwrap()
            .mesh
    }

    #[test]
    fn embeds_images_as_data_uris() {
        let mesh = cube_gltf(
            vec![MaterialMap {
                name: "cube-albedo.png".to_owned(),
                slot: MaterialSlot::BaseColor,
                bake: MaterialBake::RgbaColor,
            }],
            ResourceStorage::Embedded,
        );

        // Text JSON carrying an embedded PNG data URI, not a binary GLB.
        let text = String::from_utf8(mesh.clone()).unwrap();
        assert!(text.trim_start().starts_with('{'));
        assert!(text.contains("data:image/png;base64,"));

        let gltf = Gltf::from_slice(&mesh).unwrap();
        let material = gltf.materials().next().unwrap();
        assert!(
            material
                .pbr_metallic_roughness()
                .base_color_texture()
                .is_some()
        );
    }

    #[test]
    fn a_map_with_no_slot_lands_in_material_extras() {
        let mesh = cube_gltf(
            vec![
                MaterialMap {
                    name: "cube-albedo.png".to_owned(),
                    slot: MaterialSlot::BaseColor,
                    bake: MaterialBake::RgbaColor,
                },
                MaterialMap {
                    name: "cube-mse.png".to_owned(),
                    slot: MaterialSlot::None,
                    bake: MaterialBake::Packing(vec![MaterialChannel::Attribute {
                        key: METALLIC_FACTOR.to_owned(),
                        component: None,
                        invert: false,
                    }]),
                },
            ],
            ResourceStorage::Embedded,
        );

        let document: Value = serde_json::from_slice(&mesh).unwrap();
        let material = &document["materials"][0];

        // The base color takes texture 0; the slotless map is listed under
        // extras by name, pointing at its texture index.
        assert_eq!(
            material["pbrMetallicRoughness"]["baseColorTexture"]["index"],
            0
        );
        assert_eq!(material["extras"]["vxl"]["maps"]["cube-mse.png"], 1);
    }
}
