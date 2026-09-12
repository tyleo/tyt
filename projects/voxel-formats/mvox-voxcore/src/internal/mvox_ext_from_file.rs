use crate::{
    Error, MVoxExt, MVoxExtCamera, MVoxExtFrame, MVoxExtLayer, MVoxExtMaterial, MVoxExtNode,
    MVoxExtNodeBody, MVoxExtShapeModel, MVoxExtUnknownChunk, Result, material_type_token,
};
use branded_id::U32Id;
use mvox::{MVoxCamera, MVoxFile, MVoxFrame, MVoxLayer, MVoxSceneNode, MVoxSceneNodeBody};
use std::collections::BTreeMap;

/// The ext of a read of `file`: the MagicaVoxel state with no native voxcore
/// home. Scene node `n` in stored order keys hierarchy node `n`, a model index
/// keys the object at that listing index, and a material's `MATL` id keys the
/// material. Errors on a material id outside `0..=255`, since no material of
/// the loaded palette has it.
pub fn mvox_ext_from_file(file: &MVoxFile) -> Result<MVoxExt> {
    let mut materials = BTreeMap::new();
    for material in &file.materials {
        let Ok(id) = u8::try_from(material.id) else {
            return Err(Error::invalid(format!(
                "material id {} is outside the palette index range 0..=255",
                material.id
            )));
        };
        materials.insert(
            U32Id::from_u32(id as u32),
            MVoxExtMaterial {
                material_type: material.material_type.as_ref().map(material_type_token),
                weight: material.weight,
                rough: material.rough,
                spec: material.spec,
                ior: material.ior,
                att: material.att,
                flux: material.flux,
                extra: material.extra.0.clone(),
            },
        );
    }

    Ok(MVoxExt {
        version: file.version,
        palette_present: file.palette.is_some(),
        materials,
        scene_nodes: file
            .scene_nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (U32Id::from_u32(index as u32), node_provenance(node)))
            .collect(),
        layers: file.layers.iter().map(layer_provenance).collect(),
        render_objects: file
            .render_objects
            .iter()
            .map(|render_object| render_object.attributes.0.clone())
            .collect(),
        cameras: file.cameras.iter().map(camera_provenance).collect(),
        palette_notes: file.palette_notes.clone(),
        index_map: file.index_map.map(|map| map.to_vec()),
        unknown_chunks: file
            .unknown_chunks
            .iter()
            .map(|chunk| MVoxExtUnknownChunk {
                id: chunk.id,
                content: chunk.content.clone(),
                children: chunk.children.clone(),
            })
            .collect(),
    })
}

/// The ext entry for one scene node.
fn node_provenance(node: &MVoxSceneNode) -> MVoxExtNode {
    MVoxExtNode {
        id: node.id,
        hidden: node.attributes.hidden,
        attr_extra: node.attributes.extra.0.clone(),
        body: match &node.body {
            MVoxSceneNodeBody::Transform(transform) => MVoxExtNodeBody::Transform {
                layer: transform.layer,
                frames: transform.frames.iter().map(frame_provenance).collect(),
            },
            MVoxSceneNodeBody::Group(_) => MVoxExtNodeBody::Group,
            MVoxSceneNodeBody::Shape(shape) => MVoxExtNodeBody::Shape {
                models: shape
                    .models
                    .iter()
                    .map(|model| MVoxExtShapeModel {
                        object: U32Id::from_u32(model.model),
                        frame_index: model.frame_index,
                        extra: model.extra.0.clone(),
                    })
                    .collect(),
            },
        },
    }
}

/// The ext provenance for one transform-node frame.
fn frame_provenance(frame: &MVoxFrame) -> MVoxExtFrame {
    MVoxExtFrame {
        rotation: frame.rotation.0,
        translation: frame.translation,
        frame_index: frame.frame_index,
        extra: frame.extra.0.clone(),
    }
}

/// The ext provenance for one layer.
fn layer_provenance(layer: &MVoxLayer) -> MVoxExtLayer {
    MVoxExtLayer {
        id: layer.id,
        name: layer.name.clone(),
        hidden: layer.hidden,
        extra: layer.extra.0.clone(),
    }
}

/// The ext provenance for one camera.
fn camera_provenance(camera: &MVoxCamera) -> MVoxExtCamera {
    MVoxExtCamera {
        id: camera.id,
        mode: camera.mode.clone(),
        focus: camera.focus,
        angle: camera.angle,
        radius: camera.radius,
        frustum: camera.frustum,
        fov: camera.fov,
        extra: camera.extra.0.clone(),
    }
}
