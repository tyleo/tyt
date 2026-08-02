use crate::{Error, MaterialMeshRequest, MeshFiles, MeshTarget, Result, build_material_document};
use gltf::binary::{Glb, Header};
use std::borrow::Cow;
use voxcore::{VoxMain, VoxObject};

/// Meshes `object` material-aware and writes it as a binary glTF (`.glb`) with a
/// material sampling the baked palette atlas, plus any loose image files
/// `request.storage` asks for. `state` resolves the object's referenced
/// palettes; embedded images share the GLB binary chunk with the geometry. An
/// object with no live voxels writes a valid glTF with an empty scene.
pub fn object_to_material_glb(
    state: &VoxMain,
    object: &VoxObject,
    request: &MaterialMeshRequest,
) -> Result<MeshFiles> {
    let built = build_material_document(state, object, request, MeshTarget::Glb)?;

    let json = serde_json::to_vec(&built.document).map_err(Error::invalid)?;

    let glb = Glb {
        // `to_vec` recomputes the length, so the header length is a placeholder.
        header: Header {
            magic: *b"glTF",
            version: 2,
            length: 0,
        },
        json: Cow::Owned(json),
        bin: (!built.blob.is_empty()).then_some(Cow::Owned(built.blob)),
    };

    Ok(MeshFiles {
        mesh: glb.to_vec()?,
        sidecars: built.sidecars,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        AtlasShape, BASE_COLOR, METALLIC, MaterialBake, MaterialMap, MaterialMeshRequest,
        MaterialSlot, MeshMethod, ResourceStorage, object_to_material_glb,
    };
    use branded_id::U32Id;
    use gltf::{Gltf, image::Source};
    use ty_math::TyVector3U32;
    use voxcore::{BVoxObject, VoxMain, VoxObject, VoxPalette, VoxValuePool};

    /// The 8-byte PNG signature.
    const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    /// A 2x1x1 bar whose two voxels are red and blue through one
    /// `baseColor` palette, so the mesh uses two materials.
    fn red_blue_bar() -> (VoxMain, U32Id<BVoxObject>) {
        let mut state = VoxMain::default();

        let base_value_pool_id = state.add_value_pool(
            VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]]).unwrap(),
        );

        let mut palette = VoxPalette::default();
        palette
            .add_property(
                BASE_COLOR.to_owned(),
                base_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        let red_id = palette.add_material(vec![U32Id::from_u32(0)]).unwrap();
        let blue_id = palette.add_material(vec![U32Id::from_u32(1)]).unwrap();
        let palette_id = state.add_palette(palette).unwrap();

        let mut object = VoxObject::new("bar".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        object.add_layer(palette_id, red_id);
        for (x, material_id) in [(0, red_id), (1, blue_id)] {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel_id, &[material_id]).unwrap();
        }
        let object_id = state.add_object(object).unwrap();

        (state, object_id)
    }

    /// An albedo (base-color) request under `storage`.
    fn albedo_request(storage: ResourceStorage) -> MaterialMeshRequest {
        MaterialMeshRequest {
            method: MeshMethod::Greedy,
            scale: 1.0,
            maps: vec![MaterialMap {
                name: "bar-albedo.png".to_owned(),
                slot: MaterialSlot::BaseColor,
                bake: MaterialBake::RgbaColor,
            }],
            storage,
            shape: AtlasShape::Fit,
        }
    }

    #[test]
    fn embeds_a_base_color_material_and_a_uv_set() {
        let (state, object_id) = red_blue_bar();
        let files = object_to_material_glb(
            &state,
            state.object(object_id).unwrap(),
            &albedo_request(ResourceStorage::Embedded),
        )
        .unwrap();

        // Embedded storage writes no loose files.
        assert!(files.sidecars.is_empty());

        let gltf = Gltf::from_slice(&files.mesh).unwrap();
        assert_eq!(gltf.materials().count(), 1);
        assert_eq!(gltf.textures().count(), 1);
        assert_eq!(gltf.images().count(), 1);

        // The one map wires into the base-color slot, and the image rides in the
        // binary buffer.
        let material = gltf.materials().next().unwrap();
        assert!(
            material
                .pbr_metallic_roughness()
                .base_color_texture()
                .is_some()
        );
        let image = gltf.images().next().unwrap();
        assert!(matches!(image.source(), Source::View { .. }));

        // One UV per vertex, sampling the atlas.
        let primitive = gltf.meshes().next().unwrap().primitives().next().unwrap();
        let blob = gltf.blob.as_deref();
        let reader = primitive.reader(|_| blob);
        let uvs: Vec<[f32; 2]> = reader.read_tex_coords(0).unwrap().into_f32().collect();
        let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
        assert_eq!(uvs.len(), positions.len());
    }

    #[test]
    fn an_out_of_range_vocabulary_value_fails_the_export() {
        // A metallic outside the glTF `[0, 1]` trips the vocabulary range
        // check before anything is written.
        let mut state = VoxMain::default();
        let base_value_pool_id =
            state.add_value_pool(VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap());
        let metallic_value_pool_id = state.add_value_pool(VoxValuePool::float(vec![1.5]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .add_property(
                BASE_COLOR.to_owned(),
                base_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette
            .add_property(
                METALLIC.to_owned(),
                metallic_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        let material_id = palette
            .add_material(vec![U32Id::from_u32(0), U32Id::from_u32(0)])
            .unwrap();
        let palette_id = state.add_palette(palette).unwrap();

        let mut object = VoxObject::new("bar".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.add_layer(palette_id, material_id);
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel_id, &[material_id]).unwrap();
        let object_id = state.add_object(object).unwrap();

        let result = object_to_material_glb(
            &state,
            state.object(object_id).unwrap(),
            &albedo_request(ResourceStorage::Embedded),
        );

        let message = match result {
            Err(error) => error.to_string(),
            Ok(_) => panic!("the export accepted an out-of-range metallic"),
        };
        assert!(message.contains(METALLIC), "{message}");
        assert!(message.contains("1.5"), "{message}");
    }

    #[test]
    fn external_storage_writes_a_sidecar_and_references_it() {
        let (state, object_id) = red_blue_bar();
        let files = object_to_material_glb(
            &state,
            state.object(object_id).unwrap(),
            &albedo_request(ResourceStorage::External),
        )
        .unwrap();

        // The loose PNG is written and named as requested.
        assert_eq!(files.sidecars.len(), 1);
        assert_eq!(files.sidecars[0].0, "bar-albedo.png");
        assert!(files.sidecars[0].1.starts_with(&PNG_MAGIC));

        // The image references the loose file by relative URI.
        let gltf = Gltf::from_slice(&files.mesh).unwrap();
        match gltf.images().next().unwrap().source() {
            Source::Uri { uri, .. } => assert_eq!(uri, "bar-albedo.png"),
            other => panic!("expected an external uri, got {other:?}"),
        }
    }
}
