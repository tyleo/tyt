use crate::{ColorSpace, Dither, ReductionMethod, Result};
use branded_id::U32Id;
use std::{cmp::Ordering, collections::HashMap, mem};
use ty_math::{TySrgbaColor, TyVector3F64, TyVector3U32};
use voxcore::{BVoxObject, BVoxPalette, BVoxPaletteRef, VoxMain, VoxObject, VoxValue};

/// Reduces `palette` in `state` to at most `max_cells` cells: cells cluster by
/// `rgba` and each cluster collapses onto one real representative, so a merged
/// voxel takes the representative's whole row, not an average. Colorless cells
/// are left untouched.
///
/// Returns `Some((before, after))` when the reduction fired, `None` when the
/// palette already fit (leaving `method` / `space` / `dither` inert). The state
/// is left compacted and valid.
///
/// # Arguments
/// * `state` - the document, reduced in place.
/// * `palette` - the palette to reduce; every referencing object is remapped.
/// * `max_cells` - the cell cap.
/// * `method` - the clustering algorithm.
/// * `space` - the color space compared in.
/// * `dither` - error diffusion when snapping samples.
pub fn reduce_palette(
    state: &mut VoxMain,
    palette: U32Id<BVoxPalette>,
    max_cells: usize,
    method: ReductionMethod,
    space: ColorSpace,
    dither: Dither,
) -> Result<Option<(usize, usize)>> {
    // A missing palette is a caller bug, not a silent no-op.
    let palette_ref = state
        .palette(palette)
        .expect("reduce_palette was given a palette not in the state");

    let total = palette_ref.cell_count();
    if total <= max_cells {
        return Ok(None);
    }

    // The `rgba` attribute; colorless cells have nothing to cluster on.
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

    // Tally per-cell voxel usage and place each colored cell in the space.
    let populations = cell_populations(state, palette);

    let points: Vec<Point> = colored
        .into_iter()
        .map(|(cell, color)| Point {
            cell,
            coords: to_space(color, space),
            population: populations.get(&cell).copied().unwrap_or(0),
        })
        .collect();

    let target = max_cells.saturating_sub(survivors).max(1);

    let clusters = match method {
        ReductionMethod::MedianCut => median_cut(points, target),
        ReductionMethod::Octree => octree(points, target),
        ReductionMethod::Kmeans => kmeans(points, target),
    };

    let after = clusters.len() + survivors;

    // With the palette chosen, dithering is the per-voxel remap onto it: each
    // voxel snaps individually, spreading one merged color across several
    // representatives instead of collapsing onto one.
    if !matches!(dither, Dither::None) {
        dither_voxels(state, palette, &clusters, dither);
    }

    // Drop every non-representative cell onto its representative, then compact.
    // After dithering no voxel samples a non-representative, so the repaint is a
    // no-op and only the drop remains.
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

/// A palette cell as a clustering point: id, color in the working space, and
/// live-voxel sample count.
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

/// A cluster's representative cell: the most-sampled, ties to the lowest id.
fn representative(cluster: &[Point]) -> u32 {
    representative_point(cluster).cell
}

/// The representative (see [`representative`]) as a whole [`Point`], for the
/// dither pass's snap target.
fn representative_point(cluster: &[Point]) -> Point {
    cluster
        .iter()
        .copied()
        .max_by(|a, b| {
            a.population
                .cmp(&b.population)
                .then_with(|| b.cell.cmp(&a.cell))
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

/// Partitions `points` into at most `target` clusters by k-means: seed by
/// farthest-point (deterministic, no random init), then alternate assignment and
/// population-weighted centroid updates until settled or a step cap. Empty
/// clusters are dropped, so the result may hold fewer than `target`.
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

/// A cell's `rgba` as a point in `space`; alpha is dropped. `rgb` uses the
/// stored sRGB bytes; `oklab` and `lab` decode to linear light first.
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

    Some(TySrgbaColor::from_hex(hex)?.to_array())
}

/// Snaps every live voxel sampling `palette` to a representative, diffusing the
/// error per `dither`, so one merged color dithers across several
/// representatives. Runs per referencing object in raster order.
fn dither_voxels(
    state: &mut VoxMain,
    palette: U32Id<BVoxPalette>,
    clusters: &[Vec<Point>],
    dither: Dither,
) {
    // Cluster coords are already in the working space; read each cell's color
    // and the representatives off the clusters once, shared across objects.
    let coords_of: HashMap<u32, TyVector3F64> = clusters
        .iter()
        .flat_map(|cluster| cluster.iter())
        .map(|point| (point.cell, point.coords))
        .collect();

    let representatives: Vec<Point> = clusters.iter().map(|c| representative_point(c)).collect();

    let spacing = palette_spacing(&representatives);

    // Objects referencing the palette, collected so the read borrow ends before
    // the mutation below.
    let targets: Vec<(U32Id<BVoxObject>, Vec<U32Id<BVoxPaletteRef>>)> = state
        .iter_objects()
        .filter_map(|(object_id, object)| {
            let references: Vec<_> = object
                .iter_palette_refs()
                .filter(|&(_, referenced)| referenced == palette)
                .map(|(reference, _)| reference)
                .collect();
            (!references.is_empty()).then_some((object_id, references))
        })
        .collect();

    for (object_id, references) in targets {
        let object = state
            .object_mut(object_id)
            .expect("a referencing object is one of the state's");

        for reference in references {
            dither_reference(
                object,
                reference,
                &coords_of,
                &representatives,
                spacing,
                dither,
            );
        }
    }
}

/// Dithers one palette reference on one object: walk live voxels in raster
/// order, snap each to the nearest representative, and reassign via
/// `retain_voxel`, swapping only this reference's cell.
fn dither_reference(
    object: &mut VoxObject,
    reference: U32Id<BVoxPaletteRef>,
    coords_of: &HashMap<u32, TyVector3F64>,
    representatives: &[Point],
    spacing: f64,
    dither: Dither,
) {
    let bounds = object.bounds();

    // Live voxels ascend by raster id, so every diffusion target is a
    // not-yet-visited voxel.
    let voxels: Vec<_> = object.iter_live().collect();

    // Reassignment rewrites the full row, so find the reference order and this
    // reference's slot once.
    let references: Vec<_> = object.iter_palette_refs().map(|(id, _)| id).collect();
    let slot = references
        .iter()
        .position(|&id| id == reference)
        .expect("the reference is one of the object's");

    // Floyd-Steinberg's sparse per-voxel error; ordered needs no buffer.
    let mut errors: HashMap<u32, TyVector3F64> = HashMap::new();

    for voxel in voxels {
        let cell = object
            .voxel_cell(voxel, reference)
            .expect("a live voxel samples every reference");

        // A colorless survivor was never clustered, so it has no color to snap.
        let Some(&original) = coords_of.get(&cell.to_u32()) else {
            continue;
        };

        let position = object
            .voxel_position(voxel)
            .expect("a live voxel is within the grid");

        let target = match dither {
            Dither::FloydSteinberg => {
                original
                    + errors
                        .get(&voxel.to_u32())
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

        if chosen.cell != cell.to_u32() {
            let mut row: Vec<_> = references
                .iter()
                .map(|&id| {
                    object
                        .voxel_cell(voxel, id)
                        .expect("a live voxel samples every reference")
                })
                .collect();
            row[slot] = U32Id::from_u32(chosen.cell);
            object
                .retain_voxel(voxel, &row)
                .expect("a live voxel takes a full-arity row");
        }
    }
}

/// The representative nearest `coords` by Euclidean distance in the clustering
/// space, ties broken by the lowest cell id so the snap is deterministic.
fn nearest_representative(coords: TyVector3F64, representatives: &[Point]) -> Point {
    representatives
        .iter()
        .copied()
        .min_by(|a, b| {
            (coords - a.coords)
                .magnitude_squared()
                .partial_cmp(&(coords - b.coords).magnitude_squared())
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.cell.cmp(&b.cell))
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
    let id = position.x * plane + position.y * bounds.z + position.z;

    let mut push = |carry: bool, neighbor: u32, weight: f64| {
        if carry {
            let slot = errors.entry(neighbor).or_insert(TyVector3F64::ZERO);
            *slot = *slot + error * weight;
        }
    };

    push(position.z + 1 < bounds.z, id + 1, 3.0 / 8.0);
    push(position.y + 1 < bounds.y, id + bounds.z, 3.0 / 8.0);
    push(position.x + 1 < bounds.x, id + plane, 2.0 / 8.0);
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
                nearest = nearest.min((representative.coords - other.coords).magnitude());
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
    use crate::{ColorSpace, Dither, ReductionMethod, reduce_palette};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{BVoxObject, BVoxPalette, VoxMain, VoxObject, VoxPalette, VoxValue};

    /// One object whose voxels sample a palette of `#RRGGBBAA` `colors`, each
    /// with a distinct `tag` scalar so a merge's whole-row take is visible. Voxel
    /// `i` samples cell `i`; `repeats[i]` adds extra voxels on cell `i`.
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

    /// One object of size `bounds` with an `rgba` palette of `colors` and one
    /// live voxel per `(position, color-index)` entry, so a dither test can place
    /// a known color at a known position.
    fn grid_state(
        bounds: TyVector3U32,
        colors: &[&str],
        voxels: &[(TyVector3U32, usize)],
    ) -> (VoxMain, U32Id<BVoxPalette>, U32Id<BVoxObject>) {
        let mut palette = VoxPalette::default();
        palette.add_attribute("rgba".to_owned());
        let cells: Vec<_> = colors
            .iter()
            .map(|color| {
                palette
                    .add_cell(vec![VoxValue::Text((*color).to_owned())])
                    .unwrap()
            })
            .collect();

        let mut object = VoxObject::new("o".to_owned(), bounds).unwrap();
        let mut state = VoxMain::default();
        let palette_id = state.add_palette(palette);
        object.add_palette_ref(palette_id, cells[0]);

        for &(position, color) in voxels {
            let voxel = object.voxel_id(position).unwrap();
            object.retain_voxel(voxel, &[cells[color]]).unwrap();
        }

        let object_id = state.add_object(object);
        (state, palette_id, object_id)
    }

    /// The `rgba` hex the voxel at `position` samples, through the object's one
    /// palette reference.
    fn voxel_color(
        state: &VoxMain,
        object: U32Id<BVoxObject>,
        palette: U32Id<BVoxPalette>,
        position: TyVector3U32,
    ) -> String {
        let object = state.object(object).unwrap();
        let (reference, _) = object.iter_palette_refs().next().unwrap();
        let voxel = object.voxel_id(position).unwrap();
        let cell = object.voxel_cell(voxel, reference).unwrap();
        let palette = state.palette(palette).unwrap();
        let (rgba, _) = palette
            .iter_attributes()
            .find(|(_, name)| *name == "rgba")
            .unwrap();
        match palette.cell_value(cell, rgba) {
            Some(VoxValue::Text(hex)) => hex.clone(),
            other => panic!("cell has no rgba text: {other:?}"),
        }
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
        // Two near reds and a blue; cap 2 fuses the reds onto the more-sampled
        // second red, whose whole row (tag 1) survives, not an average.
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

        // The fused first red's voxel now samples the survivor, tag 1.
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
    fn dither_is_inert_under_the_cap_and_reduces_over_it() {
        // Under the cap: dither is inert, so the reduction does not fire.
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

        // Over the cap: both dither methods reduce and leave the state valid.
        for dither in [Dither::FloydSteinberg, Dither::Ordered] {
            let (mut state, palette, _) =
                state_with_colors(&["#FF0000FF", "#00FF00FF", "#0000FFFF"], &[]);
            let outcome = reduce_palette(
                &mut state,
                palette,
                2,
                ReductionMethod::MedianCut,
                ColorSpace::Oklab,
                dither,
            )
            .unwrap();
            assert_eq!(outcome, Some((3, 2)), "dither {dither:?}");
            assert_eq!(cell_count(&state, palette), 2, "dither {dither:?}");
            assert_eq!(state.validate(), Ok(()), "dither {dither:?}");
        }
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
        let (mut state, palette, object) =
            grid_state(TyVector3U32::new(4, 1, 3), &[black, mid, red], &voxels);

        let outcome = reduce_palette(
            &mut state,
            palette,
            2,
            ReductionMethod::MedianCut,
            ColorSpace::Rgb,
            Dither::Ordered,
        )
        .unwrap();
        assert_eq!(outcome, Some((3, 2)));
        assert_eq!(state.validate(), Ok(()));

        // Reps differ only on x, so only the x offset decides: bayer(x,0,0) is
        // 0, 48, 6, 54 for x=0..3, below/above the midpoint 31.5, so mid snaps
        // black, red, black, red.
        let expected = [black, red, black, red];
        for (x, want) in expected.iter().enumerate() {
            let got = voxel_color(&state, object, palette, TyVector3U32::new(x as u32, 0, 0));
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
        let (mut state, palette, object) =
            grid_state(TyVector3U32::new(1, 1, 10), &[black, mid, red], &voxels);

        let outcome = reduce_palette(
            &mut state,
            palette,
            2,
            ReductionMethod::MedianCut,
            ColorSpace::Rgb,
            Dither::FloydSteinberg,
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
            let got = voxel_color(&state, object, palette, TyVector3U32::new(0, 0, z));
            assert_eq!(&got, want, "z = {z}");
        }
    }
}
