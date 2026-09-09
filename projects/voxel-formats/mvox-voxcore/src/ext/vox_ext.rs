use crate::ext::{MVoxExt, MVoxExtNodeBody, MVoxExtShapeModel};
use std::any::Any;
use voxcore::{
    VoxMap,
    ext::{FollowListing, Result, VoxExt, encode_entry},
};

/// The MagicaVoxel ext as a state's ext. Its block is the `mvox` entry. The
/// scene nodes follow the hierarchy listing. A released node leaves every
/// group's child list. A retained node takes no entry. The writer errors on
/// it. A shape's model indices follow the object listing. The material ids
/// follow the first palette's materials.
impl VoxExt for MVoxExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        encode_entry(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn VoxExt> {
        Box::new(self.clone())
    }

    fn hierarchy_node_did_retain(&mut self, index: usize) {
        self.scene_nodes.follow_retain(index);
    }

    fn hierarchy_node_will_release(&mut self, index: usize) {
        if index >= self.scene_nodes.len() {
            return;
        }

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
