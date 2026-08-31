use crate::VoxjMain;
use crate::validation::{Check, Failures};

/// When present, edit state lists one edit grid per runtime object and each
/// edit grid contains its object's runtime grid on every axis.
pub fn check_edit_state(main: &VoxjMain, failures: &mut Failures) {
    let Some(edit_state) = &main.edit_state else {
        return;
    };
    let objects = &main.runtime_state.objects;
    if edit_state.objects.len() != objects.len() {
        failures.report(
            Check::EditState,
            format!(
                "edit state lists {} objects, but the document has {} runtime objects",
                edit_state.objects.len(),
                objects.len()
            ),
        );
        return;
    }
    for (object_index, (edit, object)) in edit_state.objects.iter().zip(objects).enumerate() {
        if !failures.go() {
            return;
        }
        for axis in 0..3 {
            let edit_min = i64::from(edit.origin[axis]);
            let edit_max = edit_min + i64::from(edit.bounds[axis]);
            let run_min = i64::from(object.origin[axis]);
            let run_max = run_min + i64::from(object.bounds[axis]);
            if edit_min > run_min || edit_max < run_max {
                failures.report(
                    Check::EditState,
                    format!(
                        "edit grid {object_index} on axis {axis} ([{edit_min}, {edit_max})) does not \
                         contain the runtime grid ([{run_min}, {run_max}))"
                    ),
                );
                if !failures.go() {
                    return;
                }
            }
        }
    }
}
