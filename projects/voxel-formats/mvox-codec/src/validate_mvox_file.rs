use crate::{Result, invalid};
use mvox::{MVoxFile, MVoxSceneNode, MVoxSceneNodeBody};
use std::collections::{HashMap, HashSet};

/// The palette holds 256 colors at indices `0..=255`.
const PALETTE_LEN: usize = 256;

/// The transform-node layer value meaning "no layer".
const NO_LAYER: i32 = -1;

/// Checks a decoded [`MVoxFile`] for cross-reference and bounds faults the `.vox`
/// byte layout cannot catch:
///
/// 1. every voxel lies within its model size and has a usable palette index
///    `1..=255`, never the empty slot `0`;
/// 2. every material id is a palette index `0..=255`;
/// 3. scene node ids are unique, and every transform child and group child
///    resolves to an existing node, with no group repeating a child;
/// 4. layer ids are unique, and every transform layer is `-1` or an existing
///    layer;
/// 5. every shape node indexes an existing model;
/// 6. the scene graph is acyclic.
///
/// Decoding rejects malformed bytes. A hand-built or edited [`MVoxFile`] can
/// still hold dangling references that
/// [`to_mvox_file_bytes`](crate::to_mvox_file_bytes) would write as a structurally
/// valid but broken file. Decoding does not call this. Run it when you need the
/// guarantee.
pub fn validate_mvox_file(file: &MVoxFile) -> Result<()> {
    validate_models(file)?;
    validate_materials(file)?;
    let node_index = node_index(file)?;
    let layer_ids = layer_ids(file)?;
    validate_node_refs(file, &node_index, &layer_ids)?;

    // Acyclicity last, after every child reference resolves.
    if let Some(node) = first_cycle_node(file, &node_index) {
        return Err(invalid(format!(
            "scene graph is not acyclic: a cycle reaches node {node}"
        )));
    }
    Ok(())
}

/// Every voxel sits within its model's bounds and names a usable palette index.
fn validate_models(file: &MVoxFile) -> Result<()> {
    for (index, model) in file.models.iter().enumerate() {
        let [size_x, size_y, size_z] = model.size;
        for voxel in &model.voxels {
            if u32::from(voxel.x) >= size_x
                || u32::from(voxel.y) >= size_y
                || u32::from(voxel.z) >= size_z
            {
                return Err(invalid(format!(
                    "model {index} voxel [{}, {}, {}] lies outside its size \
                     [{size_x}, {size_y}, {size_z}]",
                    voxel.x, voxel.y, voxel.z
                )));
            }
            if voxel.color_index == 0 {
                return Err(invalid(format!(
                    "model {index} voxel [{}, {}, {}] has color index 0, \
                     the reserved empty slot",
                    voxel.x, voxel.y, voxel.z
                )));
            }
        }
    }
    Ok(())
}

/// Every material id names a palette index. Id 0 is allowed here; the empty-slot
/// rule applies to stored voxels, not material definitions.
fn validate_materials(file: &MVoxFile) -> Result<()> {
    for material in &file.materials {
        if material.id < 0 || material.id as usize >= PALETTE_LEN {
            return Err(invalid(format!(
                "material id {} is outside the palette index range 0..={}",
                material.id,
                PALETTE_LEN - 1
            )));
        }
    }
    Ok(())
}

/// Maps each scene node id to its position. Rejects duplicate ids so references
/// resolve unambiguously.
fn node_index(file: &MVoxFile) -> Result<HashMap<i32, usize>> {
    let mut index = HashMap::with_capacity(file.scene_nodes.len());
    for (position, node) in file.scene_nodes.iter().enumerate() {
        if index.insert(node.id, position).is_some() {
            return Err(invalid(format!(
                "scene node id {} is declared more than once",
                node.id
            )));
        }
    }
    Ok(index)
}

/// The layer ids. Rejects duplicates so a transform layer resolves unambiguously.
fn layer_ids(file: &MVoxFile) -> Result<HashSet<i32>> {
    let mut ids = HashSet::with_capacity(file.layers.len());
    for layer in &file.layers {
        if !ids.insert(layer.id) {
            return Err(invalid(format!(
                "layer id {} is declared more than once",
                layer.id
            )));
        }
    }
    Ok(ids)
}

/// Checks each node's outgoing references: a transform child and layer, a group's
/// child nodes with no repeats, and a shape's model indices.
fn validate_node_refs(
    file: &MVoxFile,
    node_index: &HashMap<i32, usize>,
    layer_ids: &HashSet<i32>,
) -> Result<()> {
    for node in &file.scene_nodes {
        match &node.body {
            MVoxSceneNodeBody::Transform(transform) => {
                if !node_index.contains_key(&transform.child) {
                    return Err(invalid(format!(
                        "transform node {} references child node {}, which does not exist",
                        node.id, transform.child
                    )));
                }
                if transform.layer != NO_LAYER && !layer_ids.contains(&transform.layer) {
                    return Err(invalid(format!(
                        "transform node {} references layer {}, which does not exist",
                        node.id, transform.layer
                    )));
                }
            }
            MVoxSceneNodeBody::Group(group) => {
                let mut seen = HashSet::with_capacity(group.children.len());
                for &child in &group.children {
                    if !node_index.contains_key(&child) {
                        return Err(invalid(format!(
                            "group node {} references child node {child}, which does not exist",
                            node.id
                        )));
                    }
                    if !seen.insert(child) {
                        return Err(invalid(format!(
                            "group node {} lists child node {child} more than once",
                            node.id
                        )));
                    }
                }
            }
            MVoxSceneNodeBody::Shape(shape) => {
                for shape_model in &shape.models {
                    if shape_model.model as usize >= file.models.len() {
                        return Err(invalid(format!(
                            "shape node {} references model {}, but the file has {} models",
                            node.id,
                            shape_model.model,
                            file.models.len()
                        )));
                    }
                }
            }
        }
    }
    Ok(())
}

/// A node on a child cycle, or `None` if the scene graph is acyclic. Iterative
/// three-colour DFS, so a deep chain cannot overflow the stack. Call only after
/// every child reference resolves.
fn first_cycle_node(file: &MVoxFile, node_index: &HashMap<i32, usize>) -> Option<i32> {
    const WHITE: u8 = 0;
    const GREY: u8 = 1;
    const BLACK: u8 = 2;

    let nodes = &file.scene_nodes;
    let mut colour = vec![WHITE; nodes.len()];
    for start in 0..nodes.len() {
        if colour[start] != WHITE {
            continue;
        }
        colour[start] = GREY;
        // Each frame is a node position and how many of its children we walked.
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        while let Some(&(position, cursor)) = stack.last() {
            match child_position(nodes, node_index, position, cursor) {
                Some(child) => {
                    stack.last_mut().unwrap().1 += 1;
                    match colour[child] {
                        WHITE => {
                            colour[child] = GREY;
                            stack.push((child, 0));
                        }
                        GREY => return Some(nodes[child].id),
                        _ => {}
                    }
                }
                None => {
                    colour[position] = BLACK;
                    stack.pop();
                }
            }
        }
    }
    None
}

/// Position of node `position`'s `cursor`-th child, or `None` past the last or
/// if the child id is unknown. Transforms have one child, groups a list, shapes
/// none. Refs resolve before this runs, so an unknown id is treated as no child
/// rather than panicking.
fn child_position(
    nodes: &[MVoxSceneNode],
    node_index: &HashMap<i32, usize>,
    position: usize,
    cursor: usize,
) -> Option<usize> {
    let child_id = match &nodes[position].body {
        MVoxSceneNodeBody::Transform(transform) => (cursor == 0).then_some(transform.child),
        MVoxSceneNodeBody::Group(group) => group.children.get(cursor).copied(),
        MVoxSceneNodeBody::Shape(_) => None,
    }?;
    node_index.get(&child_id).copied()
}

#[cfg(test)]
mod tests {
    use crate::validate_mvox_file;
    use mvox::{
        MVoxDict, MVoxFile, MVoxFrame, MVoxGroupNode, MVoxLayer, MVoxMaterial, MVoxModel,
        MVoxNodeAttributes, MVoxSceneNode, MVoxSceneNodeBody, MVoxShapeModel, MVoxShapeNode,
        MVoxTransformNode, MVoxVoxel,
    };

    /// A scene node carrying `body` under id `id` and default attributes.
    fn node(id: i32, body: MVoxSceneNodeBody) -> MVoxSceneNode {
        MVoxSceneNode {
            id,
            attributes: MVoxNodeAttributes::default(),
            body,
        }
    }

    /// A small but complete valid file: one two-voxel model, a transform -> group
    /// -> shape chain drawing it, one layer, and one in-range material.
    fn valid_file() -> MVoxFile {
        MVoxFile {
            models: vec![MVoxModel {
                size: [2, 1, 1],
                voxels: vec![
                    MVoxVoxel {
                        x: 0,
                        y: 0,
                        z: 0,
                        color_index: 1,
                    },
                    MVoxVoxel {
                        x: 1,
                        y: 0,
                        z: 0,
                        color_index: 2,
                    },
                ],
            }],
            scene_nodes: vec![
                node(
                    0,
                    MVoxSceneNodeBody::Transform(MVoxTransformNode {
                        child: 1,
                        layer: 0,
                        frames: vec![MVoxFrame::default()],
                    }),
                ),
                node(
                    1,
                    MVoxSceneNodeBody::Group(MVoxGroupNode { children: vec![2] }),
                ),
                node(
                    2,
                    MVoxSceneNodeBody::Shape(MVoxShapeNode {
                        models: vec![MVoxShapeModel {
                            model: 0,
                            ..Default::default()
                        }],
                    }),
                ),
            ],
            layers: vec![MVoxLayer {
                id: 0,
                name: None,
                hidden: None,
                extra: MVoxDict::default(),
            }],
            materials: vec![MVoxMaterial {
                id: 1,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    /// The transform node's body, for tests that tweak it.
    fn transform_mut(file: &mut MVoxFile) -> &mut MVoxTransformNode {
        match &mut file.scene_nodes[0].body {
            MVoxSceneNodeBody::Transform(transform) => transform,
            _ => unreachable!("node 0 is the transform"),
        }
    }

    /// The group node's body, for tests that tweak it.
    fn group_mut(file: &mut MVoxFile) -> &mut MVoxGroupNode {
        match &mut file.scene_nodes[1].body {
            MVoxSceneNodeBody::Group(group) => group,
            _ => unreachable!("node 1 is the group"),
        }
    }

    #[test]
    fn accepts_a_valid_file() {
        assert!(validate_mvox_file(&valid_file()).is_ok());
    }

    #[test]
    fn accepts_an_empty_file() {
        assert!(validate_mvox_file(&MVoxFile::default()).is_ok());
    }

    #[test]
    fn rejects_a_voxel_outside_its_model() {
        let mut file = valid_file();
        // Size is [2, 1, 1]; x = 9 is out of bounds.
        file.models[0].voxels[0].x = 9;
        assert!(validate_mvox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_voxel_with_the_empty_color_index() {
        let mut file = valid_file();
        file.models[0].voxels[0].color_index = 0;
        assert!(validate_mvox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_material_id_out_of_range() {
        let mut file = valid_file();
        file.materials[0].id = PALETTE_LEN_AS_I32;
        assert!(validate_mvox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_negative_material_id() {
        let mut file = valid_file();
        file.materials[0].id = -1;
        assert!(validate_mvox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_duplicate_node_id() {
        let mut file = valid_file();
        file.scene_nodes[1].id = 0;
        assert!(validate_mvox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_dangling_transform_child() {
        let mut file = valid_file();
        transform_mut(&mut file).child = 99;
        assert!(validate_mvox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_dangling_transform_layer() {
        let mut file = valid_file();
        transform_mut(&mut file).layer = 99;
        assert!(validate_mvox_file(&file).is_err());
    }

    #[test]
    fn accepts_the_no_layer_sentinel() {
        let mut file = valid_file();
        // -1 means "no layer"; it need not resolve even with layers present.
        transform_mut(&mut file).layer = -1;
        assert!(validate_mvox_file(&file).is_ok());
    }

    #[test]
    fn rejects_a_duplicate_layer_id() {
        let mut file = valid_file();
        file.layers.push(MVoxLayer {
            id: 0,
            name: None,
            hidden: None,
            extra: MVoxDict::default(),
        });
        assert!(validate_mvox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_dangling_group_child() {
        let mut file = valid_file();
        group_mut(&mut file).children = vec![99];
        assert!(validate_mvox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_duplicate_group_child() {
        let mut file = valid_file();
        group_mut(&mut file).children = vec![2, 2];
        assert!(validate_mvox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_shape_model_out_of_range() {
        let mut file = valid_file();
        if let MVoxSceneNodeBody::Shape(shape) = &mut file.scene_nodes[2].body {
            shape.models[0].model = 5;
        }
        assert!(validate_mvox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_cycle() {
        let mut file = valid_file();
        // node 0 (transform) -> node 1 (group); make node 1 -> node 0, a cycle.
        group_mut(&mut file).children = vec![0];
        assert!(validate_mvox_file(&file).is_err());
    }

    #[test]
    fn accepts_a_shared_child_dag() {
        let mut file = valid_file();
        // A second transform also targeting the group: sharing a child is a legal
        // DAG, not a cycle.
        file.scene_nodes.push(node(
            3,
            MVoxSceneNodeBody::Transform(MVoxTransformNode {
                child: 1,
                layer: -1,
                frames: vec![MVoxFrame::default()],
            }),
        ));
        assert!(validate_mvox_file(&file).is_ok());
    }

    /// `PALETTE_LEN` as the `i32` a material id is, for the boundary test.
    const PALETTE_LEN_AS_I32: i32 = super::PALETTE_LEN as i32;
}
