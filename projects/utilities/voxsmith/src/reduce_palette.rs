use crate::{BASE_COLOR, ColorSpace, Dither, ReductionMethod, Result, value_pool_color};
use branded_id::U32Id;
use std::{cmp::Ordering, collections::HashMap, mem};
use ty_math::{
    FromColor, TyCielabColorF64, TyColorToVector3, TyLinSrgbaF64, TyOklabColorF64, TySrgbaU8,
    TyVector3Ext, TyVector3F64, TyVector3U32,
};
use voxcore::{BVoxLayer, BVoxMaterial, BVoxObject, BVoxPalette, BVoxProperty, VoxMain};

/// Reduces `palette` in `state` to at most `max_materials` materials, then,
/// unless `keep_unused_values`, prunes the value-pool values the reduction
/// leaves unreferenced. The prune runs state-wide through
/// [`VoxMain::prune_value_pools`]. Either way the state stays valid, with the
/// surviving value ids left for [`VoxMain::gc`] to compact.
///
/// Materials cluster by the `baseColor` property and each cluster
/// collapses onto one real representative, so a merged voxel takes the
/// representative's whole material. Colorless materials are left untouched.
/// Returns `Some((before, after))` when the reduction fired, `None` when the
/// palette already fit.
///
/// # Arguments
/// * `state` - the document, reduced in place.
/// * `palette_id` - the palette to reduce; every referencing object is
///   remapped.
/// * `max_materials` - the material cap.
/// * `method` - the clustering algorithm.
/// * `space` - the color space compared in.
/// * `dither` - error diffusion when snapping samples.
/// * `keep_unused_values` - keep value-pool values left unreferenced.
pub fn reduce_palette(
    state: &mut VoxMain,
    palette_id: U32Id<BVoxPalette>,
    max_materials: usize,
    method: ReductionMethod,
    space: ColorSpace,
    dither: Dither,
    keep_unused_values: bool,
) -> Result<Option<(usize, usize)>> {
    let outcome = reduce_materials(state, palette_id, max_materials, method, space, dither)?;
    if !keep_unused_values {
        state.prune_value_pools();
    }
    Ok(outcome)
}

/// Reduces `palette` in `state` to at most `max_materials` materials:
/// materials cluster by `baseColor` and each cluster collapses onto one
/// real representative, so a merged voxel takes the representative's whole
/// material. Colorless materials are left untouched.
///
/// Returns `Some((before, after))` when the reduction fired, `None` when the
/// palette already fit (leaving `method` / `space` / `dither` inert). The state
/// is left compacted and valid.
fn reduce_materials(
    state: &mut VoxMain,
    palette_id: U32Id<BVoxPalette>,
    max_materials: usize,
    method: ReductionMethod,
    space: ColorSpace,
    dither: Dither,
) -> Result<Option<(usize, usize)>> {
    // A missing palette is a caller bug.
    let palette_ref = state
        .palette(palette_id)
        .expect("reduce_palette was given a palette not in the state");

    let total = palette_ref.material_count();
    if total <= max_materials {
        return Ok(None);
    }

    // The `baseColor` property; only colored materials cluster.
    let color_property_id = palette_ref.property_id_by_name(BASE_COLOR);

    let colored: Vec<(u32, [u8; 4])> = match color_property_id {
        Some(property_id) => palette_ref
            .iter_materials()
            .filter_map(|material_id| {
                let color = material_color(state, palette_id, material_id, property_id)?;
                Some((material_id.to_u32(), color))
            })
            .collect(),
        None => Vec::new(),
    };

    let survivors = total - colored.len();

    if colored.is_empty() {
        return Ok(None);
    }

    // Tally per-material voxel usage and place each colored material in the
    // space.
    let populations = material_populations(state, palette_id);

    let points: Vec<Point> = colored
        .into_iter()
        .map(|(material_id, color)| Point {
            material_id,
            coords: to_space(color, space),
            population: populations.get(&material_id).copied().unwrap_or(0),
        })
        .collect();

    let target = max_materials.saturating_sub(survivors).max(1);

    let clusters = match method {
        ReductionMethod::MedianCut => median_cut(points, target),
        ReductionMethod::Octree => octree(points, target),
        ReductionMethod::Kmeans => kmeans(points, target),
    };

    let after = clusters.len() + survivors;

    // With the palette chosen, dithering is the per-voxel remap onto it: each
    // voxel snaps individually, spreading one merged color across several
    // representatives.
    if !matches!(dither, Dither::None) {
        dither_voxels(state, palette_id, &clusters, dither);
    }

    // Drop every non-representative material onto its representative, then
    // compact. After dithering no voxel samples a non-representative, so the
    // repaint is a no-op and only the drop remains. One batched removal keeps
    // the repaint to a single pass over the voxels, rather than one per
    // dropped material.
    let replacements: HashMap<_, _> = clusters
        .iter()
        .flat_map(|cluster| {
            let representative_id = representative_id(cluster);
            cluster
                .iter()
                .filter(move |point| point.material_id != representative_id)
                .map(move |point| {
                    (
                        U32Id::from_u32(point.material_id),
                        U32Id::from_u32(representative_id),
                    )
                })
        })
        .collect();

    state
        .remove_materials(palette_id, &replacements)
        .expect("the cluster's materials are live and distinct");

    state.gc();

    Ok(Some((total, after)))
}

/// A palette material as a clustering point: id, color in the working space,
/// and live-voxel sample count.
#[derive(Clone, Copy)]
struct Point {
    material_id: u32,

    coords: TyVector3F64,

    population: u64,
}

/// The sRGB bytes of a material's `baseColor`, or `None` if the value is
/// absent or its bound value pool holds no float vectors. Resolves the bound
/// value-pool value and encodes it with [`value_pool_color`].
fn material_color(
    state: &VoxMain,
    palette_id: U32Id<BVoxPalette>,
    material_id: U32Id<BVoxMaterial>,
    property_id: U32Id<BVoxProperty>,
) -> Option<[u8; 4]> {
    let (value_pool, value_id) = state.material_value(palette_id, material_id, property_id)?;

    value_pool_color(value_pool, value_id)
}

/// How many live voxels sample each material of `palette`, across every object
/// referencing it.
fn material_populations(state: &VoxMain, palette_id: U32Id<BVoxPalette>) -> HashMap<u32, u64> {
    let mut populations = HashMap::new();

    for (_, object) in state.iter_objects() {
        for (layer_id, referenced_palette_id) in object.iter_layers() {
            if referenced_palette_id != palette_id {
                continue;
            }

            for voxel_id in object.iter_live() {
                if let Some(material_id) = object.voxel_material(voxel_id, layer_id) {
                    *populations.entry(material_id.to_u32()).or_insert(0) += 1;
                }
            }
        }
    }
    populations
}

/// A cluster's representative material: the most-sampled, ties to the lowest
/// id.
fn representative_id(cluster: &[Point]) -> u32 {
    representative_point(cluster).material_id
}

/// The representative (see [`representative_id`]) as a whole [`Point`], for the
/// dither pass's snap target.
fn representative_point(cluster: &[Point]) -> Point {
    cluster
        .iter()
        .copied()
        .max_by(|a, b| {
            a.population
                .cmp(&b.population)
                .then_with(|| b.material_id.cmp(&a.material_id))
        })
        .expect("a cluster holds at least one point")
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

        for (index, color_box) in boxes.iter().enumerate() {
            if color_box.len() < 2 {
                continue;
            }

            let (axis, extent) = longest_axis(color_box);
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

        let mut color_box = boxes.swap_remove(index);

        color_box.sort_by(|a, b| {
            a.coords[axis]
                .partial_cmp(&b.coords[axis])
                .unwrap_or(Ordering::Equal)
        });

        let high = color_box.split_off(color_box.len() / 2);

        boxes.push(color_box);
        boxes.push(high);
    }

    boxes
}

/// The axis of widest spread in a box, and that spread.
fn longest_axis(color_box: &[Point]) -> (usize, f64) {
    let (low, high) = point_bounds(color_box);

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
/// build a fixed-depth octree over the color cube, then fold the least-populated
/// all-leaf node into one leaf until at most `target` leaves remain, so the
/// rarest colors merge first.
fn octree(points: Vec<Point>, target: usize) -> Vec<Vec<Point>> {
    const DEPTH: u32 = 8;
    const BUCKETS: u32 = 1 << DEPTH;

    // The bucketing box; a color's per-axis bucket in `[0, BUCKETS)` spells its
    // octree path. It covers the point set, since oklab and lab axes are signed.
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

/// Partitions `points` into at most `target` clusters by k-means: seed by
/// farthest-point (deterministic, no random init), then alternate assignment and
/// population-weighted centroid updates until settled or a step cap. Empty
/// clusters are dropped, so the result may hold fewer than `target`.
fn kmeans(points: Vec<Point>, target: usize) -> Vec<Vec<Point>> {
    const MAX_STEPS: usize = 32;

    let k = target.min(points.len()).max(1);

    // Seed 0 is the most-sampled point (ties to the lowest material); each next
    // seed is the point farthest from the seeds so far.
    let first = points
        .iter()
        .max_by(|a, b| {
            a.population
                .cmp(&b.population)
                .then_with(|| b.material_id.cmp(&a.material_id))
        })
        .expect("kmeans is given at least one point");

    let mut centroids = vec![first.coords];

    while centroids.len() < k {
        let mut best: Option<(usize, f64)> = None;

        for (index, point) in points.iter().enumerate() {
            let nearest = centroids
                .iter()
                .map(|centroid| (point.coords - *centroid).length_squared())
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

            sum[cluster] += point.coords * w;

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
        low = low.min(point.coords);
        high = high.max(point.coords);
    }

    (low, high)
}

/// The index of the nearest centroid to `coords`, ties to the lowest index.
fn nearest_centroid(coords: TyVector3F64, centroids: &[TyVector3F64]) -> usize {
    let mut best = 0;
    let mut best_distance = f64::INFINITY;

    for (index, centroid) in centroids.iter().enumerate() {
        let distance = (coords - *centroid).length_squared();
        if distance < best_distance {
            best_distance = distance;
            best = index;
        }
    }

    best
}

/// An sRGB byte color as a point in `space`; alpha is dropped. `srgb` uses the
/// stored sRGB bytes; `oklab` and `lab` decode to linear light first.
fn to_space(rgba: [u8; 4], space: ColorSpace) -> TyVector3F64 {
    let srgb = TySrgbaU8::from(rgba).into_format::<f64, f64>();

    match space {
        ColorSpace::Srgb => srgb.color.to_vector3(),
        ColorSpace::Oklab => {
            let linear: TyLinSrgbaF64 = srgb.into_linear();
            TyOklabColorF64::from_color(linear).to_vector3()
        }
        ColorSpace::Lab => {
            let linear: TyLinSrgbaF64 = srgb.into_linear();
            TyCielabColorF64::from_color(linear).to_vector3()
        }
    }
}

/// Snaps every live voxel sampling `palette` to a representative, diffusing the
/// error per `dither`, so one merged color dithers across several
/// representatives. Runs per referencing object in raster order.
fn dither_voxels(
    state: &mut VoxMain,
    palette_id: U32Id<BVoxPalette>,
    clusters: &[Vec<Point>],
    dither: Dither,
) {
    // Cluster coords are already in the working space; read each material's
    // color and the representatives off the clusters once, shared across
    // objects.
    let coords_of: HashMap<u32, TyVector3F64> = clusters
        .iter()
        .flat_map(|cluster| cluster.iter())
        .map(|point| (point.material_id, point.coords))
        .collect();

    let representatives: Vec<Point> = clusters.iter().map(|c| representative_point(c)).collect();

    let spacing = palette_spacing(&representatives);

    // Objects referencing the palette, collected so the read borrow ends before
    // the mutation below.
    let targets: Vec<(U32Id<BVoxObject>, Vec<U32Id<BVoxLayer>>)> = state
        .iter_objects()
        .filter_map(|(object_id, object)| {
            let layer_ids: Vec<_> = object
                .iter_layers()
                .filter(|&(_, referenced_palette_id)| referenced_palette_id == palette_id)
                .map(|(layer_id, _)| layer_id)
                .collect();
            (!layer_ids.is_empty()).then_some((object_id, layer_ids))
        })
        .collect();

    for (object_id, layer_ids) in targets {
        for layer_id in layer_ids {
            dither_layer(
                state,
                object_id,
                layer_id,
                &coords_of,
                &representatives,
                spacing,
                dither,
            );
        }
    }
}

/// Dithers one layer on one object: walk live voxels in raster order, snap each
/// to the nearest representative, and reassign via
/// [`VoxMain::retain_voxel`], swapping only this layer's material.
#[allow(clippy::too_many_arguments)]
fn dither_layer(
    state: &mut VoxMain,
    object_id: U32Id<BVoxObject>,
    layer_id: U32Id<BVoxLayer>,
    coords_of: &HashMap<u32, TyVector3F64>,
    representatives: &[Point],
    spacing: f64,
    dither: Dither,
) {
    let object = state
        .object(object_id)
        .expect("a referencing object is one of the state's");
    let bounds = object.bounds();

    // Live voxels ascend by raster id, so every diffusion target is a
    // not-yet-visited voxel.
    let voxel_ids: Vec<_> = object.iter_live().collect();

    // Reassignment rewrites the full sample row, so find the layer order and
    // this layer's slot once.
    let layer_ids: Vec<_> = object.iter_layers().map(|(layer_id, _)| layer_id).collect();
    let slot_index = layer_ids
        .iter()
        .position(|&other_layer_id| other_layer_id == layer_id)
        .expect("the layer is one of the object's");

    // Floyd-Steinberg's sparse per-voxel error; ordered needs no buffer.
    let mut errors: HashMap<u32, TyVector3F64> = HashMap::new();

    for voxel_id in voxel_ids {
        let object = state
            .object(object_id)
            .expect("a referencing object is one of the state's");
        let material_id = object
            .voxel_material(voxel_id, layer_id)
            .expect("a live voxel samples every layer");

        // A colorless survivor was never clustered, so it has no color to snap.
        let Some(&original) = coords_of.get(&material_id.to_u32()) else {
            continue;
        };

        let position = object
            .voxel_position(voxel_id)
            .expect("a live voxel is within the grid");

        let target = match dither {
            Dither::FloydSteinberg => {
                original
                    + errors
                        .get(&voxel_id.to_u32())
                        .copied()
                        .unwrap_or(TyVector3F64::ZERO)
            }
            Dither::Ordered => original + ordered_offset(position, spacing),
            Dither::None => original,
        };

        let chosen = nearest_representative(target, representatives);

        if matches!(dither, Dither::FloydSteinberg) {
            diffuse_error(&mut errors, bounds, position, target - chosen.coords);
        }

        if chosen.material_id != material_id.to_u32() {
            let mut row: Vec<_> = layer_ids
                .iter()
                .map(|&layer_id| {
                    object
                        .voxel_material(voxel_id, layer_id)
                        .expect("a live voxel samples every layer")
                })
                .collect();
            row[slot_index] = U32Id::from_u32(chosen.material_id);
            state
                .retain_voxel(object_id, voxel_id, &row)
                .expect("a live voxel takes a full-arity row of palette materials");
        }
    }
}

/// The representative nearest `coords` by Euclidean distance in the clustering
/// space, ties broken by the lowest material id so the snap is deterministic.
fn nearest_representative(coords: TyVector3F64, representatives: &[Point]) -> Point {
    representatives
        .iter()
        .copied()
        .min_by(|a, b| {
            (coords - a.coords)
                .length_squared()
                .partial_cmp(&(coords - b.coords).length_squared())
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.material_id.cmp(&b.material_id))
        })
        .expect("the palette reduces to at least one representative")
}

/// Pushes `error` to the three raster-forward neighbors: `+z` and `+y` take
/// `3/8` each, `+x` takes `2/8`. 2D Floyd-Steinberg's kernel has no 3D standard,
/// so this defines one; error past a grid edge is dropped.
fn diffuse_error(
    errors: &mut HashMap<u32, TyVector3F64>,
    bounds: TyVector3U32,
    position: TyVector3U32,
    error: TyVector3F64,
) {
    // Voxel id is the raster index x*Y*Z + y*Z + z, so a forward neighbor's id
    // shifts by one plane, row, or cell.
    let plane = bounds.y * bounds.z;
    let voxel_id = position.x * plane + position.y * bounds.z + position.z;

    let mut push = |carry: bool, neighbor_id: u32, weight: f64| {
        if carry {
            let slot = errors.entry(neighbor_id).or_insert(TyVector3F64::ZERO);
            *slot += error * weight;
        }
    };

    push(position.z + 1 < bounds.z, voxel_id + 1, 3.0 / 8.0);
    push(position.y + 1 < bounds.y, voxel_id + bounds.z, 3.0 / 8.0);
    push(position.x + 1 < bounds.x, voxel_id + plane, 2.0 / 8.0);
}

/// The mean nearest-neighbor distance between representatives, scaling the
/// ordered-dither threshold to about one palette step. Zero for a lone
/// representative, disabling the perturbation.
fn palette_spacing(representatives: &[Point]) -> f64 {
    if representatives.len() < 2 {
        return 0.0;
    }

    let mut total = 0.0;
    for (index, representative) in representatives.iter().enumerate() {
        let mut nearest = f64::INFINITY;
        for (other_index, other) in representatives.iter().enumerate() {
            if index != other_index {
                nearest = nearest.min((representative.coords - other.coords).length());
            }
        }
        total += nearest;
    }

    total / representatives.len() as f64
}

/// A per-axis ordered-dither offset from the 3D Bayer matrix, scaled to
/// `spacing`. Each axis reads a rotation of the position so the channels
/// decorrelate; a raw threshold maps to `[-0.5, 0.5) * spacing`.
fn ordered_offset(position: TyVector3U32, spacing: f64) -> TyVector3F64 {
    let level = |raw: u32| ((raw as f64 + 0.5) / BAYER_LEVELS as f64 - 0.5) * spacing;
    let (x, y, z) = (position.x, position.y, position.z);

    TyVector3F64::new(
        level(bayer(x, y, z)),
        level(bayer(y, z, x)),
        level(bayer(z, x, y)),
    )
}

/// Side of the 3D Bayer matrix, a power of two for the doubling recurrence.
const BAYER_SIDE: u32 = 4;

/// Distinct threshold levels in the matrix, `BAYER_SIDE^3`.
const BAYER_LEVELS: u32 = BAYER_SIDE * BAYER_SIDE * BAYER_SIDE;

/// The Bayer threshold at `(x, y, z)` in `[0, BAYER_LEVELS)`, tiling by
/// [`BAYER_SIDE`]. One doubling of a parity-ordered 2x2x2 base (the 3D analog of
/// `[[0, 2], [3, 1]]`): `M(p) = 8 * base(p mod 2) + base(p / 2 mod 2)`.
fn bayer(x: u32, y: u32, z: u32) -> u32 {
    // The cube corners permuted 0..8, even-parity before odd so successive
    // thresholds land far apart.
    const BASE: [[[u32; 2]; 2]; 2] = [[[0, 4], [5, 1]], [[6, 2], [3, 7]]];

    let base = |x: u32, y: u32, z: u32| BASE[(x % 2) as usize][(y % 2) as usize][(z % 2) as usize];

    8 * base(x, y, z) + base(x / 2, y / 2, z / 2)
}

#[cfg(test)]
mod tests {
    use crate::{
        BASE_COLOR, ColorSpace, Dither, ReductionMethod, lin_srgba_from_srgba_u8, reduce_palette,
        srgba_u8_from_lin_srgba,
    };
    use branded_id::U32Id;
    use ty_math::{TyLinSrgbaF64, TySrgbaU8, TyVector3U32};
    use voxcore::{
        BVoxMaterial, BVoxObject, BVoxPalette, BVoxValuePoolValue, VoxMain, VoxObject, VoxPalette,
        VoxValuePool, VoxValuePoolValueRef,
    };

    /// The branded value id `index`.
    fn value_id(index: usize) -> U32Id<BVoxValuePoolValue> {
        U32Id::from_u32(index as u32)
    }

    /// The linear-light components of a `#RRGGBBAA` hex string.
    fn linear_rgba(hex: &str) -> [f64; 4] {
        let digits = hex.strip_prefix('#').expect("a leading #");
        let byte = |index: usize| {
            u8::from_str_radix(&digits[index * 2..index * 2 + 2], 16).expect("two hex digits")
        };
        lin_srgba_from_srgba_u8(TySrgbaU8::new(byte(0), byte(1), byte(2), byte(3))).into()
    }

    /// The `#RRGGBBAA` hex of a material's `baseColor`, uppercase.
    fn material_hex(
        state: &VoxMain,
        palette_id: U32Id<BVoxPalette>,
        material_id: U32Id<BVoxMaterial>,
    ) -> String {
        let property_id = state
            .palette(palette_id)
            .unwrap()
            .property_id_by_name(BASE_COLOR)
            .unwrap();
        match state
            .material_value(palette_id, material_id, property_id)
            .and_then(|(value_pool, value_id)| value_pool.value(value_id))
        {
            Some(VoxValuePoolValueRef::Vec4Float(&color)) => hex_of(color),
            other => panic!("material has no srgba baseColor: {other:?}"),
        }
    }

    /// A `#RRGGBBAA` uppercase hex string of a stored linear color.
    fn hex_of(color: [f64; 4]) -> String {
        let bytes = <[u8; 4]>::from(srgba_u8_from_lin_srgba(TyLinSrgbaF64::from(color)));
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            bytes[0], bytes[1], bytes[2], bytes[3]
        )
    }

    /// One object whose voxels sample a palette of `#RRGGBBAA` `colors`, each
    /// material with a distinct `tag` scalar so a merge's whole-material take
    /// is visible. Voxel `i` samples material `i`; `repeats[i]` adds extra
    /// voxels on material `i`.
    fn state_with_colors(
        colors: &[&str],
        repeats: &[usize],
    ) -> (VoxMain, U32Id<BVoxPalette>, U32Id<BVoxObject>) {
        let mut state = VoxMain::default();

        // baseColor draws from an sRGBA color value pool; tag draws from
        // an unbounded float value pool with one distinct value per material.
        let base_value_pool_id = state.add_value_pool(
            VoxValuePool::vec_4_float(colors.iter().map(|color| linear_rgba(color)).collect())
                .unwrap(),
        );
        let tag_value_pool_id = state.add_value_pool(
            VoxValuePool::float((0..colors.len()).map(|index| index as f64).collect()).unwrap(),
        );

        let mut palette = VoxPalette::default();
        palette
            .add_property(
                BASE_COLOR.to_owned(),
                base_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        palette
            .add_property("tag".to_owned(), tag_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        let material_ids: Vec<_> = (0..colors.len())
            .map(|index| {
                palette
                    .add_material(vec![value_id(index), value_id(index)])
                    .unwrap()
            })
            .collect();

        let count: usize = colors.len() + repeats.iter().sum::<usize>();
        let mut object =
            VoxObject::new("o".to_owned(), TyVector3U32::new(count as u32, 1, 1)).unwrap();
        let palette_id = state.add_palette(palette).unwrap();
        object.add_layer(palette_id, material_ids[0]);

        // One voxel per color, plus `repeats[i]` extra voxels sampling material
        // i.
        let mut voxel_id = 0u32;
        let mut retain = |object: &mut VoxObject, material_id| {
            object
                .retain_voxel(U32Id::from_u32(voxel_id), &[material_id])
                .unwrap();
            voxel_id += 1;
        };
        for (index, &material_id) in material_ids.iter().enumerate() {
            retain(&mut object, material_id);
            for _ in 0..repeats.get(index).copied().unwrap_or(0) {
                retain(&mut object, material_id);
            }
        }
        let object_id = state.add_object(object).unwrap();
        (state, palette_id, object_id)
    }

    fn material_count(state: &VoxMain, palette_id: U32Id<BVoxPalette>) -> usize {
        state.palette(palette_id).unwrap().material_count()
    }

    /// The set of baseColor hexes still in the palette.
    fn colors(state: &VoxMain, palette_id: U32Id<BVoxPalette>) -> Vec<String> {
        state
            .palette(palette_id)
            .unwrap()
            .iter_materials()
            .map(|material_id| material_hex(state, palette_id, material_id))
            .collect()
    }

    /// One object of size `bounds` with a `baseColor` palette of `colors`
    /// and one live voxel per `(position, color-index)` entry, so a dither test
    /// can place a known color at a known position.
    fn grid_state(
        bounds: TyVector3U32,
        colors: &[&str],
        voxels: &[(TyVector3U32, usize)],
    ) -> (VoxMain, U32Id<BVoxPalette>, U32Id<BVoxObject>) {
        let mut state = VoxMain::default();

        let base_value_pool_id = state.add_value_pool(
            VoxValuePool::vec_4_float(colors.iter().map(|color| linear_rgba(color)).collect())
                .unwrap(),
        );

        let mut palette = VoxPalette::default();
        palette
            .add_property(
                BASE_COLOR.to_owned(),
                base_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        let material_ids: Vec<_> = (0..colors.len())
            .map(|index| palette.add_material(vec![value_id(index)]).unwrap())
            .collect();

        let mut object = VoxObject::new("o".to_owned(), bounds).unwrap();
        let palette_id = state.add_palette(palette).unwrap();
        object.add_layer(palette_id, material_ids[0]);

        for &(position, color_index) in voxels {
            let voxel_id = object.voxel_id(position).unwrap();
            object
                .retain_voxel(voxel_id, &[material_ids[color_index]])
                .unwrap();
        }

        let object_id = state.add_object(object).unwrap();
        (state, palette_id, object_id)
    }

    /// The baseColor hex the voxel at `position` samples, through the
    /// object's one layer.
    fn voxel_color(
        state: &VoxMain,
        object_id: U32Id<BVoxObject>,
        palette_id: U32Id<BVoxPalette>,
        position: TyVector3U32,
    ) -> String {
        let object = state.object(object_id).unwrap();
        let (layer_id, _) = object.iter_layers().next().unwrap();
        let voxel_id = object.voxel_id(position).unwrap();
        let material_id = object.voxel_material(voxel_id, layer_id).unwrap();
        material_hex(state, palette_id, material_id)
    }

    #[test]
    fn no_op_when_already_within_the_cap() {
        let (mut state, palette_id, _) = state_with_colors(&["#FF0000FF", "#00FF00FF"], &[]);
        let outcome = reduce_palette(
            &mut state,
            palette_id,
            5,
            ReductionMethod::MedianCut,
            ColorSpace::Oklab,
            Dither::None,
            false,
        )
        .unwrap();
        assert_eq!(outcome, None);
        assert_eq!(material_count(&state, palette_id), 2);
    }

    #[test]
    fn merges_near_colors_and_keeps_a_real_representative_material() {
        // Two near reds and a blue; cap 2 fuses the reds onto the more-sampled
        // second red, whose whole material (tag 1) survives.
        let (mut state, palette_id, object_id) =
            state_with_colors(&["#FE0000FF", "#FF0000FF", "#0000FFFF"], &[0, 3, 0]);
        let outcome = reduce_palette(
            &mut state,
            palette_id,
            2,
            ReductionMethod::MedianCut,
            ColorSpace::Oklab,
            Dither::None,
            false,
        )
        .unwrap();
        assert_eq!(outcome, Some((3, 2)));
        assert_eq!(material_count(&state, palette_id), 2);
        assert_eq!(state.validate(), Ok(()));

        let mut survivors = colors(&state, palette_id);
        survivors.sort();
        assert_eq!(survivors, ["#0000FFFF", "#FF0000FF"]);

        // The fused first red's voxel now samples the survivor, tag 1.
        let object = state.object(object_id).unwrap();
        let (layer_id, _) = object.iter_layers().next().unwrap();
        let palette_ref = state.palette(palette_id).unwrap();
        let tag_property_id = palette_ref.property_id_by_name("tag").unwrap();
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        let material_id = object.voxel_material(voxel_id, layer_id).unwrap();
        match state
            .material_value(palette_id, material_id, tag_property_id)
            .and_then(|(value_pool, value_id)| value_pool.value(value_id))
        {
            Some(VoxValuePoolValueRef::Float(number)) => assert_eq!(number, 1.0),
            other => panic!("unexpected tag value {other:?}"),
        }
    }

    #[test]
    fn octree_and_kmeans_reduce_to_the_cap() {
        // Both methods cluster four colors down to at most the cap of two.
        for method in [ReductionMethod::Octree, ReductionMethod::Kmeans] {
            let (mut state, palette_id, _) =
                state_with_colors(&["#FF0000FF", "#00FF00FF", "#0000FFFF", "#FFFF00FF"], &[]);
            let outcome = reduce_palette(
                &mut state,
                palette_id,
                2,
                method,
                ColorSpace::Oklab,
                Dither::None,
                false,
            )
            .unwrap();
            let (before, after) = outcome.expect("the reduction fired");
            assert_eq!(before, 4, "method {method:?}");
            assert!(
                (1..=2).contains(&after),
                "method {method:?} left {after} materials"
            );
            assert_eq!(
                material_count(&state, palette_id),
                after,
                "method {method:?}"
            );
            assert_eq!(state.validate(), Ok(()), "method {method:?}");
        }
    }

    #[test]
    fn dither_is_inert_under_the_cap_and_reduces_over_it() {
        // Under the cap: dither is inert, so the reduction does not fire.
        let (mut state, palette_id, _) = state_with_colors(&["#FF0000FF", "#00FF00FF"], &[]);
        assert!(
            reduce_palette(
                &mut state,
                palette_id,
                5,
                ReductionMethod::MedianCut,
                ColorSpace::Oklab,
                Dither::FloydSteinberg,
                false
            )
            .unwrap()
            .is_none()
        );

        // Over the cap: both dither methods reduce and leave the state valid.
        for dither in [Dither::FloydSteinberg, Dither::Ordered] {
            let (mut state, palette_id, _) =
                state_with_colors(&["#FF0000FF", "#00FF00FF", "#0000FFFF"], &[]);
            let outcome = reduce_palette(
                &mut state,
                palette_id,
                2,
                ReductionMethod::MedianCut,
                ColorSpace::Oklab,
                dither,
                false,
            )
            .unwrap();
            assert_eq!(outcome, Some((3, 2)), "dither {dither:?}");
            assert_eq!(material_count(&state, palette_id), 2, "dither {dither:?}");
            assert_eq!(state.validate(), Ok(()), "dither {dither:?}");
        }
    }

    #[test]
    fn reduces_across_all_color_spaces() {
        for space in [ColorSpace::Oklab, ColorSpace::Lab, ColorSpace::Srgb] {
            let (mut state, palette_id, _) =
                state_with_colors(&["#FF0000FF", "#00FF00FF", "#0000FFFF", "#FFFF00FF"], &[]);
            let outcome = reduce_palette(
                &mut state,
                palette_id,
                2,
                ReductionMethod::MedianCut,
                space,
                Dither::None,
                false,
            )
            .unwrap();
            assert_eq!(outcome, Some((4, 2)), "space {space:?}");
            assert_eq!(material_count(&state, palette_id), 2, "space {space:?}");
            assert_eq!(state.validate(), Ok(()), "space {space:?}");
        }
    }

    #[test]
    fn ordered_dither_lands_a_known_pattern() {
        // rgb space, so coords are the #-bytes over 255: black (0,0,0), mid
        // #800000 (~0.502,0,0), red (1,0,0), all on x. The four mid voxels at z=0
        // are under test; the black and seven red seeds at z=1,2 make black and
        // red the reps (red outvotes mid 7 to 4, black stands alone).
        let black = "#000000FF";
        let mid = "#800000FF";
        let red = "#FF0000FF";
        let mut voxels = vec![
            (TyVector3U32::new(0, 0, 0), 1),
            (TyVector3U32::new(1, 0, 0), 1),
            (TyVector3U32::new(2, 0, 0), 1),
            (TyVector3U32::new(3, 0, 0), 1),
            (TyVector3U32::new(0, 0, 1), 0),
        ];
        for x in 1..4 {
            voxels.push((TyVector3U32::new(x, 0, 1), 2));
        }
        for x in 0..4 {
            voxels.push((TyVector3U32::new(x, 0, 2), 2));
        }
        let (mut state, palette_id, object_id) =
            grid_state(TyVector3U32::new(4, 1, 3), &[black, mid, red], &voxels);

        let outcome = reduce_palette(
            &mut state,
            palette_id,
            2,
            ReductionMethod::MedianCut,
            ColorSpace::Srgb,
            Dither::Ordered,
            false,
        )
        .unwrap();
        assert_eq!(outcome, Some((3, 2)));
        assert_eq!(state.validate(), Ok(()));

        // Reps differ only on x, so only the x offset decides: bayer(x,0,0) is
        // 0, 48, 6, 54 for x=0..3, below/above the midpoint 31.5, so mid snaps
        // black, red, black, red.
        let expected = [black, red, black, red];
        for (x, want) in expected.iter().enumerate() {
            let got = voxel_color(
                &state,
                object_id,
                palette_id,
                TyVector3U32::new(x as u32, 0, 0),
            );
            assert_eq!(&got, want, "x = {x}");
        }
    }

    #[test]
    fn floyd_steinberg_dither_lands_a_known_pattern() {
        // Same x-axis palette on a (1,1,10) line: the mid voxels are the highest
        // ids (z=6..9) so error diffuses only among them; the black and five red
        // seeds (z=0..5) snap to their own rep with zero residual. Red outvotes
        // mid 5 to 4, so black and red are the reps.
        let black = "#000000FF";
        let mid = "#800000FF";
        let red = "#FF0000FF";
        let mut voxels = vec![(TyVector3U32::new(0, 0, 0), 0)];
        for z in 1..6 {
            voxels.push((TyVector3U32::new(0, 0, z), 2));
        }
        for z in 6..10 {
            voxels.push((TyVector3U32::new(0, 0, z), 1));
        }
        let (mut state, palette_id, object_id) =
            grid_state(TyVector3U32::new(1, 1, 10), &[black, mid, red], &voxels);

        let outcome = reduce_palette(
            &mut state,
            palette_id,
            2,
            ReductionMethod::MedianCut,
            ColorSpace::Srgb,
            Dither::FloydSteinberg,
            false,
        )
        .unwrap();
        assert_eq!(outcome, Some((3, 2)));
        assert_eq!(state.validate(), Ok(()));

        // On a line only +z carries, at 3/8. Tracing mid = 0.502 from zero error:
        //   z=6: 0.502          -> red   (residual -0.498, carries -0.187)
        //   z=7: 0.502 - 0.187  -> black (residual  0.315, carries  0.118)
        //   z=8: 0.502 + 0.118  -> red   (residual -0.380, carries -0.142)
        //   z=9: 0.502 - 0.142  -> black
        let expected = [(6, red), (7, black), (8, red), (9, black)];
        for (z, want) in expected {
            let got = voxel_color(&state, object_id, palette_id, TyVector3U32::new(0, 0, z));
            assert_eq!(&got, want, "z = {z}");
        }
    }

    /// The number of values in the value pool the property `name` draws from in
    /// the first palette that carries it.
    fn value_pool_len(state: &VoxMain, name: &str) -> usize {
        for (_, palette) in state.iter_palettes() {
            if let Some(property_id) = palette.property_id_by_name(name) {
                let value_pool_id = palette.property(property_id).unwrap().value_pool_id;
                return state.value_pool(value_pool_id).unwrap().values_len();
            }
        }
        panic!("no palette carries {name}");
    }

    #[test]
    fn pruning_drops_the_merged_away_values() {
        // Three colors fused to two; with keep_unused_values off, the reduction
        // prunes the merged-away color and its tag from the value pools.
        let (mut state, palette_id, _) =
            state_with_colors(&["#FE0000FF", "#FF0000FF", "#0000FFFF"], &[0, 3, 0]);
        assert_eq!(value_pool_len(&state, BASE_COLOR), 3);

        let outcome = reduce_palette(
            &mut state,
            palette_id,
            2,
            ReductionMethod::MedianCut,
            ColorSpace::Oklab,
            Dither::None,
            false,
        )
        .unwrap();
        assert_eq!(outcome, Some((3, 2)));

        // Both value pools now carry only the two survivors' values.
        assert_eq!(value_pool_len(&state, BASE_COLOR), 2);
        assert_eq!(value_pool_len(&state, "tag"), 2);
        assert_eq!(state.validate(), Ok(()));
        // The survivors still resolve to their original colors.
        let mut survivors = colors(&state, palette_id);
        survivors.sort();
        assert_eq!(survivors, ["#0000FFFF", "#FF0000FF"]);
    }

    #[test]
    fn a_colorless_palette_is_left_untouched() {
        // The palette carries no `baseColor`, so every material counts
        // as colorless and the reduction no-ops even over the cap.
        let mut state = VoxMain::default();
        let tag_value_pool_id = state.add_value_pool(
            VoxValuePool::float((0..3).map(|index| index as f64).collect()).unwrap(),
        );

        let mut palette = VoxPalette::default();
        palette
            .add_property("tag".to_owned(), tag_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        let material_ids: Vec<_> = (0..3)
            .map(|index| palette.add_material(vec![value_id(index)]).unwrap())
            .collect();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(3, 1, 1)).unwrap();
        let palette_id = state.add_palette(palette).unwrap();
        object.add_layer(palette_id, material_ids[0]);
        for (x, &material_id) in material_ids.iter().enumerate() {
            let voxel_id = object.voxel_id(TyVector3U32::new(x as u32, 0, 0)).unwrap();
            object.retain_voxel(voxel_id, &[material_id]).unwrap();
        }
        state.add_object(object).unwrap();

        let outcome = reduce_palette(
            &mut state,
            palette_id,
            2,
            ReductionMethod::MedianCut,
            ColorSpace::Oklab,
            Dither::None,
            false,
        )
        .unwrap();

        assert_eq!(outcome, None);
        assert_eq!(material_count(&state, palette_id), 3);
        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn dithering_reduces_around_a_second_layer() {
        // A trailing layer over another palette; the dither rebuilds
        // full-arity sample rows around its slot and the reduction still
        // lands.
        let (mut state, palette_id, object_id) =
            state_with_colors(&["#FF0000FF", "#00FF00FF", "#0000FFFF"], &[]);
        let strength_value_pool_id = state.add_value_pool(VoxValuePool::float(vec![1.5]).unwrap());

        let mut glow_palette = VoxPalette::default();
        glow_palette
            .add_property(
                "emissiveStrength".to_owned(),
                strength_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        glow_palette.add_material(vec![value_id(0)]).unwrap();
        let glow_palette_id = state.add_palette(glow_palette).unwrap();
        state
            .add_layer(object_id, glow_palette_id, U32Id::from_u32(0))
            .unwrap();

        let outcome = reduce_palette(
            &mut state,
            palette_id,
            2,
            ReductionMethod::MedianCut,
            ColorSpace::Oklab,
            Dither::FloydSteinberg,
            false,
        )
        .unwrap();

        assert_eq!(outcome, Some((3, 2)));
        assert_eq!(material_count(&state, palette_id), 2);
        assert_eq!(state.validate(), Ok(()));

        // The trailing layer and its palette are untouched.
        let object = state.object(object_id).unwrap();
        assert_eq!(object.layer_count(), 2);
        assert_eq!(material_count(&state, glow_palette_id), 1);
    }

    #[test]
    fn keep_unused_values_leaves_the_value_pools_whole() {
        // The same reduction with keep_unused_values keeps the merged-away
        // color in the value pool, unreferenced.
        let (mut state, palette_id, _) =
            state_with_colors(&["#FE0000FF", "#FF0000FF", "#0000FFFF"], &[0, 3, 0]);
        reduce_palette(
            &mut state,
            palette_id,
            2,
            ReductionMethod::MedianCut,
            ColorSpace::Oklab,
            Dither::None,
            true,
        )
        .unwrap();
        assert_eq!(material_count(&state, palette_id), 2);
        assert_eq!(value_pool_len(&state, BASE_COLOR), 3);
        assert_eq!(state.validate(), Ok(()));
    }
}
