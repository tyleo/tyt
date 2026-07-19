use crate::{Check, Failures};
use std::collections::HashSet;
use voxj::VoxjMain;

/// Object layers, node children, child objects, and roots all resolve; node
/// children, child objects, and roots each list no index twice. Two layers may
/// reference the same palette, so a repeated layer entry is allowed.
pub fn check_indices(main: &VoxjMain, failures: &mut Failures) {
    let state = &main.runtime_state;

    for (index, object) in state.objects.iter().enumerate() {
        for &layer in &object.layers {
            if !failures.go() {
                return;
            }
            if layer >= state.palettes.len() {
                failures.report(
                    Check::Indices,
                    format!(
                        "object {index} references palette {layer}, but the document has {} palettes",
                        state.palettes.len()
                    ),
                );
            }
        }
    }

    for (index, node) in state.nodes.iter().enumerate() {
        if !failures.go() {
            return;
        }
        let mut seen_nodes = HashSet::with_capacity(node.child_nodes.len());
        for &child in &node.child_nodes {
            if child >= state.nodes.len() {
                failures.report(
                    Check::Indices,
                    format!(
                        "hierarchy node {index} lists child node {child}, but the document has {} nodes",
                        state.nodes.len()
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

    let mut seen_roots = HashSet::with_capacity(state.root_nodes.len());
    for &root in &state.root_nodes {
        if !failures.go() {
            return;
        }
        if root >= state.nodes.len() {
            failures.report(
                Check::Indices,
                format!(
                    "root references hierarchy node {root}, but the document has {} nodes",
                    state.nodes.len()
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
