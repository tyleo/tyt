use crate::ext::{
    GoxlExt, GoxlExtCamera, GoxlExtImage, GoxlExtLayer, GoxlExtLight, GoxlExtMaterial,
    GoxlExtPreview, GoxlExtUnknownChunk,
};
use goxl::{GoxlCamera, GoxlFile, GoxlLayer, GoxlLight, GoxlMaterial, GoxlShape};
use voxcore::{VoxExt, VoxMain};

/// Puts the ext a read yields on the bare state. [`GoxlExt`] keeps it. `()`
/// drops it.
pub trait GoxlExtSink: VoxExt + Sized {
    /// Puts on `state` the ext of a read of `file`.
    fn record_file(state: VoxMain<()>, file: &GoxlFile) -> VoxMain<Self>;
}

impl GoxlExtSink for GoxlExt {
    fn record_file(state: VoxMain<()>, file: &GoxlFile) -> VoxMain<Self> {
        state.put_ext(GoxlExt {
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
                .map(|layer| Some(layer_provenance(layer)))
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
        })
    }
}

impl GoxlExtSink for () {
    fn record_file(state: VoxMain<()>, _file: &GoxlFile) -> VoxMain<Self> {
        state
    }
}

/// The ext provenance for one layer: its metadata, clone and shape definition,
/// and the full placement list.
fn layer_provenance(layer: &GoxlLayer) -> GoxlExtLayer {
    GoxlExtLayer {
        name: layer.name.clone(),
        id: layer.id,
        base_id: layer.base_id,
        material: layer.material,
        mode: layer.mode,
        visible: layer.visible,
        transform: layer.transform,
        bounding_box: layer.bounding_box,
        image_path: layer.image_path.clone(),
        shape: layer.shape.map(shape_token),
        color: layer.color,
        placements: layer
            .blocks
            .iter()
            .map(|block| (block.block_index, block.position))
            .collect(),
        extra: layer.extra.0.clone(),
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

/// The on-disk shape name for a procedural shape.
fn shape_token(shape: GoxlShape) -> String {
    match shape {
        GoxlShape::Sphere => "sphere".to_owned(),
        GoxlShape::Cube => "cube".to_owned(),
        GoxlShape::Cylinder => "cylinder".to_owned(),
    }
}
