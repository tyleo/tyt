use crate::{Check, Failures};
use std::collections::HashSet;
use voxj::VoxjMain;

/// Palette refs, node children, child objects, and roots all resolve and none
/// repeats within its list.
pub fn check_indices(main: &VoxjMain, failures: &mut Failures) {
    let state = &main.runtime_state;

    for (index, object) in state.objects.iter().enumerate() {
        if !failures.go() {
            return;
        }
        let mut seen = HashSet::with_capacity(object.palette_refs.len());
        for &palette_ref in &object.palette_refs {
            if palette_ref >= state.palettes.len() {
                failures.report(
                    Check::Indices,
                    format!(
                        "object {index} references palette {palette_ref}, but the document has {} palettes",
                        state.palettes.len()
                    ),
                );
            } else if !seen.insert(palette_ref) {
                failures.report(
                    Check::Indices,
                    format!("object {index} references palette {palette_ref} more than once"),
                );
            }
            if !failures.go() {
                return;
            }
        }
    }

    for (index, node) in state.hierarchy_nodes.iter().enumerate() {
        if !failures.go() {
            return;
        }
        let mut seen_nodes = HashSet::with_capacity(node.child_nodes.len());
        for &child in &node.child_nodes {
            if child >= state.hierarchy_nodes.len() {
                failures.report(
                    Check::Indices,
                    format!(
                        "hierarchy node {index} lists child node {child}, but the document has {} nodes",
                        state.hierarchy_nodes.len()
                    ),
                );
            } else if !seen_nodes.insert(child) {
                failures.report(
                    Check::Indices,
                    format!("hierarchy node {index} lists child node {child} more than once"),
                );
            }
            if !failures.go() {
                return;
            }
        }

        let mut seen_objects = HashSet::with_capacity(node.child_objects.len());
        for &object in &node.child_objects {
            if object >= state.objects.len() {
                failures.report(
                    Check::Indices,
                    format!(
                        "hierarchy node {index} places object {object}, but the document has {} objects",
                        state.objects.len()
                    ),
                );
            } else if !seen_objects.insert(object) {
                failures.report(
                    Check::Indices,
                    format!("hierarchy node {index} places object {object} more than once"),
                );
            }
            if !failures.go() {
                return;
            }
        }
    }

    let mut seen_roots = HashSet::with_capacity(state.root_hierarchy_nodes.len());
    for &root in &state.root_hierarchy_nodes {
        if !failures.go() {
            return;
        }
        if root >= state.hierarchy_nodes.len() {
            failures.report(
                Check::Indices,
                format!(
                    "root references hierarchy node {root}, but the document has {} nodes",
                    state.hierarchy_nodes.len()
                ),
            );
        } else if !seen_roots.insert(root) {
            failures.report(
                Check::Indices,
                format!("root lists hierarchy node {root} more than once"),
            );
        }
    }
}
