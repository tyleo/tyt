use crate::{
    ChannelSource, ColorComponent, Format, MeshFormat, MeshMethod, MeshTextureMap, ResourceStorage,
    Result, Texture, TextureBake, implementation,
};
use std::{
    fs,
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use voxsmith::{
    ColorChannel, MaterialBake, MaterialChannel, MaterialMap, MaterialMeshRequest, MaterialSlot,
    MeshMethod as VoxsmithMeshMethod, ResourceStorage as VoxsmithResourceStorage,
    object_to_glb_bytes, object_to_gltf_bytes, object_to_material_glb, object_to_material_gltf,
};

/// Meshes the object at index `object` of the voxel file at `input` into a mesh
/// at `output`. With no `maps` it writes pure geometry; otherwise it bakes the
/// object's palette materials into textures the mesh samples, writing any loose
/// images beside `output`. The index is a position into the document's objects,
/// as [`resolve_objects`] returns.
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

    // The material path bakes one layer's materials; default to the object's
    // first layer. An object with no layers has no materials to bake.
    let layer = object
        .iter_layers()
        .next()
        .map(|(layer, _)| layer)
        .ok_or_else(|| {
            IOError::new(
                ErrorKind::InvalidInput,
                format!("object `{}` has no layers to mesh", object.name()),
            )
        })?;

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
