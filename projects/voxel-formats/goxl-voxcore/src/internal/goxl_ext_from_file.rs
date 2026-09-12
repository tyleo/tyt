use crate::{
    GoxlExt, GoxlExtCamera, GoxlExtImage, GoxlExtLight, GoxlExtMaterial, GoxlExtPreview,
    GoxlExtUnknownChunk, layer_provenance,
};
use branded_id::U32Id;
use goxl::{GoxlCamera, GoxlFile, GoxlLight, GoxlMaterial};

/// The ext of a read of `file`: everything but the blocks, with one layer
/// entry per layer, keyed by the node id the loader gives the layer, its
/// stored index.
pub fn goxl_ext_from_file(file: &GoxlFile) -> GoxlExt {
    GoxlExt {
        version: file.version,
        image: GoxlExtImage {
            bounding_box: file.image.bounding_box,
            extra: file.image.extra.0.clone(),
        },
        preview: file.preview.as_ref().map(|preview| GoxlExtPreview {
            width: preview.width,
            height: preview.height,
            pixels: preview.pixels.clone(),
        }),
        materials: file.materials.iter().map(material_provenance).collect(),
        layers: file
            .layers
            .iter()
            .enumerate()
            .map(|(index, layer)| (U32Id::from_u32(index as u32), layer_provenance(layer)))
            .collect(),
        cameras: file.cameras.iter().map(camera_provenance).collect(),
        light: file.light.as_ref().map(light_provenance),
        unknown_chunks: file
            .unknown_chunks
            .iter()
            .map(|chunk| GoxlExtUnknownChunk {
                id: chunk.id,
                data: chunk.data.clone(),
            })
            .collect(),
    }
}

/// The ext provenance for one material.
fn material_provenance(material: &GoxlMaterial) -> GoxlExtMaterial {
    GoxlExtMaterial {
        name: material.name.clone(),
        base_color: material.base_color,
        metallic: material.metallic,
        roughness: material.roughness,
        emission: material.emission,
        extra: material.extra.0.clone(),
    }
}

/// The ext provenance for one camera.
fn camera_provenance(camera: &GoxlCamera) -> GoxlExtCamera {
    GoxlExtCamera {
        name: camera.name.clone(),
        distance: camera.distance,
        orthographic: camera.orthographic,
        transform: camera.transform,
        active: camera.active,
        extra: camera.extra.0.clone(),
    }
}

/// The ext provenance for the light settings.
fn light_provenance(light: &GoxlLight) -> GoxlExtLight {
    GoxlExtLight {
        pitch: light.pitch,
        yaw: light.yaw,
        intensity: light.intensity,
        fixed: light.fixed,
        ambient: light.ambient,
        shadow: light.shadow,
        extra: light.extra.0.clone(),
    }
}
