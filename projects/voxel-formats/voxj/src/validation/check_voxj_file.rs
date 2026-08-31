use crate::VoxjFile;
use crate::validation::{VoxjCheck, build_voxj_report, collect_voxj_failures};

/// Checks a [`VoxjFile`] against every one of the format's document rules and
/// returns one [`VoxjCheck`] per check, in a fixed order, each marked passed,
/// failed (with one message per problem found), or unverifiable. Unlike
/// [`validate_voxj_file`](crate::validation::validate_voxj_file()), which stops at the
/// first failure, this runs them all so a report can list every problem.
///
/// The checks, in report order, are:
/// 1. `version`: the version is recognized.
/// 2. `palettes`:
///    1. every property has a non-empty name, distinct within the palette,
///       and an in-range value pool;
///    2. materials hold one row per material, each of exactly one in-range
///       value-index per property.
/// 3. `indices`:
///    1. object layers, node children, child objects, and roots resolve;
///    2. node children, child objects, and roots each appear at most once; a
///       palette may back two layers, so a repeated layer entry is allowed.
/// 4. `blocks`: each object's position and sample blocks decode:
///    1. canonical base64;
///    2. exact bitmap and packed byte counts with zero pad bits;
///    3. well-formed run streams and varints;
///    4. the Hilbert bits cap;
///    5. one channel per layer with one value per voxel.
/// 5. `unique-positions`: voxel positions within an object are unique.
/// 6. `bounds`: positions lie within bounds and bounds are exactly tight.
/// 7. `sample-materials`: each sample indexes a real material of its layer's
///    palette.
/// 8. `acyclic`: the hierarchy has no cycle.
/// 9. `scale`: no transform scale component is zero.
/// 10. `rotation`: every transform rotation is a unit quaternion within `1e-6`.
/// 11. `edit-state`: when present, each edit grid contains its runtime grid.
/// 12. `sample-order`: always unverifiable, an authoring invariant no document
///     can witness.
///
/// A check whose work an earlier failure made moot reports no failure rather
/// than a spurious one: an object's geometry checks are skipped when its layers
/// do not resolve, so they may read as passed while `indices` carries the real
/// fault.
pub fn check_voxj_file(file: &VoxjFile) -> Vec<VoxjCheck> {
    build_voxj_report(collect_voxj_failures(file, false))
}

#[cfg(test)]
mod tests {
    use crate::validation::{VoxjCheck, VoxjCheckStatus, check_voxj_file};
    use crate::{
        VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjObject, VoxjPalette, VoxjPositionBlock,
        VoxjProperty, VoxjRuntimeState, VoxjSampleBlock, VoxjTransform, VoxjValuePool,
    };
    use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

    /// A `vec-4-float` value pool of four colors backing the property's
    /// value-indices, and an unreferenced one-value `float` value pool.
    fn value_pools() -> Vec<VoxjValuePool> {
        vec![
            VoxjValuePool::Vec4Float(vec![[0.0, 0.0, 0.0, 1.0]; 4]),
            VoxjValuePool::Float(vec![1.5]),
        ]
    }

    /// A palette of `materials` materials: one property binding
    /// `baseColor` to value pool 0, its rows the value-indices
    /// `0..materials`.
    fn palette(materials: usize) -> VoxjPalette {
        VoxjPalette {
            properties: vec![VoxjProperty {
                name: "baseColor".to_owned(),
                value_pool: 0,
            }],
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

    /// The same small, complete, valid document the fail-fast tests use: one
    /// four-material palette over a single color value pool, an object sampling
    /// it across two voxels, and a two-node DAG with a root.
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

    /// The status of the check named `name`.
    fn status<'a>(checks: &'a [VoxjCheck], name: &str) -> &'a VoxjCheckStatus {
        &checks
            .iter()
            .find(|check| check.name == name)
            .unwrap_or_else(|| panic!("no check named {name}"))
            .status
    }

    #[test]
    fn reports_every_check_in_order() {
        let checks = check_voxj_file(&valid_file());
        let names: Vec<&str> = checks.iter().map(|check| check.name).collect();
        assert_eq!(
            names,
            [
                "version",
                "palettes",
                "indices",
                "blocks",
                "unique-positions",
                "bounds",
                "sample-materials",
                "acyclic",
                "scale",
                "rotation",
                "edit-state",
                "sample-order",
            ]
        );
    }

    #[test]
    fn passes_every_check_for_a_valid_document() {
        let checks = check_voxj_file(&valid_file());
        for check in &checks {
            let expected = if check.name == "sample-order" {
                VoxjCheckStatus::Unverifiable
            } else {
                VoxjCheckStatus::Passed
            };
            assert_eq!(check.status, expected, "check {}", check.name);
        }
    }

    #[test]
    fn aggregates_independent_failures() {
        let mut file = valid_file();
        // Three faults in three different checks; a fail-fast run would surface
        // only the first.
        file.main.runtime_state.objects[0].layers = vec![5];
        file.main.runtime_state.nodes[0].transform.scale = [1.0, 0.0, 1.0];
        file.main.runtime_state.nodes[1].transform.rotation = [0.0, 0.0, 0.0, 2.0];
        let checks = check_voxj_file(&file);

        assert!(matches!(
            status(&checks, "indices"),
            VoxjCheckStatus::Failed(_)
        ));
        assert!(matches!(
            status(&checks, "scale"),
            VoxjCheckStatus::Failed(_)
        ));
        assert!(matches!(
            status(&checks, "rotation"),
            VoxjCheckStatus::Failed(_)
        ));
        // Untouched checks still pass: the report is not truncated at the first
        // failure.
        assert_eq!(*status(&checks, "version"), VoxjCheckStatus::Passed);
        assert_eq!(*status(&checks, "edit-state"), VoxjCheckStatus::Passed);
    }

    #[test]
    fn records_one_message_per_problem_in_a_check() {
        let mut file = valid_file();
        // Two distinct index faults: an out-of-range layer and a duplicate
        // root.
        file.main.runtime_state.objects[0].layers = vec![5];
        file.main.runtime_state.root_nodes = vec![0, 0];
        let checks = check_voxj_file(&file);
        match status(&checks, "indices") {
            VoxjCheckStatus::Failed(messages) => assert_eq!(messages.len(), 2),
            other => panic!("expected indices to fail, got {other:?}"),
        }
    }

    #[test]
    fn rejects_materials_value_index_out_of_range() {
        let mut file = valid_file();
        // Value pool 0 has four values, so value-index 9 in material 3's row
        // is out of range.
        file.main.runtime_state.palettes[0].materials = vec![vec![0], vec![1], vec![2], vec![9]];
        let checks = check_voxj_file(&file);
        assert!(matches!(
            status(&checks, "palettes"),
            VoxjCheckStatus::Failed(_)
        ));
    }

    #[test]
    fn passes_an_empty_value_pool_and_a_material_less_palette() {
        let mut file = valid_file();
        // A palette with no materials samples nothing, and its property may
        // bind an empty value pool: no material row indexes into it.
        file.main
            .runtime_state
            .value_pools
            .push(VoxjValuePool::Float(vec![]));
        file.main.runtime_state.palettes.push(VoxjPalette {
            properties: vec![VoxjProperty {
                name: "baseColor".to_owned(),
                value_pool: 2,
            }],
            materials: vec![],
        });
        let checks = check_voxj_file(&file);
        assert_eq!(*status(&checks, "palettes"), VoxjCheckStatus::Passed);
    }

    #[test]
    fn reports_block_internal_failure() {
        let mut file = valid_file();
        // A bitmap whose final byte sets a pad bit fails to decode, so the
        // block-internal fault reports through `blocks`. The later geometry
        // checks are skipped for the object, so they still read as passed.
        file.main.runtime_state.objects[0].voxel_positions =
            VoxjPositionBlock::BitmapBase64(BASE64.encode([0xC1]));
        let checks = check_voxj_file(&file);
        assert!(matches!(
            status(&checks, "blocks"),
            VoxjCheckStatus::Failed(_)
        ));
        assert_eq!(
            *status(&checks, "sample-materials"),
            VoxjCheckStatus::Passed
        );
    }

    #[test]
    fn sample_order_is_always_unverifiable() {
        let checks = check_voxj_file(&valid_file());
        assert_eq!(
            *status(&checks, "sample-order"),
            VoxjCheckStatus::Unverifiable
        );
    }
}
