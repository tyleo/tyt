use crate::VoxjMain;
use crate::validation::{Check, Failures};

/// The hierarchy is acyclic.
pub fn check_acyclic(main: &VoxjMain, failures: &mut Failures) {
    if let Some(node_index) = first_cycle_node_index(main) {
        failures.report(
            Check::Acyclic,
            format!("hierarchy is not acyclic: a cycle reaches node {node_index}"),
        );
    }
}

/// The index of a node on a `child_nodes` cycle, or `None` if the hierarchy is
/// acyclic. An iterative three-colour DFS, so a deep chain cannot overflow the
/// stack: a back edge into an in-progress node is a cycle, revisiting a
/// finished node is not. An out-of-range child edge is treated as absent, since
/// [`check_indices`](crate::validation::check_indices()) reports it; this keeps the walk
/// safe to run regardless.
fn first_cycle_node_index(main: &VoxjMain) -> Option<usize> {
    const WHITE: u8 = 0;
    const GREY: u8 = 1;
    const BLACK: u8 = 2;

    let nodes = &main.runtime_state.nodes;
    let mut colour = vec![WHITE; nodes.len()];
    for start_index in 0..nodes.len() {
        if colour[start_index] != WHITE {
            continue;
        }
        colour[start_index] = GREY;
        // Each frame is a node index plus how many of that node's children we
        // have walked.
        let mut stack: Vec<(usize, usize)> = vec![(start_index, 0)];
        while let Some(&(node_index, cursor)) = stack.last() {
            let children = &nodes[node_index].child_nodes;
            if cursor < children.len() {
                stack.last_mut().unwrap().1 += 1;
                let child_index = children[cursor];
                if child_index >= nodes.len() {
                    continue;
                }
                match colour[child_index] {
                    WHITE => {
                        colour[child_index] = GREY;
                        stack.push((child_index, 0));
                    }
                    GREY => return Some(child_index),
                    _ => {}
                }
            } else {
                colour[node_index] = BLACK;
                stack.pop();
            }
        }
    }
    None
}
