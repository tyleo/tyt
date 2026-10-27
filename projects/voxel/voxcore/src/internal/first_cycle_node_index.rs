use crate::BVoxHierarchyNode;
use branded_id::U32Id;
use std::collections::HashMap;

/// The `children` index of a node lying on a `child_node_ids` cycle, or `None`
/// if the graph is acyclic.
///
/// `children` holds each node's child ids at that node's index, and `index_of`
/// maps a child id back to its index. A child missing from `index_of` leads
/// outside the checked set, where no edge can return, so it is skipped.
///
/// The walk is an iterative three-colour DFS, so a deep chain cannot overflow
/// the stack. A back edge into an in-progress node is a cycle; revisiting a
/// finished one is not.
pub fn first_cycle_node_index(
    children: &[&[U32Id<BVoxHierarchyNode>]],
    index_of: &HashMap<U32Id<BVoxHierarchyNode>, usize>,
) -> Option<usize> {
    const WHITE: u8 = 0;
    const GREY: u8 = 1;
    const BLACK: u8 = 2;

    let count = children.len();
    let mut colour = vec![WHITE; count];

    for start_index in 0..count {
        if colour[start_index] != WHITE {
            continue;
        }

        colour[start_index] = GREY;
        // Each frame is a node index plus how many children we have walked.
        let mut stack: Vec<(usize, usize)> = vec![(start_index, 0)];
        while let Some(&(node_index, cursor)) = stack.last() {
            let node_children = children[node_index];
            match (cursor < node_children.len()).then(|| node_children[cursor]) {
                Some(child_id) => {
                    stack.last_mut().unwrap().1 += 1;

                    let Some(&child_index) = index_of.get(&child_id) else {
                        continue;
                    };

                    match colour[child_index] {
                        WHITE => {
                            colour[child_index] = GREY;
                            stack.push((child_index, 0));
                        }

                        GREY => return Some(child_index),

                        _ => {}
                    }
                }

                None => {
                    colour[node_index] = BLACK;
                    stack.pop();
                }
            }
        }
    }

    None
}
