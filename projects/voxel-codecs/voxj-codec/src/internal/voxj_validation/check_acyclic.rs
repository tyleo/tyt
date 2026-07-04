use crate::{Check, Failures};
use voxj::VoxjMain;

/// The hierarchy is acyclic.
pub fn check_acyclic(main: &VoxjMain, failures: &mut Failures) {
    if let Some(node) = first_cycle_node(main) {
        failures.report(
            Check::Acyclic,
            format!("hierarchy is not acyclic: a cycle reaches node {node}"),
        );
    }
}

/// A node on a `child_nodes` cycle, or `None` if the hierarchy is acyclic. An
/// iterative three-colour DFS, so a deep chain cannot overflow the stack: a
/// back edge into an in-progress node is a cycle, revisiting a finished node is
/// not. An out-of-range child edge is treated as absent, since
/// [`check_indices`](crate::check_indices()) reports it; this keeps the walk
/// safe to run regardless.
fn first_cycle_node(main: &VoxjMain) -> Option<usize> {
    const WHITE: u8 = 0;
    const GREY: u8 = 1;
    const BLACK: u8 = 2;

    let nodes = &main.runtime_state.hierarchy_nodes;
    let mut colour = vec![WHITE; nodes.len()];
    for start in 0..nodes.len() {
        if colour[start] != WHITE {
            continue;
        }
        colour[start] = GREY;
        // Each frame is a node plus how many of its children we have walked.
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        while let Some(&(node, cursor)) = stack.last() {
            let children = &nodes[node].child_nodes;
            if cursor < children.len() {
                stack.last_mut().unwrap().1 += 1;
                let child = children[cursor];
                if child >= nodes.len() {
                    continue;
                }
                match colour[child] {
                    WHITE => {
                        colour[child] = GREY;
                        stack.push((child, 0));
                    }
                    GREY => return Some(child),
                    _ => {}
                }
            } else {
                colour[node] = BLACK;
                stack.pop();
            }
        }
    }
    None
}
