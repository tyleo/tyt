use crate::utilities::QuantizePoint;
use ty_math::TyVector4F64;

/// Partitions `points` into at most `target` clusters by k-means, seeded by
/// farthest point so no randomness enters. Empty clusters drop, so the result
/// may hold fewer than `target`.
pub fn kmeans(points: Vec<QuantizePoint>, target: usize) -> Vec<Vec<QuantizePoint>> {
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

        let mut sum = vec![TyVector4F64::ZERO; centroids.len()];

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

    let mut clusters: Vec<Vec<QuantizePoint>> = vec![Vec::new(); centroids.len()];

    for (index, point) in points.into_iter().enumerate() {
        clusters[assignment[index]].push(point);
    }

    clusters.retain(|cluster| !cluster.is_empty());

    clusters
}

/// The index of the nearest centroid to `coords`, ties to the lowest index.
fn nearest_centroid(coords: TyVector4F64, centroids: &[TyVector4F64]) -> usize {
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
