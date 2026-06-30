use crate::{VoxjCheck, build_voxj_report, collect_voxj_failures};
use voxj::VoxjFile;

/// Checks a [`VoxjFile`] against every one of the format's document rules and
/// returns one [`VoxjCheck`] per check, in a fixed order, each marked passed,
/// failed (with one message per problem found), or unverifiable. Unlike
/// [`validate_voxj_file`](crate::validate_voxj_file()), which stops at the first
/// failure, this runs them all so a report can list every problem.
///
/// The checks, in report order, are:
/// 1. `version`: the version is recognized.
/// 2. `palettes`: every palette is rectangular, declares each attribute key
///    once, and stores any `rgba` value as a `#RRGGBBAA` string.
/// 3. `indices`: palette refs, node children, child objects, and roots resolve
///    and none repeats within its list.
/// 4. `blocks`: each object's position and sample blocks decode, with one
///    channel per referenced palette and one value per voxel.
/// 5. `unique-positions`: voxel positions within an object are unique.
/// 6. `bounds`: positions lie within bounds and bounds are exactly tight.
/// 7. `sample-cells`: each sample indexes a real cell of the palette it samples.
/// 8. `acyclic`: the hierarchy has no cycle.
/// 9. `scale`: no transform scale component is zero.
/// 10. `rotation`: every transform rotation is a unit quaternion within `1e-6`.
/// 11. `edit-state`: when present, each edit grid contains its runtime grid.
/// 12. `sample-order`: always unverifiable, an authoring invariant no document
///     can witness.
///
/// A check whose work an earlier failure made moot reports no failure rather
/// than a spurious one: an object's geometry checks are skipped when its palette
/// refs do not resolve, so they may read as passed while `indices` carries the
/// real fault.
pub fn check_voxj_file(file: &VoxjFile) -> Vec<VoxjCheck> {
    build_voxj_report(collect_voxj_failures(file, false))
}

#[cfg(test)]
mod tests {
    use crate::{VoxjCheck, VoxjCheckStatus, check_voxj_file};
    use voxj::{
        VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjObject, VoxjPalette, VoxjPositionBlock,
        VoxjRuntimeState, VoxjSampleBlock, VoxjTransform, VoxjValue,
    };

    /// A palette with `cells` rows, each a valid `#RRGGBBAA` rgba value.
    fn rgba_palette(cells: usize) -> VoxjPalette {
        VoxjPalette {
            attributes: vec!["rgba".to_owned()],
            data: (0..cells)
                .map(|_| vec![VoxjValue::Text("#000000FF".to_owned())])
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

    /// The same small, complete, valid document the fail-fast tests use, but
    /// with rgba cells as colors so it passes the rgba-format check.
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
                    palettes: vec![rgba_palette(4)],
                    hierarchy_nodes: vec![node(vec![1], vec![0]), node(vec![], vec![])],
                    root_hierarchy_nodes: vec![0],
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
                "sample-cells",
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
        file.main.runtime_state.objects[0].palette_refs = vec![5];
        file.main.runtime_state.hierarchy_nodes[0].transform.scale = [1.0, 0.0, 1.0];
        file.main.runtime_state.hierarchy_nodes[1]
            .transform
            .rotation = [0.0, 0.0, 0.0, 2.0];
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
        // Two distinct index faults: an out-of-range palette ref and a
        // duplicate root.
        file.main.runtime_state.objects[0].palette_refs = vec![5];
        file.main.runtime_state.root_hierarchy_nodes = vec![0, 0];
        let checks = check_voxj_file(&file);
        match status(&checks, "indices") {
            VoxjCheckStatus::Failed(messages) => assert_eq!(messages.len(), 2),
            other => panic!("expected indices to fail, got {other:?}"),
        }
    }

    #[test]
    fn rejects_lowercase_rgba() {
        let mut file = valid_file();
        // Lowercase hex does not match `^#[0-9A-F]{8}$`.
        file.main.runtime_state.palettes[0].data[0] = vec![VoxjValue::Text("#ffffffff".to_owned())];
        let checks = check_voxj_file(&file);
        assert!(matches!(
            status(&checks, "palettes"),
            VoxjCheckStatus::Failed(_)
        ));
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
