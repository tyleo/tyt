use crate::{
    DecodeBase64, VoxjFile,
    validation::{Check, VoxjCheck, VoxjCheckStatus, collect_voxj_failures},
};

/// Every check, in the order [`check_voxj_file`] reports them.
const REPORT_ORDER: [Check; 12] = [
    Check::Version,
    Check::Palettes,
    Check::Indices,
    Check::Blocks,
    Check::UniquePositions,
    Check::Bounds,
    Check::SampleMaterials,
    Check::Acyclic,
    Check::Scale,
    Check::Rotation,
    Check::EditState,
    Check::SampleOrder,
];

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
/// fault. The geometry checks decode each object's blocks through
/// `dependencies`.
pub fn check_voxj_file<D: DecodeBase64>(dependencies: &D, file: &VoxjFile) -> Vec<VoxjCheck> {
    build_voxj_report(collect_voxj_failures(dependencies, file, false))
}

/// Groups tagged failures into one [`VoxjCheck`] per check, in
/// [`REPORT_ORDER`]. A check with no failures passed; [`Check::SampleOrder`] is
/// always unverifiable.
fn build_voxj_report(failures: Vec<(Check, String)>) -> Vec<VoxjCheck> {
    REPORT_ORDER
        .iter()
        .map(|&check| {
            let status = if check == Check::SampleOrder {
                VoxjCheckStatus::Unverifiable
            } else {
                let messages: Vec<String> = failures
                    .iter()
                    .filter(|(c, _)| *c == check)
                    .map(|(_, message)| message.clone())
                    .collect();
                if messages.is_empty() {
                    VoxjCheckStatus::Passed
                } else {
                    VoxjCheckStatus::Failed(messages)
                }
            };
            VoxjCheck {
                name: check.name(),
                status,
            }
        })
        .collect()
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        DependenciesImpl, VoxjPalette, VoxjPositionBlock, VoxjProperty, VoxjValuePool,
        test::{standard_base64, valid_file},
        validation::{VoxjCheck, VoxjCheckStatus, check_voxj_file},
    };

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
        let checks = check_voxj_file(&DependenciesImpl, &valid_file());
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
        let checks = check_voxj_file(&DependenciesImpl, &valid_file());
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
        let checks = check_voxj_file(&DependenciesImpl, &file);

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
        let checks = check_voxj_file(&DependenciesImpl, &file);
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
        let checks = check_voxj_file(&DependenciesImpl, &file);
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
        let checks = check_voxj_file(&DependenciesImpl, &file);
        assert_eq!(*status(&checks, "palettes"), VoxjCheckStatus::Passed);
    }

    #[test]
    fn reports_block_internal_failure() {
        let mut file = valid_file();
        // A bitmap whose final byte sets a pad bit fails to decode, so the
        // block-internal fault reports through `blocks`. The later geometry
        // checks are skipped for the object, so they still read as passed.
        file.main.runtime_state.objects[0].voxel_positions =
            VoxjPositionBlock::BitmapBase64(standard_base64(&[0xC1]));
        let checks = check_voxj_file(&DependenciesImpl, &file);
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
        let checks = check_voxj_file(&DependenciesImpl, &valid_file());
        assert_eq!(
            *status(&checks, "sample-order"),
            VoxjCheckStatus::Unverifiable
        );
    }
}
