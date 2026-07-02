use crate::{ColorSpace, Dither, Error, ReductionMethod, Result};
use branded_id::U32Id;
use std::{cmp::Ordering, collections::HashMap, mem};
use ty_math::{TySrgbaColor, TyVector3F64};
use voxcore::{BVoxPalette, VoxMain, VoxValue};

/// Reduces `palette` in `state` to at most `max_cells` cells on the
/// material-follows-color rule: the cells are clustered by their `rgba` color
/// with `method` in `space`, and each cluster collapses onto one real
/// representative cell, so a merged voxel adopts the representative's whole row
/// rather than an average. Cells with no `rgba` value are left untouched.
///
/// Returns `Some((before, after))` cell counts when the reduction fired, or
/// `None` when the palette already fit, in which case `method`, `space`, and
/// `dither` are inert. The state is left compacted and referentially valid.
///
/// # Arguments
/// * `state` - the document to reduce in place.
/// * `palette` - the palette to reduce; every object referencing it is remapped.
/// * `max_cells` - the cap; the palette ends with at most this many cells.
/// * `method` - the clustering algorithm.
/// * `space` - the color space colors are compared in.
/// * `dither` - error diffusion applied when snapping samples.
pub fn reduce_palette(
    state: &mut VoxMain,
    palette: U32Id<BVoxPalette>,
    max_cells: usize,
    method: ReductionMethod,
    space: ColorSpace,
    dither: Dither,
) -> Result<Option<(usize, usize)>> {
    // The caller reduces one of the state's palettes; a missing one is a bug,
    // not a silent no-op.
    let palette_ref = state
        .palette(palette)
        .expect("reduce_palette was given a palette not in the state");

    let total = palette_ref.cell_count();
    if total <= max_cells {
        return Ok(None);
    }

    // The color of each cell that has a parseable `rgba`; a colorless cell has
    // nothing to cluster on, so it survives untouched.
    let rgba = palette_ref
        .iter_attributes()
        .find(|(_, name)| *name == "rgba")
        .map(|(attribute, _)| attribute);

    let colored: Vec<(u32, [u8; 4])> = palette_ref
        .iter_cells()
        .filter_map(|cell| {
            let color = rgba
                .and_then(|attribute| palette_ref.cell_value(cell, attribute))
                .and_then(cell_rgba)?;
            Some((cell.to_u32(), color))
        })
        .collect();

    let survivors = total - colored.len();

    if colored.is_empty() {
        return Ok(None);
    }

    // The palette borrow is released now, so tally per-cell voxel usage and
    // place each colored cell in the working space.
    let populations = cell_populations(state, palette);

    let points: Vec<Point> = colored
        .into_iter()
        .map(|(cell, color)| Point {
            cell,
            coords: to_space(color, space),
            population: populations.get(&cell).copied().unwrap_or(0),
        })
        .collect();

    // The reduction is firing, so the controls now apply.
    if !matches!(dither, Dither::None) {
        return Err(unsupported(
            "dithering is not yet implemented; only no dithering is available",
        ));
    }

    let target = max_cells.saturating_sub(survivors).max(1);

    let clusters = match method {
        ReductionMethod::MedianCut => median_cut(points, target),
        ReductionMethod::Octree => octree(points, target),
        ReductionMethod::Kmeans => kmeans(points, target),
    };

    // Collapse every non-representative cell onto its cluster's representative,
    // then compact the holes the removals leave.
    let after = clusters.len() + survivors;

    for cluster in &clusters {
        let representative = representative(cluster);

        for point in cluster {
            if point.cell != representative {
                state.remove_cell(
                    palette,
                    U32Id::from_u32(point.cell),
                    U32Id::from_u32(representative),
                );
            }
        }
    }

    state.gc();

    Ok(Some((total, after)))
}

/// One palette cell as a point to cluster: its id, its color in the working
/// space, and how many live voxels sample it.
#[derive(Clone, Copy)]
struct Point {
    cell: u32,

    coords: TyVector3F64,

    population: u64,
}

/// How many live voxels sample each cell of `palette`, across every object
/// referencing it.
fn cell_populations(state: &VoxMain, palette: U32Id<BVoxPalette>) -> HashMap<u32, u64> {
    let mut populations = HashMap::new();

    for (_, object) in state.iter_objects() {
        for (reference, referenced) in object.iter_palette_refs() {
            if referenced != palette {
                continue;
            }

            for voxel in object.iter_live() {
                if let Some(cell) = object.voxel_cell(voxel, reference) {
                    *populations.entry(cell.to_u32()).or_insert(0) += 1;
                }
            }
        }
    }
    populations
}

/// The representative cell of a cluster: the most-sampled, ties broken by the
/// lowest id, so the choice is deterministic and favors the common color.
fn representative(cluster: &[Point]) -> u32 {
    cluster
        .iter()
        .copied()
        .max_by(|a, b| {
            a.population
                .cmp(&b.population)
                .then_with(|| b.cell.cmp(&a.cell))
        })
        .expect("a cluster holds at least one point")
        .cell
}

/// Partitions `points` into at most `target` clusters by median cut: repeatedly
/// split the box with the widest color axis at its median along that axis, until
/// the target is met or no box can be split further.
fn median_cut(points: Vec<Point>, target: usize) -> Vec<Vec<Point>> {
    let mut boxes = vec![points];

    while boxes.len() < target {
        // The splittable box (two or more distinct-colored points) with the
        // widest single axis.
        let mut widest: Option<(usize, usize, f64)> = None;

        for (index, cell_box) in boxes.iter().enumerate() {
            if cell_box.len() < 2 {
                continue;
            }

            let (axis, extent) = longest_axis(cell_box);
            if extent <= 0.0 {
                continue;
            }

            if widest.is_none_or(|(_, _, best)| extent > best) {
                widest = Some((index, axis, extent));
            }
        }

        let Some((index, axis, _)) = widest else {
            break;
        };

        let mut cell_box = boxes.swap_remove(index);

        cell_box.sort_by(|a, b| {
            a.coords
                .component(axis)
                .partial_cmp(&b.coords.component(axis))
                .unwrap_or(Ordering::Equal)
        });

        let high = cell_box.split_off(cell_box.len() / 2);

        boxes.push(cell_box);
        boxes.push(high);
    }

    boxes
}

/// The axis of widest spread in a box, and that spread.
fn longest_axis(cell_box: &[Point]) -> (usize, f64) {
    let (low, high) = point_bounds(cell_box);

    let spread = (high - low).to_array();

    let mut axis = 0;
    let mut extent = spread[0];

    for (candidate, &value) in spread.iter().enumerate() {
        if value > extent {
            extent = value;
            axis = candidate;
        }
    }

    (axis, extent)
}

/// Partitions `points` into at most `target` clusters by octree quantization:
/// build a fixed-depth octree over the color cube, then repeatedly fold the
/// least-populated node whose children are all leaves into one leaf, until at
/// most `target` leaves remain. Folding the least-populated node first merges the
/// rarest colors first, so common colors keep their own cell.
fn octree(points: Vec<Point>, target: usize) -> Vec<Vec<Point>> {
    const DEPTH: u32 = 8;
    const BUCKETS: u32 = 1 << DEPTH;

    // The box the color cube is bucketed within; a color's per-axis bucket in
    // `[0, BUCKETS)` spells its octree path. The box covers the point set rather
    // than assuming a fixed range, since oklab and lab axes are signed.
    let (low, high) = point_bounds(&points);

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
        let q = point.coords.quantize(low, high, BUCKETS).to_array();

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

    // Fold up until the leaf count fits the target.
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

        let Some((node, _)) = best else { break };

        let children = nodes[node].children;

        let mut folded = 0;

        for child in children.into_iter().filter(|&c| c >= 0) {
            let taken = mem::take(&mut nodes[child as usize].points);

            nodes[node].points.extend(taken);

            folded += 1;
        }

        nodes[node].children = [-1; 8];

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

/// Partitions `points` into at most `target` clusters by k-means: seed centroids
/// by farthest-point (so the start is deterministic, no random init), then
/// alternate nearest-centroid assignment and population-weighted centroid updates
/// until the assignment settles or a step cap is hit. Empty clusters are dropped,
/// so the result may hold fewer than `target`.
fn kmeans(points: Vec<Point>, target: usize) -> Vec<Vec<Point>> {
    const MAX_STEPS: usize = 32;

    let k = target.min(points.len()).max(1);

    // Seed 0 is the most-sampled point (ties to the lowest cell); each next seed
    // is the point farthest from the seeds so far.
    let first = points
        .iter()
        .max_by(|a, b| {
            a.population
                .cmp(&b.population)
                .then_with(|| b.cell.cmp(&a.cell))
        })
        .expect("kmeans is given at least one point");

    let mut centroids = vec![first.coords];

    while centroids.len() < k {
        let mut best: Option<(usize, f64)> = None;

        for (index, point) in points.iter().enumerate() {
            let nearest = centroids
                .iter()
                .map(|centroid| (point.coords - *centroid).magnitude_squared())
                .fold(f64::INFINITY, f64::min);

            if best.is_none_or(|(_, far)| nearest > far) {
                best = Some((index, nearest));
            }
        }

        let (index, distance) = best.expect("points is non-empty");

        if distance <= 0.0 {
            break; // every remaining point coincides with a seed
        }

        centroids.push(points[index].coords);
    }

    // Lloyd iterations: assign, then move each centroid to its cluster's mean.
    let mut assignment = vec![usize::MAX; points.len()];

    for _ in 0..MAX_STEPS {
        let mut changed = false;

        for (index, point) in points.iter().enumerate() {
            let nearest = nearest_centroid(point.coords, &centroids);

            if nearest != assignment[index] {
                assignment[index] = nearest;
                changed = true;
            }
        }

        if !changed {
            break;
        }

        let mut sum = vec![TyVector3F64::default(); centroids.len()];

        let mut weight = vec![0.0f64; centroids.len()];

        for (index, point) in points.iter().enumerate() {
            let cluster = assignment[index];

            let w = point.population.max(1) as f64;

            sum[cluster] = sum[cluster] + point.coords * w;

            weight[cluster] += w;
        }

        for (cluster, centroid) in centroids.iter_mut().enumerate() {
            if weight[cluster] > 0.0 {
                *centroid = sum[cluster] * (1.0 / weight[cluster]);
            }
        }
    }

    let mut clusters: Vec<Vec<Point>> = vec![Vec::new(); centroids.len()];

    for (index, point) in points.into_iter().enumerate() {
        clusters[assignment[index]].push(point);
    }

    clusters.retain(|cluster| !cluster.is_empty());

    clusters
}

/// The `(min, max)` corners of a point set's coordinates, per axis.
fn point_bounds(points: &[Point]) -> (TyVector3F64, TyVector3F64) {
    let mut low = TyVector3F64::INFINITY;
    let mut high = TyVector3F64::NEG_INFINITY;

    for point in points {
        low = low.component_min_with(&point.coords);
        high = high.component_max_with(&point.coords);
    }

    (low, high)
}

/// The index of the nearest centroid to `coords`, ties to the lowest index.
fn nearest_centroid(coords: TyVector3F64, centroids: &[TyVector3F64]) -> usize {
    let mut best = 0;
    let mut best_distance = f64::INFINITY;

    for (index, centroid) in centroids.iter().enumerate() {
        let distance = (coords - *centroid).magnitude_squared();
        if distance < best_distance {
            best_distance = distance;
            best = index;
        }
    }

    best
}

/// A cell's color as a point in the chosen space; alpha is not a clustering
/// dimension (it rides along with the representative's row). The `rgb` space is
/// the naive distance on the stored sRGB components; `oklab` and `lab` decode to
/// linear light first.
fn to_space(rgba: [u8; 4], space: ColorSpace) -> TyVector3F64 {
    let color = TySrgbaColor::from_array(rgba);

    match space {
        ColorSpace::Rgb => color.to_rgba().to_vector3(),
        ColorSpace::Oklab => color.to_linear_rgba().to_oklab().to_vector3(),
        ColorSpace::Lab => color.to_linear_rgba().to_cielab().to_vector3(),
    }
}

/// The straight RGBA of a cell's `rgba` value, or `None` if it is not a
/// `#RRGGBB` / `#RRGGBBAA` hex string.
fn cell_rgba(value: &VoxValue) -> Option<[u8; 4]> {
    let VoxValue::Text(hex) = value else {
        return None;
    };

    let hex = hex.strip_prefix('#')?;
    if hex.len() != 6 && hex.len() != 8 {
        return None;
    }

    let byte = |index: usize| {
        hex.get(index * 2..index * 2 + 2)
            .and_then(|pair| u8::from_str_radix(pair, 16).ok())
    };

    Some([
        byte(0)?,
        byte(1)?,
        byte(2)?,
        if hex.len() == 8 { byte(3)? } else { 255 },
    ])
}

/// An error for a reduction control that is accepted but not yet built.
fn unsupported(message: &str) -> Error {
    Error::invalid(message)
}

#[cfg(test)]
mod tests {
    use crate::{ColorSpace, Dither, ReductionMethod, reduce_palette};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{BVoxObject, BVoxPalette, VoxMain, VoxObject, VoxPalette, VoxValue};

    /// A state of one object whose voxels sample a palette of the given
    /// `#RRGGBBAA` colors, each also carrying a distinct `tag` scalar so a merge
    /// can be seen to take the representative's whole row. Voxel `i` samples cell
    /// `colors[i]`, so every cell has population 1 unless `repeats` gives extra.
    fn state_with_colors(
        colors: &[&str],
        repeats: &[usize],
    ) -> (VoxMain, U32Id<BVoxPalette>, U32Id<BVoxObject>) {
        let mut palette = VoxPalette::default();
        palette.add_attribute("rgba".to_owned());
        palette.add_attribute("tag".to_owned());
        let cells: Vec<_> = colors
            .iter()
            .enumerate()
            .map(|(index, color)| {
                palette
                    .add_cell(vec![
                        VoxValue::Text((*color).to_owned()),
                        VoxValue::Number(index as f64),
                    ])
                    .unwrap()
            })
            .collect();

        let count: usize = colors.len() + repeats.iter().sum::<usize>();
        let mut object =
            VoxObject::new("o".to_owned(), TyVector3U32::new(count as u32, 1, 1)).unwrap();
        let mut state = VoxMain::default();
        let palette_id = state.add_palette(palette);
        object.add_palette_ref(palette_id, cells[0]);

        // One voxel per color, plus `repeats[i]` extra voxels sampling cell i.
        let mut voxel = 0u32;
        let mut retain = |object: &mut VoxObject, cell| {
            object
                .retain_voxel(U32Id::from_u32(voxel), &[cell])
                .unwrap();
            voxel += 1;
        };
        for (index, &cell) in cells.iter().enumerate() {
            retain(&mut object, cell);
            for _ in 0..repeats.get(index).copied().unwrap_or(0) {
                retain(&mut object, cell);
            }
        }
        let object_id = state.add_object(object);
        (state, palette_id, object_id)
    }

    fn cell_count(state: &VoxMain, palette: U32Id<BVoxPalette>) -> usize {
        state.palette(palette).unwrap().cell_count()
    }

    /// The set of rgba hexes still in the palette.
    fn colors(state: &VoxMain, palette: U32Id<BVoxPalette>) -> Vec<String> {
        let palette = state.palette(palette).unwrap();
        let (rgba, _) = palette
            .iter_attributes()
            .find(|(_, name)| *name == "rgba")
            .unwrap();
        palette
            .iter_cells()
            .filter_map(|cell| match palette.cell_value(cell, rgba) {
                Some(VoxValue::Text(hex)) => Some(hex.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn no_op_when_already_within_the_cap() {
        let (mut state, palette, _) = state_with_colors(&["#FF0000FF", "#00FF00FF"], &[]);
        let outcome = reduce_palette(
            &mut state,
            palette,
            5,
            ReductionMethod::MedianCut,
            ColorSpace::Oklab,
            Dither::None,
        )
        .unwrap();
        assert_eq!(outcome, None);
        assert_eq!(cell_count(&state, palette), 2);
    }

    #[test]
    fn merges_near_colors_and_keeps_a_real_representative_row() {
        // Two near-identical reds and one blue; cap 2 fuses the reds. The second
        // red is sampled more, so it is the representative and its whole row
        // (tag 1) survives, not an average.
        let (mut state, palette, object) =
            state_with_colors(&["#FE0000FF", "#FF0000FF", "#0000FFFF"], &[0, 3, 0]);
        let outcome = reduce_palette(
            &mut state,
            palette,
            2,
            ReductionMethod::MedianCut,
            ColorSpace::Oklab,
            Dither::None,
        )
        .unwrap();
        assert_eq!(outcome, Some((3, 2)));
        assert_eq!(cell_count(&state, palette), 2);
        assert_eq!(state.validate(), Ok(()));

        let mut survivors = colors(&state, palette);
        survivors.sort();
        assert_eq!(survivors, ["#0000FFFF", "#FF0000FF"]);

        // The voxel that sampled the fused first red now samples the survivor,
        // whose tag is 1 (material followed color, whole row).
        let object = state.object(object).unwrap();
        let (reference, _) = object.iter_palette_refs().next().unwrap();
        let palette_ref = state.palette(palette).unwrap();
        let (tag, _) = palette_ref
            .iter_attributes()
            .find(|(_, name)| *name == "tag")
            .unwrap();
        let voxel = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        let cell = object.voxel_cell(voxel, reference).unwrap();
        assert_eq!(
            palette_ref.cell_value(cell, tag),
            Some(&VoxValue::Number(1.0))
        );
    }

    #[test]
    fn octree_and_kmeans_reduce_to_the_cap() {
        // Both methods cluster four colors down to at most the cap of two.
        for method in [ReductionMethod::Octree, ReductionMethod::Kmeans] {
            let (mut state, palette, _) =
                state_with_colors(&["#FF0000FF", "#00FF00FF", "#0000FFFF", "#FFFF00FF"], &[]);
            let outcome = reduce_palette(
                &mut state,
                palette,
                2,
                method,
                ColorSpace::Oklab,
                Dither::None,
            )
            .unwrap();
            let (before, after) = outcome.expect("the reduction fired");
            assert_eq!(before, 4, "method {method:?}");
            assert!(
                (1..=2).contains(&after),
                "method {method:?} left {after} cells"
            );
            assert_eq!(cell_count(&state, palette), after, "method {method:?}");
            assert_eq!(state.validate(), Ok(()), "method {method:?}");
        }
    }

    #[test]
    fn dither_errors_only_when_the_reduction_fires() {
        // Under the cap: dither is inert, so no error even though it is unbuilt.
        let (mut state, palette, _) = state_with_colors(&["#FF0000FF", "#00FF00FF"], &[]);
        assert!(
            reduce_palette(
                &mut state,
                palette,
                5,
                ReductionMethod::MedianCut,
                ColorSpace::Oklab,
                Dither::FloydSteinberg
            )
            .unwrap()
            .is_none()
        );

        // Over the cap, dither is not yet built, so it errors.
        let (mut state, palette, _) =
            state_with_colors(&["#FF0000FF", "#00FF00FF", "#0000FFFF"], &[]);
        assert!(
            reduce_palette(
                &mut state,
                palette,
                2,
                ReductionMethod::MedianCut,
                ColorSpace::Oklab,
                Dither::Ordered
            )
            .is_err()
        );
    }

    #[test]
    fn reduces_across_all_color_spaces() {
        for space in [ColorSpace::Oklab, ColorSpace::Lab, ColorSpace::Rgb] {
            let (mut state, palette, _) =
                state_with_colors(&["#FF0000FF", "#00FF00FF", "#0000FFFF", "#FFFF00FF"], &[]);
            let outcome = reduce_palette(
                &mut state,
                palette,
                2,
                ReductionMethod::MedianCut,
                space,
                Dither::None,
            )
            .unwrap();
            assert_eq!(outcome, Some((4, 2)), "space {space:?}");
            assert_eq!(cell_count(&state, palette), 2, "space {space:?}");
            assert_eq!(state.validate(), Ok(()), "space {space:?}");
        }
    }
}
