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
        VoxjBound, VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjObject, VoxjPalette,
        VoxjPaletteBinding, VoxjPositionBlock, VoxjRuntimeState, VoxjSampleBlock, VoxjTransform,
        VoxjValuePool,
    };

    /// One `srgba-hex` value pool of four colors, enough to back a four-material
    /// palette's value-indices.
    fn value_pools() -> Vec<VoxjValuePool> {
        vec![VoxjValuePool::SrgbaHex {
            values: vec!["#000000FF".to_owned(); 4],
        }]
    }

    /// A binding of `attribute` to value pool `pool_ref`.
    fn binding(attribute: &str, pool_ref: usize) -> VoxjPaletteBinding {
        VoxjPaletteBinding {
            attribute: attribute.to_owned(),
            pool_ref,
        }
    }

    /// A palette of `materials` materials binding `baseColorFactor` to value
    /// pool 0, its column the value-indices `0..materials`.
    fn palette(materials: usize) -> VoxjPalette {
        VoxjPalette {
            bindings: vec![binding("baseColorFactor", 0)],
            materials: vec![(0..materials).collect()],
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

    /// A small but complete valid document: one four-material palette over a
    /// single color pool, an object sampling it across two in-bounds voxels
    /// (raw-json blocks), and a two-node DAG with a root.
    fn valid_file() -> VoxjFile {
        VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: value_pools(),
                    palettes: vec![palette(4)],
                    objects: vec![VoxjObject {
                        name: "o".to_owned(),
                        layer_palette_refs: vec![0],
                        bounds: [2, 1, 1],
                        origin: [0, 0, 0],
                        voxel_positions: VoxjPositionBlock::RawJson(vec![[0, 0, 0], [1, 0, 0]]),
                        voxel_samples: VoxjSampleBlock::RawJson(vec![vec![1, 3]]),
                    }],
                    hierarchy_nodes: vec![node(vec![1], vec![0]), node(vec![], vec![])],
                    root_hierarchy_nodes: vec![0],
                },
                edit_state: None,
                ext: None,
            },
        }
    }

    /// `valid_file` with `pool` appended as an extra, unreferenced value pool.
    /// No palette binds it, so a content fault in `pool` is the document's only
    /// failure and isolates the value-pools check.
    fn file_with_extra_pool(pool: VoxjValuePool) -> VoxjFile {
        let mut file = valid_file();
        file.main.runtime_state.value_pools.push(pool);
        file
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
    fn rejects_empty_value_pool() {
        let file = file_with_extra_pool(VoxjValuePool::Float {
            min: VoxjBound::None,
            max: VoxjBound::None,
            values: vec![],
        });
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_value_below_min() {
        let file = file_with_extra_pool(VoxjValuePool::Float {
            min: VoxjBound::Number(0.0),
            max: VoxjBound::Number(1.0),
            values: vec![-0.5],
        });
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_value_above_max() {
        let file = file_with_extra_pool(VoxjValuePool::Float {
            min: VoxjBound::Number(0.0),
            max: VoxjBound::Number(1.0),
            values: vec![2.0],
        });
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_min_greater_than_max() {
        let file = file_with_extra_pool(VoxjValuePool::Float {
            min: VoxjBound::Number(1.0),
            max: VoxjBound::Number(0.0),
            values: vec![0.5],
        });
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_non_integer_int_bound() {
        let file = file_with_extra_pool(VoxjValuePool::Int {
            min: VoxjBound::Number(0.0),
            max: VoxjBound::Number(2.5),
            values: vec![1],
        });
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_malformed_hex_color() {
        // 'G' is not a hex digit, so this fails the srgba-hex pattern.
        let file = file_with_extra_pool(VoxjValuePool::SrgbaHex {
            values: vec!["#GGGGGGGG".to_owned()],
        });
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_lowercase_hex_color() {
        // The pattern is uppercase-only; a lowercase digit rejects.
        let file = file_with_extra_pool(VoxjValuePool::SrgbaHex {
            values: vec!["#ff0000ff".to_owned()],
        });
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_srgb_component_out_of_unit_range() {
        // Alpha 1.5 is outside the sRGB range [0, 1].
        let file = file_with_extra_pool(VoxjValuePool::SrgbaFloat {
            values: vec![[1.0, 0.0, 0.0, 1.5]],
        });
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_linear_component_below_zero() {
        let file = file_with_extra_pool(VoxjValuePool::LinearRgbaFloat {
            values: vec![[1.0, 0.0, 0.0, -0.1]],
        });
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn accepts_unbounded_and_hdr_pools() {
        let mut file = valid_file();
        // An unbounded-on-one-side float pool and a linear color above 1 (HDR)
        // are both well-formed.
        file.main
            .runtime_state
            .value_pools
            .push(VoxjValuePool::Float {
                min: VoxjBound::Number(1.0),
                max: VoxjBound::None,
                values: vec![1.5, 42.0],
            });
        file.main
            .runtime_state
            .value_pools
            .push(VoxjValuePool::LinearRgbFloat {
                values: vec![[2.0, 0.0, 5.0]],
            });
        assert!(validate_voxj_file(&file).is_ok());
    }

    #[test]
    fn rejects_duplicate_binding_attribute() {
        let mut file = valid_file();
        // Two bindings of the same attribute; keep the column count and lengths
        // valid so the duplicate is the only fault.
        file.main.runtime_state.palettes[0] = VoxjPalette {
            bindings: vec![binding("baseColorFactor", 0), binding("baseColorFactor", 0)],
            materials: vec![vec![0, 1, 2, 3], vec![0, 1, 2, 3]],
        };
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_materials_column_count_mismatch() {
        let mut file = valid_file();
        // One binding but two materials columns.
        file.main.runtime_state.palettes[0].materials = vec![vec![0, 1, 2, 3], vec![0, 1, 2, 3]];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_materials_value_index_out_of_range() {
        let mut file = valid_file();
        // Pool 0 has four values, so value-index 9 is out of range.
        file.main.runtime_state.palettes[0].materials = vec![vec![0, 1, 2, 9]];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_binding_pool_ref_out_of_range() {
        let mut file = valid_file();
        // The document has a single value pool (index 0); pool ref 9 is out of
        // range.
        file.main.runtime_state.palettes[0].bindings = vec![binding("baseColorFactor", 9)];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_ragged_materials_columns() {
        let mut file = valid_file();
        // Two bindings but columns of unequal length; M is column 0's length, so
        // the short second column violates the shared-length rule.
        file.main.runtime_state.palettes[0] = VoxjPalette {
            bindings: vec![binding("baseColorFactor", 0), binding("metallicFactor", 0)],
            materials: vec![vec![0, 1, 2, 3], vec![0, 1]],
        };
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_layer_palette_ref_out_of_range() {
        let mut file = valid_file();
        file.main.runtime_state.objects[0].layer_palette_refs = vec![5];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn accepts_two_layers_sharing_a_palette() {
        let mut file = valid_file();
        // Two layers may reference the same palette; each carries its own
        // material channel over the two voxels.
        file.main.runtime_state.objects[0].layer_palette_refs = vec![0, 0];
        file.main.runtime_state.objects[0].voxel_samples =
            VoxjSampleBlock::RawJson(vec![vec![1, 3], vec![0, 2]]);
        assert!(validate_voxj_file(&file).is_ok());
    }

    #[test]
    fn rejects_sample_channel_too_short() {
        let mut file = valid_file();
        // One channel for the one layer, but only one value for two voxels.
        file.main.runtime_state.objects[0].voxel_samples = VoxjSampleBlock::RawJson(vec![vec![1]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_sample_channel_count_mismatch() {
        let mut file = valid_file();
        // Two channels where the object has one layer.
        file.main.runtime_state.objects[0].voxel_samples =
            VoxjSampleBlock::RawJson(vec![vec![1, 0], vec![3, 0]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_sample_material_out_of_range() {
        let mut file = valid_file();
        // Palette 0 has four materials; material 9 is out of range. One channel,
        // two voxels: voxel 0 samples material 1, voxel 1 samples material 9.
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
