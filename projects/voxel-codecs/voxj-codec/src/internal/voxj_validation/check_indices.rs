use crate::{Check, Failures};
use std::collections::HashSet;
use voxj::VoxjMain;

/// Object layers, node children, child objects, and roots all resolve; node
/// children, child objects, and roots each list no index twice. Two layers may
/// reference the same palette, so a repeated layer entry is allowed.
pub fn check_indices(main: &VoxjMain, failures: &mut Failures) {
    let state = &main.runtime_state;

    for (object_index, object) in state.objects.iter().enumerate() {
        for &palette_index in &object.layers {
            if !failures.go() {
                return;
            }
            if palette_index >= state.palettes.len() {
                failures.report(
                    Check::Indices,
                    format!(
                        "object {object_index} references palette {palette_index}, but the document has {} palettes",
                        state.palettes.len()
                    ),
                );
            }
        }
    }

    for (node_index, node) in state.nodes.iter().enumerate() {
        if !failures.go() {
            return;
        }
        let mut seen_nodes = HashSet::with_capacity(node.child_nodes.len());
        for &child_node_index in &node.child_nodes {
            if child_node_index >= state.nodes.len() {
                failures.report(
                    Check::Indices,
                    format!(
                        "hierarchy node {node_index} lists child node {child_node_index}, but the document has {} nodes",
                        state.nodes.len()
                    ),
                );
            } else if !seen_nodes.insert(child_node_index) {
                failures.report(
                    Check::Indices,
                    format!(
                        "hierarchy node {node_index} lists child node {child_node_index} more than once"
                    ),
                );
            }
            if !failures.go() {
                return;
            }
        }

        let mut seen_objects = HashSet::with_capacity(node.child_objects.len());
        for &child_object_index in &node.child_objects {
            if child_object_index >= state.objects.len() {
                failures.report(
                    Check::Indices,
                    format!(
                        "hierarchy node {node_index} places object {child_object_index}, but the document has {} objects",
                        state.objects.len()
                    ),
                );
            } else if !seen_objects.insert(child_object_index) {
                failures.report(
                    Check::Indices,
                    format!(
                        "hierarchy node {node_index} places object {child_object_index} more than once"
                    ),
                );
            }
            if !failures.go() {
                return;
            }
        }
    }

    let mut seen_roots = HashSet::with_capacity(state.root_nodes.len());
    for &root_node_index in &state.root_nodes {
        if !failures.go() {
            return;
        }
        if root_node_index >= state.nodes.len() {
            failures.report(
                Check::Indices,
                format!(
                    "root references hierarchy node {root_node_index}, but the document has {} nodes",
                    state.nodes.len()
                ),
            );
        } else if !seen_roots.insert(root_node_index) {
            failures.report(
                Check::Indices,
                format!("root lists hierarchy node {root_node_index} more than once"),
            );
        }
    }
}
