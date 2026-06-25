use crate::{Error, Result, decode_voxj_object, voxj_palette_cell_counts};
use std::collections::HashSet;
use voxj::{VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjObject, VoxjPalette};

/// The only Voxel Json document version this codec understands.
const SUPPORTED_VERSION: u32 = 1;

/// How far a rotation quaternion's length-squared may stray from `1` and still
/// count as a unit quaternion.
const ROTATION_TOLERANCE: f64 = 1e-6;

/// Checks a [`VoxjFile`] against the format's document rules:
///
/// 1. the version is recognized;
/// 2. palette refs, node children, and roots resolve and are each listed at most
///    once;
/// 3. each sample cell indexes a cell of its palette;
/// 4. voxel positions are unique and within their object's bounds;
/// 5. palettes are rectangular with distinct attribute keys;
/// 6. node transforms have no zero scale component and a unit-length rotation
///    within `1e-6`;
/// 7. the hierarchy is acyclic.
///
/// Each object's blocks are decoded with
/// [`decode_voxj_object`](crate::decode_voxj_object()) to run the geometry
/// checks, so a block that cannot be decoded is reported as invalid. The `rgba`
/// format is unchecked, as is whether the sample channels follow the position
/// block's voxel order, which no document can witness.
pub fn validate_voxj_file(file: &VoxjFile) -> Result<()> {
    if file.version != SUPPORTED_VERSION {
        return Err(invalid(format!(
            "unrecognized version {}, expected {SUPPORTED_VERSION}",
            file.version
        )));
    }
    validate_main(&file.main)
}

/// Checks everything below the version: palettes, objects, hierarchy nodes,
/// roots, and acyclicity.
fn validate_main(main: &VoxjMain) -> Result<()> {
    for (index, palette) in main.palettes.iter().enumerate() {
        validate_palette(index, palette)?;
    }
    for (index, object) in main.objects.iter().enumerate() {
        validate_object(index, object, &main.palettes)?;
    }
    for (index, node) in main.hierarchy_nodes.iter().enumerate() {
        validate_node(index, node, main)?;
    }
    validate_roots(main)?;

    // Acyclicity last, once every child index is known in range.
    if let Some(node) = first_cycle_node(&main.hierarchy_nodes) {
        return Err(invalid(format!(
            "hierarchy is not acyclic: a cycle reaches node {node}"
        )));
    }
    Ok(())
}

/// A palette is rectangular, with one value per attribute in every row, and its
/// attribute keys are distinct.
fn validate_palette(index: usize, palette: &VoxjPalette) -> Result<()> {
    let mut seen = HashSet::with_capacity(palette.attributes.len());
    for attribute in &palette.attributes {
        if !seen.insert(attribute.as_str()) {
            return Err(invalid(format!(
                "palette {index} declares attribute key {attribute:?} more than once"
            )));
        }
    }
    for (cell, row) in palette.data.iter().enumerate() {
        if row.len() != palette.attributes.len() {
            return Err(invalid(format!(
                "palette {index} cell {cell} has {} values but the palette has {} attributes",
                row.len(),
                palette.attributes.len()
            )));
        }
    }
    Ok(())
}

/// An object's palette refs resolve and do not repeat; its decoded samples stay
/// within each palette's cells, and its decoded positions are unique and lie
/// within bounds. Decoding also rejects a sample block whose arity or counts do
/// not match the positions.
fn validate_object(index: usize, object: &VoxjObject, palettes: &[VoxjPalette]) -> Result<()> {
    let mut seen_refs = HashSet::with_capacity(object.palette_refs.len());
    for &palette_ref in &object.palette_refs {
        if palette_ref >= palettes.len() {
            return Err(invalid(format!(
                "object {index} references palette {palette_ref}, but the document has {} palettes",
                palettes.len()
            )));
        }
        if !seen_refs.insert(palette_ref) {
            return Err(invalid(format!(
                "object {index} references palette {palette_ref} more than once"
            )));
        }
    }

    // Refs are in range, so cell counts resolve. Decoding both flattens the
    // geometry and rejects a malformed or mismatched sample block; the object
    // index is added to the message the decoder cannot supply.
    let cell_counts = voxj_palette_cell_counts(&object.palette_refs, palettes)?;
    let decoded = decode_voxj_object(object, &cell_counts).map_err(|e| {
        invalid(format!(
            "object {index} has a malformed position or sample block: {e}"
        ))
    })?;

    for (voxel, row) in decoded.samples.iter().enumerate() {
        // Decoding guarantees one sample per referenced palette, so each channel
        // indexes the palette named at that position.
        for (channel, &cell) in row.iter().enumerate() {
            let palette = object.palette_refs[channel];
            let cell_count = palettes[palette].data.len();
            if cell as usize >= cell_count {
                return Err(invalid(format!(
                    "object {index} voxel {voxel} samples cell {cell} of palette {palette}, \
                     which has {cell_count} cells"
                )));
            }
        }
    }

    let [bound_x, bound_y, bound_z] = object.bounds;
    let mut seen_positions = HashSet::with_capacity(decoded.positions.len());
    for &[x, y, z] in &decoded.positions {
        if x >= bound_x || y >= bound_y || z >= bound_z {
            return Err(invalid(format!(
                "object {index} voxel position [{x}, {y}, {z}] lies outside \
                 bounds [{bound_x}, {bound_y}, {bound_z}]"
            )));
        }
        if !seen_positions.insert([x, y, z]) {
            return Err(invalid(format!(
                "object {index} repeats voxel position [{x}, {y}, {z}]"
            )));
        }
    }
    Ok(())
}

/// A node's children resolve and do not repeat, and its transform is
/// non-degenerate.
fn validate_node(index: usize, node: &VoxjHierarchyNode, main: &VoxjMain) -> Result<()> {
    let mut seen_nodes = HashSet::with_capacity(node.child_nodes.len());
    for &child in &node.child_nodes {
        if child >= main.hierarchy_nodes.len() {
            return Err(invalid(format!(
                "hierarchy node {index} lists child node {child}, \
                 but the document has {} nodes",
                main.hierarchy_nodes.len()
            )));
        }
        if !seen_nodes.insert(child) {
            return Err(invalid(format!(
                "hierarchy node {index} lists child node {child} more than once"
            )));
        }
    }

    let mut seen_objects = HashSet::with_capacity(node.child_objects.len());
    for &object in &node.child_objects {
        if object >= main.objects.len() {
            return Err(invalid(format!(
                "hierarchy node {index} places object {object}, \
                 but the document has {} objects",
                main.objects.len()
            )));
        }
        if !seen_objects.insert(object) {
            return Err(invalid(format!(
                "hierarchy node {index} places object {object} more than once"
            )));
        }
    }

    validate_transform(index, node)
}

/// A node transform has no zero scale component and a unit rotation quaternion.
fn validate_transform(index: usize, node: &VoxjHierarchyNode) -> Result<()> {
    let [scale_x, scale_y, scale_z] = node.transform.scale;
    if scale_x == 0.0 || scale_y == 0.0 || scale_z == 0.0 {
        return Err(invalid(format!(
            "hierarchy node {index} has a transform scale component of zero"
        )));
    }

    let [rotation_x, rotation_y, rotation_z, rotation_w] = node.transform.rotation;
    let length_squared = rotation_x * rotation_x
        + rotation_y * rotation_y
        + rotation_z * rotation_z
        + rotation_w * rotation_w;
    if (length_squared - 1.0).abs() > ROTATION_TOLERANCE {
        return Err(invalid(format!(
            "hierarchy node {index} rotation is not a unit quaternion \
             (length squared {length_squared})"
        )));
    }
    Ok(())
}

/// Roots resolve and do not repeat.
fn validate_roots(main: &VoxjMain) -> Result<()> {
    let mut seen = HashSet::with_capacity(main.root_hierarchy_nodes.len());
    for &root in &main.root_hierarchy_nodes {
        if root >= main.hierarchy_nodes.len() {
            return Err(invalid(format!(
                "root references hierarchy node {root}, but the document has {} nodes",
                main.hierarchy_nodes.len()
            )));
        }
        if !seen.insert(root) {
            return Err(invalid(format!(
                "root lists hierarchy node {root} more than once"
            )));
        }
    }
    Ok(())
}

/// A node on a `child_nodes` cycle, or `None` if the hierarchy is acyclic. An
/// iterative three-colour DFS, so a deep chain cannot overflow the stack: a back
/// edge into an in-progress node is a cycle, revisiting a finished node is not.
/// Call only once every child index is known in range.
fn first_cycle_node(nodes: &[VoxjHierarchyNode]) -> Option<usize> {
    const WHITE: u8 = 0;
    const GREY: u8 = 1;
    const BLACK: u8 = 2;

    let mut colour = vec![WHITE; nodes.len()];
    for start in 0..nodes.len() {
        if colour[start] != WHITE {
            continue;
        }
        colour[start] = GREY;
        // Each frame is a node plus how many of its children we have walked.
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        while let Some(&(node, cursor)) = stack.last() {
            let children = &nodes[node].child_nodes;
            if cursor < children.len() {
                stack.last_mut().unwrap().1 += 1;
                let child = children[cursor];
                match colour[child] {
                    WHITE => {
                        colour[child] = GREY;
                        stack.push((child, 0));
                    }
                    GREY => return Some(child),
                    _ => {}
                }
            } else {
                colour[node] = BLACK;
                stack.pop();
            }
        }
    }
    None
}

/// Wraps a message describing a malformed document as invalid data.
fn invalid(message: String) -> Error {
    Error::Invalid(message)
}

#[cfg(test)]
mod tests {
    use crate::validate_voxj_file;
    use voxj::{
        VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjObject, VoxjPalette, VoxjPositionBlock,
        VoxjSampleBlock, VoxjTransform, VoxjValue,
    };

    /// A palette with `cells` rows, each carrying one value per named attribute.
    fn palette(attributes: &[&str], cells: usize) -> VoxjPalette {
        VoxjPalette {
            attributes: attributes.iter().map(|a| (*a).to_owned()).collect(),
            data: (0..cells)
                .map(|_| attributes.iter().map(|_| VoxjValue::Number(0.0)).collect())
                .collect(),
        }
    }

    /// The identity transform: zero translation, identity rotation, unit scale.
    fn identity() -> VoxjTransform {
        VoxjTransform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }

    /// A node with the given children and an identity transform.
    fn node(child_nodes: Vec<usize>, child_objects: Vec<usize>) -> VoxjHierarchyNode {
        VoxjHierarchyNode {
            name: "n".to_owned(),
            child_nodes,
            child_objects,
            transform: identity(),
        }
    }

    /// A small but complete valid document: one four-cell palette, an object
    /// sampling it across two in-bounds voxels (raw-json blocks), and a two-node
    /// DAG with a root.
    fn valid_file() -> VoxjFile {
        VoxjFile {
            version: 1,
            main: VoxjMain {
                objects: vec![VoxjObject {
                    name: "o".to_owned(),
                    palette_refs: vec![0],
                    bounds: [2, 1, 1],
                    voxel_positions: VoxjPositionBlock::RawJson(vec![[0, 0, 0], [1, 0, 0]]),
                    voxel_samples: VoxjSampleBlock::RawJson(vec![vec![1], vec![3]]),
                }],
                palettes: vec![palette(&["rgba"], 4)],
                hierarchy_nodes: vec![node(vec![1], vec![0]), node(vec![], vec![])],
                root_hierarchy_nodes: vec![0],
                ext: None,
            },
        }
    }

    #[test]
    fn accepts_a_valid_document() {
        assert!(validate_voxj_file(&valid_file()).is_ok());
    }

    #[test]
    fn rejects_unrecognized_version() {
        let mut file = valid_file();
        file.version = 2;
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_duplicate_attribute_key() {
        let mut file = valid_file();
        // Two "rgba" attributes; keep rows rectangular so the duplicate is the
        // only fault.
        file.main.palettes[0] = palette(&["rgba", "rgba"], 4);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_ragged_palette_row() {
        let mut file = valid_file();
        file.main.palettes[0].data[0] = vec![VoxjValue::Number(0.0), VoxjValue::Number(1.0)];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_palette_ref_out_of_range() {
        let mut file = valid_file();
        file.main.objects[0].palette_refs = vec![5];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_duplicate_palette_ref() {
        let mut file = valid_file();
        file.main.objects[0].palette_refs = vec![0, 0];
        file.main.objects[0].voxel_samples = VoxjSampleBlock::RawJson(vec![vec![1, 1], vec![3, 3]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_sample_row_count_mismatch() {
        let mut file = valid_file();
        file.main.objects[0].voxel_samples = VoxjSampleBlock::RawJson(vec![vec![1]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_sample_arity_mismatch() {
        let mut file = valid_file();
        file.main.objects[0].voxel_samples = VoxjSampleBlock::RawJson(vec![vec![1, 0], vec![3]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_sample_cell_out_of_range() {
        let mut file = valid_file();
        // Palette 0 has four cells; cell 9 is out of range.
        file.main.objects[0].voxel_samples = VoxjSampleBlock::RawJson(vec![vec![1], vec![9]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_position_out_of_bounds() {
        let mut file = valid_file();
        file.main.objects[0].voxel_positions =
            VoxjPositionBlock::RawJson(vec![[0, 0, 0], [5, 0, 0]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_duplicate_position() {
        let mut file = valid_file();
        file.main.objects[0].voxel_positions =
            VoxjPositionBlock::RawJson(vec![[0, 0, 0], [0, 0, 0]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_child_node_out_of_range() {
        let mut file = valid_file();
        file.main.hierarchy_nodes[0].child_nodes = vec![9];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_duplicate_child_node() {
        let mut file = valid_file();
        file.main.hierarchy_nodes[0].child_nodes = vec![1, 1];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_child_object_out_of_range() {
        let mut file = valid_file();
        file.main.hierarchy_nodes[0].child_objects = vec![9];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_duplicate_child_object() {
        let mut file = valid_file();
        file.main.hierarchy_nodes[0].child_objects = vec![0, 0];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_root_out_of_range() {
        let mut file = valid_file();
        file.main.root_hierarchy_nodes = vec![9];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_duplicate_root() {
        let mut file = valid_file();
        file.main.root_hierarchy_nodes = vec![0, 0];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_a_cycle() {
        let mut file = valid_file();
        // node 0 -> node 1 -> node 0.
        file.main.hierarchy_nodes[0].child_nodes = vec![1];
        file.main.hierarchy_nodes[1].child_nodes = vec![0];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn accepts_a_shared_child_dag() {
        let mut file = valid_file();
        // Both nodes share a third leaf node; legal in a DAG.
        file.main.hierarchy_nodes = vec![
            node(vec![2], vec![]),
            node(vec![2], vec![]),
            node(vec![], vec![]),
        ];
        file.main.root_hierarchy_nodes = vec![0, 1];
        assert!(validate_voxj_file(&file).is_ok());
    }

    #[test]
    fn rejects_zero_scale() {
        let mut file = valid_file();
        file.main.hierarchy_nodes[0].transform.scale = [1.0, 0.0, 1.0];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_non_unit_rotation() {
        let mut file = valid_file();
        // Length squared 4, well outside the unit tolerance.
        file.main.hierarchy_nodes[0].transform.rotation = [0.0, 0.0, 0.0, 2.0];
        assert!(validate_voxj_file(&file).is_err());
    }
}
