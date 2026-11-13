use crate::operations::sdf_doc::{SdfEvaluation, box_distance};
use ty_math::TyVector3F64;

/// The most children a leaf holds.
const LEAF_CHILDREN: usize = 8;

/// The deepest a node sits. Median splits keep any union of up to 2^32
/// children above this depth. The limit guards the stack.
const MOST_DEPTH: usize = 40;

/// Relative margin for rounding, so a node never hides a child that could win.
const NODE_TOLERANCE: f64 = 1e-9;

/// A tree over the boxes of a union's children that lets the union skip
/// distant children.
#[derive(Clone, Debug)]
pub struct UnionTree {
    nodes: Vec<UnionNode>,

    /// Each child's position in the union's list, in tree order.
    indices: Vec<u32>,

    /// Each child's box center, in tree order.
    centers: Vec<TyVector3F64>,

    /// Each child's box half extents, in tree order.
    half_extents: Vec<TyVector3F64>,
}

impl UnionTree {
    /// The tree over `boxes`. `boxes` holds each child's box center and half
    /// extents in the union's list order.
    ///
    /// # Panics
    ///
    /// When `boxes` is empty or holds more than `u32::MAX` children.
    pub fn new(boxes: &[(TyVector3F64, TyVector3F64)]) -> Self {
        assert!(!boxes.is_empty(), "a union holds a shape");

        let mut indices: Vec<u32> =
            (0..u32::try_from(boxes.len()).expect("a union's children fit u32")).collect();
        let mut nodes = vec![UnionNode::default()];
        split(&mut nodes, 0, &mut indices, 0, boxes, 0);

        Self {
            nodes,
            centers: indices
                .iter()
                .map(|&index| boxes[index as usize].0)
                .collect(),
            half_extents: indices
                .iter()
                .map(|&index| boxes[index as usize].1)
                .collect(),
            indices,
        }
    }

    /// The union's evaluation at the point `turned` in the boxes' frame. The
    /// child with the least distance wins. On a tie, the earliest in the
    /// union's list wins.
    ///
    /// `evaluate` takes a child's position in the list and the signed distance
    /// from `turned` to the child's box. It returns the child's evaluation.
    /// The returned distance must not be NaN and must read at least that box
    /// distance.
    pub fn nearest(
        &self,
        turned: TyVector3F64,
        mut evaluate: impl FnMut(usize, f64) -> SdfEvaluation,
    ) -> SdfEvaluation {
        let mut nearest = None;
        let mut nearest_distance = f64::INFINITY;
        let mut nearest_index = u32::MAX;

        // Each node still to visit, with the least distance any child under
        // it can read.
        let mut stack = [(0_u32, f64::NEG_INFINITY); MOST_DEPTH + 2];
        let mut pending = 1;

        while pending > 0 {
            pending -= 1;
            let (node_index, reach) = stack[pending];

            if reach > nearest_distance {
                continue;
            }

            let node = &self.nodes[node_index as usize];

            if node.count == 0 {
                let mut near = (node.start, self.reach(node.start, turned));
                let mut far = (node.start + 1, self.reach(node.start + 1, turned));

                if far.1 < near.1 {
                    (near, far) = (far, near);
                }

                stack[pending] = far;
                stack[pending + 1] = near;
                pending += 2;
                continue;
            }

            for slot in node.start as usize..(node.start + node.count) as usize {
                let to_box = box_distance(turned - self.centers[slot], self.half_extents[slot]);

                if to_box > nearest_distance {
                    continue;
                }

                let index = self.indices[slot];
                let evaluation = evaluate(index as usize, to_box);

                if evaluation.distance < nearest_distance
                    || (evaluation.distance == nearest_distance && index < nearest_index)
                {
                    nearest_distance = evaluation.distance;
                    nearest_index = index;
                    nearest = Some(evaluation);
                }
            }
        }

        nearest.expect("a union's children read a distance")
    }

    /// The least distance any child under the node at `node_index` can read
    /// at `turned`.
    fn reach(&self, node_index: u32, turned: TyVector3F64) -> f64 {
        let node = &self.nodes[node_index as usize];
        let distance = box_distance(turned - node.center, node.half_extents);

        // Because the point's offset from the node reads at most the box
        // distance plus the box's diagonal, the slack and the distance term
        // together bound the rounding.
        distance - (node.slack + NODE_TOLERANCE * distance.abs())
    }
}

/// One box of the tree.
#[derive(Clone, Debug, Default)]
struct UnionNode {
    center: TyVector3F64,

    half_extents: TyVector3F64,

    /// The part of the node's rounding margin its size and place set.
    slack: f64,

    /// For a leaf, its first child in tree order. Otherwise, the first of its
    /// two child nodes. The second follows it in the tree's nodes.
    start: u32,

    /// The children a leaf holds, or 0 for a node that splits.
    count: u32,
}

/// Fills the node at `node_index` and the nodes under it with the children
/// `indices`, which begin at `start` in tree order. A leaf lists its children
/// in the union's order.
fn split(
    nodes: &mut Vec<UnionNode>,
    node_index: usize,
    indices: &mut [u32],
    start: usize,
    boxes: &[(TyVector3F64, TyVector3F64)],
    depth: usize,
) {
    let (min, max) = indices.iter().fold(
        (TyVector3F64::INFINITY, TyVector3F64::NEG_INFINITY),
        |(min, max), &index| {
            let (center, half_extents) = boxes[index as usize];
            (
                min.min(center - half_extents),
                max.max(center + half_extents),
            )
        },
    );

    let node = &mut nodes[node_index];
    node.center = (min + max) / 2.0;
    node.half_extents = (max - min) / 2.0;
    node.slack = NODE_TOLERANCE
        * (1.0 + node.center.abs().max_element() + 3.0 * node.half_extents.max_element());

    if indices.len() <= LEAF_CHILDREN || depth == MOST_DEPTH {
        indices.sort_unstable();
        node.start = start as u32;
        node.count = indices.len() as u32;
        return;
    }

    let (center_min, center_max) = indices.iter().fold(
        (TyVector3F64::INFINITY, TyVector3F64::NEG_INFINITY),
        |(min, max), &index| {
            let center = boxes[index as usize].0;
            (min.min(center), max.max(center))
        },
    );
    let spread = center_max - center_min;
    let axis = if spread.x >= spread.y && spread.x >= spread.z {
        0
    } else if spread.y >= spread.z {
        1
    } else {
        2
    };

    let middle = indices.len() / 2;
    indices.select_nth_unstable_by(middle, |a, b| {
        boxes[*a as usize].0[axis].total_cmp(&boxes[*b as usize].0[axis])
    });

    let first = nodes.len();
    nodes.push(UnionNode::default());
    nodes.push(UnionNode::default());
    nodes[node_index].start = first as u32;

    let (low, high) = indices.split_at_mut(middle);
    split(nodes, first, low, start, boxes, depth + 1);
    split(nodes, first + 1, high, start + middle, boxes, depth + 1);
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{SdfEvaluation, UnionTree, box_distance, grid3d};
    use std::cell::Cell;
    use ty_math::TyVector3F64;

    /// A spread of boxes from a fixed linear congruential sequence. Some boxes
    /// repeat so that children tie.
    fn boxes(count: usize) -> Vec<(TyVector3F64, TyVector3F64)> {
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = move || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (state >> 11) as f64 / (1_u64 << 53) as f64
        };

        let mut boxes: Vec<(TyVector3F64, TyVector3F64)> = (0..count)
            .map(|_| {
                let center = TyVector3F64::new(next(), next(), next()) * 8.0 - 4.0;
                let half_extents = TyVector3F64::new(next(), next(), next()) * 0.75 + 0.05;
                (center, half_extents)
            })
            .collect();

        for index in (0..count).step_by(7) {
            boxes[(index * 3) % count] = boxes[index];
        }

        boxes
    }

    /// Child `index`'s evaluation given its box distance `to_box`. The
    /// distance adds a step that repeats every third child, so distinct
    /// children tie. The frame position identifies the child.
    fn child(index: usize, to_box: f64) -> SdfEvaluation {
        SdfEvaluation {
            distance: to_box + (index % 3) as f64 * 0.125,
            frame_position: TyVector3F64::splat(index as f64),
        }
    }

    #[test]
    fn the_tree_finds_what_a_fold_over_every_child_finds() {
        for count in [1, 2, 4, 5, 33, 200] {
            let boxes = boxes(count);
            let tree = UnionTree::new(&boxes);
            let evaluated = Cell::new(0);

            for point in grid3d(TyVector3F64::splat(-6.0), TyVector3F64::splat(6.0), 13) {
                let fold = boxes
                    .iter()
                    .enumerate()
                    .map(|(index, (center, half_extents))| {
                        child(index, box_distance(point - *center, *half_extents))
                    })
                    .reduce(|nearest, evaluation| {
                        if evaluation.distance < nearest.distance {
                            evaluation
                        } else {
                            nearest
                        }
                    })
                    .unwrap();

                let found = tree.nearest(point, |index, to_box| {
                    evaluated.set(evaluated.get() + 1);
                    child(index, to_box)
                });

                assert_eq!(found, fold, "{count} children at {point:?}");
            }

            if count == 200 {
                assert!(evaluated.get() < 13 * 13 * 13 * count / 4);
            }
        }
    }
}
