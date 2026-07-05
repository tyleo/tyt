use crate::{
    ChannelSource, ColorComponent, Format, MeshFormat, MeshMethod, MeshTextureMap, ResourceStorage,
    Result, Texture, TextureBake, implementation,
};
use branded_id::U32Id;
use std::{
    fs,
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use voxcore::{BVoxLayer, VoxObject};
use voxsmith::{
    ColorChannel, MaterialBake, MaterialChannel, MaterialMap, MaterialMeshRequest, MaterialSlot,
    MeshMethod as VoxsmithMeshMethod, ResourceStorage as VoxsmithResourceStorage,
    object_to_glb_bytes, object_to_gltf_bytes, object_to_material_glb, object_to_material_gltf,
};

/// Meshes the object at index `object` of the voxel file at `input` into a mesh
/// at `output`. With no `maps` it writes pure geometry; otherwise it bakes the
/// materials of the object's layer `layer`, a 0-based index into its layers,
/// into textures the mesh samples, writing any loose images beside `output`.
/// The object index is a position into the document's objects, as
/// [`resolve_objects`] returns.
///
/// [`resolve_objects`]: crate::implementation::resolve_objects
#[allow(clippy::too_many_arguments)]
pub fn mesh_object(
    input: &Path,
    from: Option<Format>,
    output: &Path,
    format: MeshFormat,
    scale: f64,
    method: MeshMethod,
    object: usize,
    layer: usize,
    maps: &[MeshTextureMap],
    storage: ResourceStorage,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;

    let (_, object) = state.iter_objects().nth(object).ok_or_else(|| {
        IOError::new(
            ErrorKind::InvalidInput,
            format!("object index {object} is out of range"),
        )
    })?;

    let method = mesh_method(method);

    // No maps is the pure-geometry path, which needs neither the palettes nor
    // the atlas machinery.
    if maps.is_empty() {
        let bytes = match format {
            MeshFormat::Gltf => object_to_gltf_bytes(object, method, scale)?,
            MeshFormat::Glb => object_to_glb_bytes(object, method, scale)?,
        };

        fs::write(output, &bytes)?;

        return Ok(());
    }

    // The material path bakes one layer's materials, chosen by `--layer`.
    let layer = resolve_layer(object, layer)?;

    let request = MaterialMeshRequest {
        method,
        layer,
        scale,
        maps: maps.iter().map(material_map).collect(),
        storage: resource_storage(storage),
    };

    let files = match format {
        MeshFormat::Gltf => object_to_material_gltf(&state, object, &request)?,
        MeshFormat::Glb => object_to_material_glb(&state, object, &request)?,
    };

    fs::write(output, &files.mesh)?;

    // Loose images go beside the mesh, named as the document references them.
    let directory = output.parent().unwrap_or_else(|| Path::new("."));

    for (name, bytes) in &files.sidecars {
        fs::write(directory.join(name), bytes)?;
    }

    Ok(())
}

/// Resolves the `--layer` ordinal to the object's layer id, a 0-based index
/// into its layers in reference order. Errors when the object has no such
/// layer.
fn resolve_layer(object: &VoxObject, layer: usize) -> Result<U32Id<BVoxLayer>> {
    object
        .iter_layers()
        .nth(layer)
        .map(|(id, _)| id)
        .ok_or_else(|| {
            IOError::new(
                ErrorKind::InvalidInput,
                format!(
                    "object `{}` has no layer {layer}; it has {} layer(s)",
                    object.name(),
                    object.layer_count(),
                ),
            )
            .into()
        })
}

/// Maps a CLI meshing method to the voxsmith method.
fn mesh_method(method: MeshMethod) -> VoxsmithMeshMethod {
    match method {
        MeshMethod::Greedy => VoxsmithMeshMethod::Greedy,
        MeshMethod::Culled => VoxsmithMeshMethod::Culled,
        MeshMethod::Naive => VoxsmithMeshMethod::Naive,
    }
}

/// Lowers a resolved map into the voxsmith map, picking its glTF slot from the
/// preset it came from.
fn material_map(map: &MeshTextureMap) -> MaterialMap {
    MaterialMap {
        name: map.name.clone(),
        slot: material_slot(map.preset),
        bake: material_bake(&map.bake),
    }
}

/// The glTF slot a preset fills; a custom packing (or a preset with no standard
/// slot) has none and is reached only through the material's extras.
fn material_slot(preset: Option<Texture>) -> MaterialSlot {
    match preset {
        Some(Texture::Albedo) => MaterialSlot::BaseColor,
        Some(Texture::Orm) => MaterialSlot::OcclusionMetallicRoughness,
        Some(Texture::MetallicRoughness) => MaterialSlot::MetallicRoughness,
        Some(Texture::Occlusion) => MaterialSlot::Occlusion,
        Some(Texture::Emissive) => MaterialSlot::Emissive,
        _ => MaterialSlot::None,
    }
}

/// Lowers a resolved bake into the voxsmith bake.
fn material_bake(bake: &TextureBake) -> MaterialBake {
    match bake {
        TextureBake::RgbaColor => MaterialBake::RgbaColor,
        TextureBake::EmissiveColor => MaterialBake::EmissiveColor,
        TextureBake::Packing(packing) => {
            MaterialBake::Packing(packing.sources().iter().map(material_channel).collect())
        }
    }
}

/// Lowers one resolved channel source into the voxsmith channel.
fn material_channel(source: &ChannelSource) -> MaterialChannel {
    match source {
        ChannelSource::Zero => MaterialChannel::Zero,
        ChannelSource::One => MaterialChannel::One,
        ChannelSource::ComputedOcclusion => MaterialChannel::ComputedOcclusion,
        ChannelSource::Attribute {
            key,
            component,
            invert,
        } => MaterialChannel::Attribute {
            key: key.clone(),
            component: component
                .as_ref()
                .map(|component| color_channel(*component)),
            invert: *invert,
        },
    }
}

/// Maps a CLI color component to the voxsmith color channel.
fn color_channel(component: ColorComponent) -> ColorChannel {
    match component {
        ColorComponent::R => ColorChannel::R,
        ColorComponent::G => ColorChannel::G,
        ColorComponent::B => ColorChannel::B,
        ColorComponent::A => ColorChannel::A,
    }
}

/// Maps a CLI storage mode to the voxsmith storage mode.
fn resource_storage(storage: ResourceStorage) -> VoxsmithResourceStorage {
    match storage {
        ResourceStorage::Embedded => VoxsmithResourceStorage::Embedded,
        ResourceStorage::External => VoxsmithResourceStorage::External,
        ResourceStorage::Both => VoxsmithResourceStorage::Both,
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_layer;
    use ty_math::TyVector3U32;
    use voxcore::{VoxMain, VoxObject, VoxPalette, VoxValuePool};

    /// A standalone object with `layers` layers over one shared palette. The
    /// backing state is dropped; the object owns its layer set.
    fn object_with_layers(layers: usize) -> VoxObject {
        let mut state = VoxMain::default();
        let pool = state.add_value_pool(VoxValuePool::Srgba {
            values: vec![[1.0, 0.0, 0.0, 1.0]],
        });
        let mut palette = VoxPalette::default();
        palette.add_binding("baseColorFactor".to_owned(), pool);
        let material = palette.add_material(vec![0]).unwrap();
        let palette = state.add_palette(palette);

        let mut object = VoxObject::new("body".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        for _ in 0..layers {
            object.add_layer(palette, material);
        }
        object
    }

    #[test]
    fn resolves_each_layer_by_its_ordinal() {
        let object = object_with_layers(2);
        let ids: Vec<_> = object.iter_layers().map(|(id, _)| id).collect();
        assert_eq!(resolve_layer(&object, 0).unwrap(), ids[0]);
        assert_eq!(resolve_layer(&object, 1).unwrap(), ids[1]);
    }

    #[test]
    fn rejects_a_layer_past_the_last() {
        let object = object_with_layers(2);
        assert!(resolve_layer(&object, 2).is_err());
    }

    #[test]
    fn rejects_the_default_layer_on_a_layerless_object() {
        let object = object_with_layers(0);
        assert!(resolve_layer(&object, 0).is_err());
    }
}
