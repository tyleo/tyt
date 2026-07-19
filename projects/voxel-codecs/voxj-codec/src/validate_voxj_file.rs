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
    use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
    use voxj::{
        VoxjArrayProperty, VoxjBound, VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjObject,
        VoxjPalette, VoxjPositionBlock, VoxjRuntimeState, VoxjSampleBlock, VoxjScalarProperty,
        VoxjTransform, VoxjValuePool,
    };

    /// An `srgba-hex` pool of four colors backing the array property's
    /// value-indices, and a one-value `float` pool backing the scalar
    /// property.
    fn value_pools() -> Vec<VoxjValuePool> {
        vec![
            VoxjValuePool::SrgbaHex {
                values: vec!["#000000FF".to_owned(); 4],
            },
            VoxjValuePool::Float {
                min: VoxjBound::None,
                max: VoxjBound::None,
                values: vec![1.5],
            },
        ]
    }

    /// An array property of `name` bound to value pool `value_pool`.
    fn array_property(name: &str, value_pool: usize) -> VoxjArrayProperty {
        VoxjArrayProperty {
            name: name.to_owned(),
            value_pool,
        }
    }

    /// A scalar property of `name` pinning `value_index` of value pool
    /// `value_pool`.
    fn scalar_property(name: &str, value_pool: usize, value_index: usize) -> VoxjScalarProperty {
        VoxjScalarProperty {
            name: name.to_owned(),
            value_pool,
            value_index,
        }
    }

    /// A palette of `materials` materials: one array property binding
    /// `baseColorFactor` to value pool 0, its rows the value-indices
    /// `0..materials`, and one scalar property pinning `emissiveStrength` to
    /// value 0 of the float pool.
    fn palette(materials: usize) -> VoxjPalette {
        VoxjPalette {
            array_properties: vec![array_property("baseColorFactor", 0)],
            scalar_properties: vec![scalar_property("emissiveStrength", 1, 0)],
            materials: (0..materials).map(|i| vec![i]).collect(),
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
                        layers: vec![0],
                        bounds: [2, 1, 1],
                        origin: [0, 0, 0],
                        voxel_positions: VoxjPositionBlock::RawJson(vec![[0, 0, 0], [1, 0, 0]]),
                        voxel_samples: VoxjSampleBlock::RawJson(vec![vec![1, 3]]),
                    }],
                    nodes: vec![node(vec![1], vec![0]), node(vec![], vec![])],
                    root_nodes: vec![0],
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
    fn rejects_duplicate_property_name() {
        let mut file = valid_file();
        // Two array properties of the same name; keep the row arity valid so
        // the duplicate is the only fault.
        file.main.runtime_state.palettes[0] = VoxjPalette {
            array_properties: vec![
                array_property("baseColorFactor", 0),
                array_property("baseColorFactor", 0),
            ],
            scalar_properties: vec![],
            materials: (0..4).map(|i| vec![i, i]).collect(),
        };
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_material_row_arity_mismatch() {
        let mut file = valid_file();
        // One array property but rows of two value-indices.
        file.main.runtime_state.palettes[0].materials = (0..4).map(|i| vec![i, i]).collect();
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_materials_value_index_out_of_range() {
        let mut file = valid_file();
        // Pool 0 has four values, so value-index 9 is out of range.
        file.main.runtime_state.palettes[0].materials = vec![vec![0], vec![1], vec![2], vec![9]];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_property_value_pool_out_of_range() {
        let mut file = valid_file();
        // The document has two value pools; value pool 9 is out of range.
        file.main.runtime_state.palettes[0].array_properties =
            vec![array_property("baseColorFactor", 9)];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_duplicate_name_across_array_and_scalar_properties() {
        let mut file = valid_file();
        // The scalar property reuses the array property's name; the palette's
        // properties share one namespace across both lists.
        file.main.runtime_state.palettes[0].scalar_properties =
            vec![scalar_property("baseColorFactor", 1, 0)];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_empty_scalar_property_name() {
        let mut file = valid_file();
        file.main.runtime_state.palettes[0].scalar_properties = vec![scalar_property("", 1, 0)];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_scalar_value_pool_out_of_range() {
        let mut file = valid_file();
        // The document has two value pools; value pool 9 is out of range.
        file.main.runtime_state.palettes[0].scalar_properties =
            vec![scalar_property("emissiveStrength", 9, 0)];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_scalar_value_index_out_of_range() {
        let mut file = valid_file();
        // Pool 1 has a single value, so value-index 1 is out of range.
        file.main.runtime_state.palettes[0].scalar_properties =
            vec![scalar_property("emissiveStrength", 1, 1)];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_ragged_material_rows() {
        let mut file = valid_file();
        // Two array properties but a short second row; every row must hold
        // exactly one value-index per array property.
        file.main.runtime_state.palettes[0] = VoxjPalette {
            array_properties: vec![
                array_property("baseColorFactor", 0),
                array_property("metallicFactor", 0),
            ],
            scalar_properties: vec![],
            materials: vec![vec![0, 1], vec![0]],
        };
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn accepts_a_property_less_palette_sampled_by_voxels() {
        let mut file = valid_file();
        // With no array properties every row is empty, but each row is still
        // one material, so M stays 4 and the object's samples of materials 1
        // and 3 stay in range.
        file.main.runtime_state.palettes[0] = VoxjPalette {
            array_properties: vec![],
            scalar_properties: vec![],
            materials: vec![vec![], vec![], vec![], vec![]],
        };
        assert!(validate_voxj_file(&file).is_ok());
    }

    #[test]
    fn rejects_a_property_less_palette_with_value_indices() {
        let mut file = valid_file();
        // Without array properties every row must be empty; a stray
        // value-index violates the row rule.
        file.main.runtime_state.palettes[0] = VoxjPalette {
            array_properties: vec![],
            scalar_properties: vec![],
            materials: vec![vec![0]; 4],
        };
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_layer_out_of_range() {
        let mut file = valid_file();
        file.main.runtime_state.objects[0].layers = vec![5];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn accepts_two_layers_sharing_a_palette() {
        let mut file = valid_file();
        // Two layers may reference the same palette; each carries its own
        // material channel over the two voxels.
        file.main.runtime_state.objects[0].layers = vec![0, 0];
        file.main.runtime_state.objects[0].voxel_samples =
            VoxjSampleBlock::RawJson(vec![vec![1, 3], vec![0, 2]]);
        assert!(validate_voxj_file(&file).is_ok());
    }

    /// A scalar-only palette: no array properties and no materials, supplying
    /// one `emissiveStrength` value for any object layered over it.
    fn scalar_only_palette() -> VoxjPalette {
        VoxjPalette {
            array_properties: vec![],
            scalar_properties: vec![scalar_property("emissiveStrength", 1, 0)],
            materials: vec![],
        }
    }

    #[test]
    fn accepts_an_unsampled_layer_with_no_channel() {
        let mut file = valid_file();
        // Palette 1 is scalar-only, so the second layer is unsampled and the
        // single channel belongs to the first layer.
        file.main.runtime_state.palettes.push(scalar_only_palette());
        file.main.runtime_state.objects[0].layers = vec![0, 1];
        assert!(validate_voxj_file(&file).is_ok());
    }

    #[test]
    fn rejects_a_channel_for_an_unsampled_layer() {
        let mut file = valid_file();
        // A scalar-only palette has no materials and is never sampled, so a
        // second channel for its layer is an arity fault.
        file.main.runtime_state.palettes.push(scalar_only_palette());
        file.main.runtime_state.objects[0].layers = vec![0, 1];
        file.main.runtime_state.objects[0].voxel_samples =
            VoxjSampleBlock::RawJson(vec![vec![1, 3], vec![0, 0]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_sample_channel_too_short() {
        let mut file = valid_file();
        // One channel for the one sampled layer, but only one value for two
        // voxels.
        file.main.runtime_state.objects[0].voxel_samples = VoxjSampleBlock::RawJson(vec![vec![1]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_sample_channel_count_mismatch() {
        let mut file = valid_file();
        // Two channels where the object has one sampled layer.
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
        file.main.runtime_state.nodes[0].child_nodes = vec![9];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_duplicate_child_node() {
        let mut file = valid_file();
        file.main.runtime_state.nodes[0].child_nodes = vec![1, 1];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_child_object_out_of_range() {
        let mut file = valid_file();
        file.main.runtime_state.nodes[0].child_objects = vec![9];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_duplicate_child_object() {
        let mut file = valid_file();
        file.main.runtime_state.nodes[0].child_objects = vec![0, 0];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_root_out_of_range() {
        let mut file = valid_file();
        file.main.runtime_state.root_nodes = vec![9];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_duplicate_root() {
        let mut file = valid_file();
        file.main.runtime_state.root_nodes = vec![0, 0];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_a_cycle() {
        let mut file = valid_file();
        // node 0 -> node 1 -> node 0.
        file.main.runtime_state.nodes[0].child_nodes = vec![1];
        file.main.runtime_state.nodes[1].child_nodes = vec![0];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn accepts_a_shared_child_dag() {
        let mut file = valid_file();
        // Both nodes share a third leaf node; legal in a DAG.
        file.main.runtime_state.nodes = vec![
            node(vec![2], vec![]),
            node(vec![2], vec![]),
            node(vec![], vec![]),
        ];
        file.main.runtime_state.root_nodes = vec![0, 1];
        assert!(validate_voxj_file(&file).is_ok());
    }

    #[test]
    fn rejects_zero_scale() {
        let mut file = valid_file();
        file.main.runtime_state.nodes[0].transform.scale = [1.0, 0.0, 1.0];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_non_unit_rotation() {
        let mut file = valid_file();
        // Length squared 4, well outside the unit tolerance.
        file.main.runtime_state.nodes[0].transform.rotation = [0.0, 0.0, 0.0, 2.0];
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn accepts_bitmap_positions() {
        let mut file = valid_file();
        // Cells 0 and 1 of the [2, 1, 1] grid occupied: bits 11 then six zero
        // pad bits, byte 0xC0. Two voxels, so the raw samples still fit.
        file.main.runtime_state.objects[0].voxel_positions =
            VoxjPositionBlock::BitmapBase64(BASE64.encode([0xC0]));
        assert!(validate_voxj_file(&file).is_ok());
    }

    #[test]
    fn accepts_hilbert_positions() {
        let mut file = valid_file();
        // The spec's 2 x 2 x 1 example: Hilbert indices [0, 3, 4, 7] decode to
        // (0,0,0), (0,1,0), (1,1,0), (1,0,0), so bounds are [2, 2, 1] and there
        // are four voxels.
        file.main.runtime_state.objects[0].bounds = [2, 2, 1];
        file.main.runtime_state.objects[0].voxel_positions =
            VoxjPositionBlock::HilbertDeltaVarintBase64("AAMBAw==".to_owned());
        file.main.runtime_state.objects[0].voxel_samples =
            VoxjSampleBlock::RawJson(vec![vec![0, 1, 2, 3]]);
        assert!(validate_voxj_file(&file).is_ok());
    }

    #[test]
    fn rejects_bitmap_with_nonzero_pad_bits() {
        let mut file = valid_file();
        // Byte 0xC1 sets one of the six pad bits past the two occupied cells.
        file.main.runtime_state.objects[0].voxel_positions =
            VoxjPositionBlock::BitmapBase64(BASE64.encode([0xC1]));
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_non_canonical_base64_block() {
        let mut file = valid_file();
        // '_' is a base64url character, not in the standard alphabet.
        file.main.runtime_state.objects[0].voxel_positions =
            VoxjPositionBlock::BitmapBase64("w_==".to_owned());
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_oversized_hilbert_grid() {
        let mut file = valid_file();
        // A 131073-wide axis needs 18 Hilbert bits, over the 17-bit cap.
        file.main.runtime_state.objects[0].bounds = [131073, 1, 1];
        file.main.runtime_state.objects[0].voxel_positions =
            VoxjPositionBlock::HilbertDeltaVarintBase64(String::new());
        file.main.runtime_state.objects[0].voxel_samples =
            VoxjSampleBlock::RawJson(vec![Vec::new()]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_zero_delta_hilbert() {
        let mut file = valid_file();
        // Deltas [0, 0] prefix-sum to indices [0, 0], so the second voxel
        // repeats the first's position; the unique-positions check catches it,
        // which is how the strictly-positive-delta rule is enforced.
        file.main.runtime_state.objects[0].bounds = [1, 1, 1];
        file.main.runtime_state.objects[0].voxel_positions =
            VoxjPositionBlock::HilbertDeltaVarintBase64(BASE64.encode([0x00, 0x00]));
        file.main.runtime_state.objects[0].voxel_samples =
            VoxjSampleBlock::RawJson(vec![vec![0, 0]]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_packed_sample_with_nonzero_pad_bits() {
        let mut file = valid_file();
        // Palette 0 has four materials, so packed width is 2. Two values fill
        // the top four bits; 0x71 sets one of the four pad bits.
        file.main.runtime_state.objects[0].voxel_samples =
            VoxjSampleBlock::PackedBase64(vec![BASE64.encode([0x71])]);
        assert!(validate_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_odd_length_rle_sample() {
        let mut file = valid_file();
        // A dangling value with no count.
        file.main.runtime_state.objects[0].voxel_samples =
            VoxjSampleBlock::RleJson(vec![vec![1, 2, 3]]);
        assert!(validate_voxj_file(&file).is_err());
    }
}
