use crate::utilities::{QuantizePoint, point_bounds};
use std::mem;
use ty_math::TyVector3Ext;

/// Partitions `points` into at most `target` clusters by octree quantization
/// over the first three axes, merging the rarest points first.
pub fn octree(points: Vec<QuantizePoint>, target: usize) -> Vec<Vec<QuantizePoint>> {
    const DEPTH: u32 = 8;
    const BUCKETS: u32 = 1 << DEPTH;

    // The bucketing box; a point's per-axis bucket in `[0, BUCKETS)` picks its
    // octree path. It covers the point set because oklab and lab axes are
    // signed.
    let (low, high) = point_bounds(&points);
    let (low, high) = (low.truncate(), high.truncate());

    // A flat node arena; node 0 is the root. Only leaves hold point indices.
    struct Node {
        children: [i32; 8],

        points: Vec<usize>,

        count: usize,
    }

    let leaf = || Node {
        children: [-1; 8],
        points: Vec::new(),
        count: 0,
    };

    let mut nodes = vec![leaf()];

    for (index, point) in points.iter().enumerate() {
        debug_assert_eq!(point.coords.w, 0.0, "octree points are 3D");

        let q = point
            .coords
            .truncate()
            .quantize(low, high, BUCKETS)
            .to_array();

        let mut current = 0usize;

        nodes[current].count += 1;

        for level in 0..DEPTH {
            let shift = DEPTH - 1 - level;

            let octant = ((((q[0] >> shift) & 1) << 2)
                | (((q[1] >> shift) & 1) << 1)
                | ((q[2] >> shift) & 1)) as usize;

            current = match nodes[current].children[octant] {
                child if child >= 0 => child as usize,

                _ => {
                    let new = nodes.len();

                    nodes.push(leaf());

                    nodes[current].children[octant] = new as i32;

                    new
                }
            };

            nodes[current].count += 1;
        }

        nodes[current].points.push(index);
    }

    let is_leaf = |nodes: &[Node], index: usize| nodes[index].children.iter().all(|&c| c < 0);

    let mut leaves = (0..nodes.len()).filter(|&i| is_leaf(&nodes, i)).count();

    while leaves > target {
        // The reducible node (all children are leaves) with the fewest points.
        let mut best: Option<(usize, usize)> = None;

        for index in 0..nodes.len() {
            let children = nodes[index].children;

            let has_child = children.iter().any(|&c| c >= 0);

            let all_leaf = children
                .iter()
                .filter(|&&c| c >= 0)
                .all(|&c| is_leaf(&nodes, c as usize));

            if has_child && all_leaf && best.is_none_or(|(_, count)| nodes[index].count < count) {
                best = Some((index, nodes[index].count));
            }
        }

        let Some((node_index, _)) = best else { break };

        let children = nodes[node_index].children;

        let mut folded = 0;

        for child_index in children.into_iter().filter(|&c| c >= 0) {
            let taken = mem::take(&mut nodes[child_index as usize].points);

            nodes[node_index].points.extend(taken);

            folded += 1;
        }

        nodes[node_index].children = [-1; 8];

        leaves = leaves - folded + 1;
    }

    let leaf_indices: Vec<usize> = (0..nodes.len())
        .filter(|&i| is_leaf(&nodes, i) && !nodes[i].points.is_empty())
        .collect();

    leaf_indices
        .into_iter()
        .map(|i| mem::take(&mut nodes[i].points))
        .map(|indices| indices.into_iter().map(|index| points[index]).collect())
        .collect()
}
