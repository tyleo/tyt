use crate::utilities::{QuantizePoint, point_bounds};
use std::cmp::Ordering;

/// Splits `boxes` into at most `target` clusters by median cut. Boxes only
/// split, so points of different seed boxes never share a cluster.
pub fn median_cut(mut boxes: Vec<Vec<QuantizePoint>>, target: usize) -> Vec<Vec<QuantizePoint>> {
    while boxes.len() < target {
        // The splittable box (two or more distinct points) with the widest
        // single axis.
        let mut widest: Option<(usize, usize, f64)> = None;

        for (index, point_box) in boxes.iter().enumerate() {
            if point_box.len() < 2 {
                continue;
            }

            let (axis, extent) = longest_axis(point_box);
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

        let mut point_box = boxes.swap_remove(index);

        point_box.sort_by(|a, b| {
            a.coords[axis]
                .partial_cmp(&b.coords[axis])
                .unwrap_or(Ordering::Equal)
        });

        // The boundary between distinct values nearest the median, so points
        // sharing a value on the axis stay in one box. A positive extent
        // guarantees one.
        let median = point_box.len() / 2;
        let split = (1..point_box.len())
            .filter(|&index| point_box[index - 1].coords[axis] < point_box[index].coords[axis])
            .min_by_key(|&index| index.abs_diff(median))
            .expect("a box with extent has two distinct values on its widest axis");

        let high = point_box.split_off(split);

        boxes.push(point_box);
        boxes.push(high);
    }

    boxes
}

/// The axis of widest spread in a box, and that spread.
fn longest_axis(point_box: &[QuantizePoint]) -> (usize, f64) {
    let (low, high) = point_bounds(point_box);

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

#[cfg(test)]
mod tests {
    use crate::utilities::{QuantizePoint, median_cut};
    use branded_id::ext::IteratorExt;
    use ty_math::TyVector4F64;

    /// One box of 1D points at `values`, material ids in order.
    fn points(values: &[f64]) -> Vec<QuantizePoint> {
        values
            .iter()
            .enumerate_ids()
            .map(|(material_id, &value)| QuantizePoint {
                material_id,
                coords: TyVector4F64::new(value, 0.0, 0.0, 0.0),
                population: 1,
            })
            .collect()
    }

    /// Each cluster's material ids, sorted, with the clusters sorted.
    fn ids(clusters: Vec<Vec<QuantizePoint>>) -> Vec<Vec<u32>> {
        let mut ids: Vec<Vec<u32>> = clusters
            .into_iter()
            .map(|cluster| {
                let mut ids: Vec<_> = cluster
                    .into_iter()
                    .map(|point| point.material_id.to_u32())
                    .collect();
                ids.sort();
                ids
            })
            .collect();
        ids.sort();
        ids
    }

    #[test]
    fn equal_values_never_part() {
        // The count median falls inside the run of zeros.
        let clusters = median_cut(vec![points(&[0.0, 1.0, 0.0, 0.0])], 2);
        assert_eq!(ids(clusters), [vec![0, 2, 3], vec![1]]);
    }

    #[test]
    fn seed_boxes_stay_apart() {
        let clusters = median_cut(vec![points(&[0.0]), points(&[0.0, 5.0])], 2);
        assert_eq!(clusters.len(), 2);
    }
}
