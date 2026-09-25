use crate::operations::voxelize::{MapSpec, document_of, triangle_of};
use branded_id::U32Id;
use meshdoc::{
    MeshImage, MeshImageMediaType, MeshImageSource, MeshMain, MeshMaterial, MeshPrimitive,
    MeshTexture, MeshTextureRef,
};
use ty_math::{TyLinSrgbF64, TyLinSrgbaF64, TyTransformF64, TyVector2F64, TyVector3F64};

/// A unit quad in the XZ plane at `y = 0` carrying two UV streams, `uv0`
/// and `uv1`, and the given PBR maps. Each map is its own image and texture,
/// so a map samples the stream it declares; a voxel spanning the quad
/// averages the texels under it.
pub(crate) fn pbr_quad_main(
    uv0: [[f64; 2]; 4],
    uv1: [[f64; 2]; 4],
    maps: &[MapSpec],
) -> MeshMain<()> {
    let mut main = MeshMain::default();

    let mut material = MeshMaterial::default();

    for map in maps {
        let image_id = main
            .retain_image(MeshImage {
                name: String::new(),
                media_type: MeshImageMediaType::Png,
                source: MeshImageSource::Bytes(map.png().to_vec()),
            })
            .unwrap();
        let texture_id = main.retain_texture(MeshTexture::new(image_id)).unwrap();
        let texture_ref = |stream: u32| MeshTextureRef {
            texture_id,
            uv_stream_id: U32Id::from_u32(stream),
        };

        match *map {
            MapSpec::BaseColor { stream, factor, .. } => {
                material.base_color_factor = TyLinSrgbaF64::from(factor);
                material.base_color_texture = Some(texture_ref(stream));
            }
            MapSpec::MetallicRoughness {
                stream,
                metallic,
                roughness,
                ..
            } => {
                material.metallic_factor = metallic;
                material.roughness_factor = roughness;
                material.metallic_roughness_texture = Some(texture_ref(stream));
            }
            MapSpec::Emissive { stream, factor, .. } => {
                material.emissive_factor = TyLinSrgbF64::from(factor);
                material.emissive_texture = Some(texture_ref(stream));
            }
            MapSpec::Occlusion {
                stream, strength, ..
            } => {
                material.occlusion_strength = strength;
                material.occlusion_texture = Some(texture_ref(stream));
            }
        }
    }

    let material_id = main.retain_material(material).unwrap();

    let mut primitive = MeshPrimitive::new(
        vec![
            TyVector3F64::new(0.0, 0.0, 0.0),
            TyVector3F64::new(1.0, 0.0, 0.0),
            TyVector3F64::new(1.0, 0.0, 1.0),
            TyVector3F64::new(0.0, 0.0, 1.0),
        ],
        vec![triangle_of(0, 1, 2), triangle_of(0, 2, 3)],
    )
    .unwrap();

    for uvs in [uv0, uv1] {
        primitive
            .push_uv_stream(uvs.iter().map(|uv| TyVector2F64::from_array(*uv)).collect())
            .unwrap();
    }

    primitive.set_material_id(Some(material_id));

    document_of(main, primitive, None, TyTransformF64::default())
}
