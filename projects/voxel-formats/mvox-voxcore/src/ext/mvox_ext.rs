use crate::ext::{
    MVoxExtCamera, MVoxExtLayer, MVoxExtMaterial, MVoxExtNode, MVoxExtNodeBody, MVoxExtShapeModel,
    MVoxExtUnknownChunk,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use voxcore::VoxExt;

/// The `mvox` ext payload stashed on a [`VoxMain`](voxcore::VoxMain):
/// the MagicaVoxel `.vox` state with no native voxcore home, kept so a file
/// loaded from a MagicaVoxel package can be written back exactly.
///
/// Geometry, colors, and the scene graph become native voxcore entities. This
/// holds the rest. The scene nodes align by index with the hierarchy nodes.
/// The materials align by id with the first palette's materials. Both follow
/// the state through the [`VoxExt`](voxcore::VoxExt) hooks.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MVoxExt {
    /// The format version from the header (`version`).
    pub version: u32,

    /// Whether the file carried an `RGBA` palette chunk. When false the colors
    /// came from the built-in palette and no chunk is written back.
    #[cfg_attr(feature = "serde", serde(rename = "palette-present"))]
    pub palette_present: bool,

    /// Per-material provenance, in stored order: the authoritative type and
    /// scalar fields for write-back.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub materials: Vec<MVoxExtMaterial>,

    /// Per scene-node provenance, aligned by index with the hierarchy nodes.
    /// A node retained after the load has `None`. The writer errors on it
    /// because a hierarchy node synthesizes to several scene nodes, more than
    /// one entry holds.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "scene-nodes", default, skip_serializing_if = "Vec::is_empty")
    )]
    pub scene_nodes: Vec<Option<MVoxExtNode>>,

    /// The layer definitions (`LAYR`), preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub layers: Vec<MVoxExtLayer>,

    /// The render-settings chunks (`rOBJ`), each an ordered key/value list.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "render-objects",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub render_objects: Vec<Vec<(String, String)>>,

    /// The render cameras (`rCAM`), preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub cameras: Vec<MVoxExtCamera>,

    /// The palette color names (`NOTE`), in stored order.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "palette-notes",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub palette_notes: Vec<String>,

    /// The palette index map (`IMAP`) as its 256 bytes, or `None` when the file
    /// omits it. Held as a `Vec` because serde does not derive for `[u8; 256]`.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "index-map", default, skip_serializing_if = "Option::is_none")
    )]
    pub index_map: Option<Vec<u8>>,

    /// Chunks the mvox crate does not model, preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "unknown-chunks",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub unknown_chunks: Vec<MVoxExtUnknownChunk>,
}

/// The MagicaVoxel ext as a state's ext. The scene nodes follow the
/// hierarchy listing. A released node leaves every group's child list. A
/// retained node takes no entry. The writer errors on it. A shape's model
/// indices follow the object listing. The material ids follow the first
/// palette's materials.
impl VoxExt for MVoxExt {
    fn hierarchy_node_did_retain(&mut self, index: usize) {
        self.scene_nodes.insert(index, None);
    }

    fn hierarchy_node_will_release(&mut self, index: usize) {
        let Some(released) = self.scene_nodes.remove(index) else {
            return;
        };

        for node in self.scene_nodes.iter_mut().flatten() {
            if let MVoxExtNodeBody::Group { children } = &mut node.body {
                children.retain(|&child| child != released.id);
            }
        }
    }

    fn object_will_release(&mut self, index: usize) {
        for models in shape_models(self) {
            models.retain(|model| model.model as usize != index);

            for model in models.iter_mut() {
                if model.model as usize > index {
                    model.model -= 1;
                }
            }
        }
    }

    fn object_did_move(&mut self, from: usize, to: usize) {
        for model in shape_models(self).flatten() {
            model.model = moved_index(model.model as usize, from, to) as u32;
        }
    }

    fn materials_will_release(&mut self, palette: usize, indices: &[usize]) {
        if palette != 0 {
            return;
        }

        for &index in indices {
            // An index past `i32` matches no id and shifts none.
            let Ok(index) = i32::try_from(index) else {
                continue;
            };

            self.materials.retain(|material| material.id != index);

            for material in &mut self.materials {
                if material.id > index {
                    material.id -= 1;
                }
            }
        }
    }
}

/// The model lists of the shape nodes.
fn shape_models(ext: &mut MVoxExt) -> impl Iterator<Item = &mut Vec<MVoxExtShapeModel>> {
    ext.scene_nodes
        .iter_mut()
        .flatten()
        .filter_map(|node| match &mut node.body {
            MVoxExtNodeBody::Shape { models } => Some(models),
            _ => None,
        })
}

/// Where listing index `index` lands after the entry at `from` moves to `to`.
fn moved_index(index: usize, from: usize, to: usize) -> usize {
    if index == from {
        to
    } else if from < index && index <= to {
        index - 1
    } else if to <= index && index < from {
        index + 1
    } else {
        index
    }
}
