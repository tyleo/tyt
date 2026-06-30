use crate::{Error, Result, collect_voxj_failures};
use voxj::VoxjFile;

/// Checks a [`VoxjFile`] against the format's document rules, returning the
/// first failure. This is the fail-fast counterpart of
/// [`check_voxj_file`](crate::check_voxj_file()), which runs every check and
/// reports each result; both share one set of checks, listed there.
pub fn validate_voxj_file(file: &VoxjFile) -> Result<()> {
    match collect_voxj_failures(file, true).into_iter().next() {
        Some((_, message)) => Err(Error::Invalid(message)),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use crate::validate_voxj_file;
    use voxj::{
        VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjObject, VoxjPalette, VoxjPositionBlock,
        VoxjRuntimeState, VoxjSampleBlock, VoxjTransform, VoxjValue,
    };

    /// A palette with `cells` rows, each carrying one value per named
    /// attribute: a valid `#RRGGBBAA` string for `rgba`, a number otherwise, so
    /// the palette passes the rgba-format check unless a test makes it fail.
    fn palette(attributes: &[&str], cells: usize) -> VoxjPalette {
        VoxjPalette {
            attributes: attributes.iter().map(|a| (*a).to_owned()).collect(),
            data: (0..cells)
                .map(|_| attributes.iter().map(|a| cell_value(a)).collect())
                .collect(),
        }
    }

    /// The placeholder value for one attribute in [`palette`].
    fn cell_value(attribute: &str) -> VoxjValue {
        if attribute == "rgba" {
            VoxjValue::Text("#000000FF".to_owned())
        } else {
            VoxjValue::Number(0.0)
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
    /// sampling it across two in-bounds voxels (raw-json blocks), and a
    /// two-node DAG with a root.
    fn valid_file() -> VoxjFile {
        VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    objects: vec![VoxjObject {
                        name: "o".to_owned(),
                        palette_refs: vec![0],
                        bounds: [2, 1, 1],
                        origin: [0, 0, 0],
                        voxel_positions: VoxjPositionBlock::RawJson(vec![[0, 0, 0], [1, 0, 0]]),
                        voxel_samples: VoxjSampleBlock::RawJson(vec![vec![1, 3]]),
                    }],
                    palettes: vec![palette(&["rgba"], 4)],
                    hierarchy_nodes: vec![node(vec![1], vec![0]), node(vec![], vec![])],
                    root_hierarchy_nodes: vec![0],
                },
                edit_state: None,
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
        file.main.runtime_state.palettes[0] = palette(&["rgba", "rgba"], 4);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_ragged_palette_row() {
        let mut file = valid_file();
        file.main.runtime_state.palettes[0].data[0] =
            vec![VoxjValue::Number(0.0), VoxjValue::Number(1.0)];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_palette_ref_out_of_range() {
        let mut file = valid_file();
        file.main.runtime_state.objects[0].palette_refs = vec![5];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_duplicate_palette_ref() {
        let mut file = valid_file();
        file.main.runtime_state.objects[0].palette_refs = vec![0, 0];
        file.main.runtime_state.objects[0].voxel_samples =
            VoxjSampleBlock::RawJson(vec![vec![1, 3], vec![1, 3]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_sample_channel_too_short() {
        let mut file = valid_file();
        // One channel for the one palette, but only one value for two voxels.
        file.main.runtime_state.objects[0].voxel_samples = VoxjSampleBlock::RawJson(vec![vec![1]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_sample_channel_count_mismatch() {
        let mut file = valid_file();
        // Two channels where the object references one palette.
        file.main.runtime_state.objects[0].voxel_samples =
            VoxjSampleBlock::RawJson(vec![vec![1, 0], vec![3, 0]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_sample_cell_out_of_range() {
        let mut file = valid_file();
        // Palette 0 has four cells; cell 9 is out of range. One channel, two
        // voxels: voxel 0 samples cell 1, voxel 1 samples cell 9.
        file.main.runtime_state.objects[0].voxel_samples =
            VoxjSampleBlock::RawJson(vec![vec![1, 9]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_position_out_of_bounds() {
        let mut file = valid_file();
        file.main.runtime_state.objects[0].voxel_positions =
            VoxjPositionBlock::RawJson(vec![[0, 0, 0], [5, 0, 0]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_duplicate_position() {
        let mut file = valid_file();
        file.main.runtime_state.objects[0].voxel_positions =
            VoxjPositionBlock::RawJson(vec![[0, 0, 0], [0, 0, 0]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_child_node_out_of_range() {
        let mut file = valid_file();
        file.main.runtime_state.hierarchy_nodes[0].child_nodes = vec![9];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_duplicate_child_node() {
        let mut file = valid_file();
        file.main.runtime_state.hierarchy_nodes[0].child_nodes = vec![1, 1];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_child_object_out_of_range() {
        let mut file = valid_file();
        file.main.runtime_state.hierarchy_nodes[0].child_objects = vec![9];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_duplicate_child_object() {
        let mut file = valid_file();
        file.main.runtime_state.hierarchy_nodes[0].child_objects = vec![0, 0];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_root_out_of_range() {
        let mut file = valid_file();
        file.main.runtime_state.root_hierarchy_nodes = vec![9];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_duplicate_root() {
        let mut file = valid_file();
        file.main.runtime_state.root_hierarchy_nodes = vec![0, 0];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_a_cycle() {
        let mut file = valid_file();
        // node 0 -> node 1 -> node 0.
        file.main.runtime_state.hierarchy_nodes[0].child_nodes = vec![1];
        file.main.runtime_state.hierarchy_nodes[1].child_nodes = vec![0];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn accepts_a_shared_child_dag() {
        let mut file = valid_file();
        // Both nodes share a third leaf node; legal in a DAG.
        file.main.runtime_state.hierarchy_nodes = vec![
            node(vec![2], vec![]),
            node(vec![2], vec![]),
            node(vec![], vec![]),
        ];
        file.main.runtime_state.root_hierarchy_nodes = vec![0, 1];
        assert!(validate_voxj_file(&file).is_ok());
    }

    #[test]
    fn rejects_zero_scale() {
        let mut file = valid_file();
        file.main.runtime_state.hierarchy_nodes[0].transform.scale = [1.0, 0.0, 1.0];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_non_unit_rotation() {
        let mut file = valid_file();
        // Length squared 4, well outside the unit tolerance.
        file.main.runtime_state.hierarchy_nodes[0]
            .transform
            .rotation = [0.0, 0.0, 0.0, 2.0];
        assert!(validate_voxj_file(&file).is_err());
    }
}
