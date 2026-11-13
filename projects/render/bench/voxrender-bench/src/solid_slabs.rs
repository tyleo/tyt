use ty_math::TyVector3U32;
use voxrender::RenderObject;
use voxrender_wgpu::dense_cells;

/// The most slabs an object gives.
const MOST: usize = 8;

/// The least cells a slab's widest face holds.
const LEAST: usize = 64;

/// The layers within which a slab stands in for a narrower one on its axis.
const NEAR: usize = 2;

/// The most rectangles one layer offers.
const OFFERS: usize = 4;

/// A box of cells as its low cell and the corner past its high cell.
type Slab = ([usize; 3], [usize; 3]);

/// Up to [`MOST`] boxes of `object`'s opaque cells alone, widest face
/// first, each as its low cell and the corner past its high cell. `opaque`
/// says whether a material id passes no light. What lies behind such a box
/// cannot show, which lets a view leave it out.
///
/// Each layer of cells across an axis offers its largest rectangles of
/// opaque cells. The widest offers win. One grows along its axis for as
/// long as the layers beside it fill its rectangle.
pub fn solid_slabs(
    object: &RenderObject,
    opaque: &dyn Fn(u32) -> bool,
) -> Vec<(TyVector3U32, TyVector3U32)> {
    let bounds = object.bounds().to_array().map(|side| side as usize);

    let solid: Vec<bool> = dense_cells(object)
        .iter()
        .map(|&value| value.checked_sub(1).is_some_and(opaque))
        .collect();

    // What a step along each axis adds to a cell's place in `solid`.
    let strides = [bounds[1] * bounds[2], bounds[2], 1];
    let at = |cell: [usize; 3]| solid[(0..3).map(|axis| cell[axis] * strides[axis]).sum::<usize>()];

    // The cell `across` of the layer `layer` of `axis`, whose two parts lie
    // along the axes after it.
    let cell = |axis: usize, layer: usize, across: [usize; 2]| {
        let mut cell = [0; 3];

        cell[axis] = layer;
        cell[(axis + 1) % 3] = across[0];
        cell[(axis + 2) % 3] = across[1];

        cell
    };

    let mut offers: Vec<(usize, usize, Slab)> = Vec::new();

    // The cells of a layer that no offer has taken, row by row.
    let mut left: Vec<bool> = Vec::new();

    for axis in 0..3 {
        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
        let size = [bounds[u], bounds[v]];

        for layer in 0..bounds[axis] {
            left.clear();

            for y in 0..size[1] {
                let row = layer * strides[axis] + y * strides[v];

                left.extend((0..size[0]).map(|x| solid[row + x * strides[u]]));
            }

            let mut count = left.iter().filter(|&&solid| solid).count();

            // Each offer takes its cells from the layer's next.
            for _ in 0..OFFERS {
                if count < LEAST {
                    break;
                }

                let (area, low, high) = largest_rectangle(size, &left);

                if area < LEAST {
                    break;
                }

                for y in low[1]..high[1] {
                    left[y * size[0] + low[0]..y * size[0] + high[0]].fill(false);
                }

                count -= area;

                let low = cell(axis, layer, low);
                let mut high = cell(axis, layer, high);

                high[axis] = layer + 1;
                offers.push((area, axis, (low, high)));
            }
        }
    }

    offers.sort_by_key(|&(area, ..)| usize::MAX - area);

    let mut slabs: Vec<(usize, Slab)> = Vec::new();

    for (_, axis, (mut low, mut high)) in offers {
        if slabs.len() == MOST {
            break;
        }

        // A slab already taken hides what this one would when it holds
        // this one, or covers this one's rectangle from a layer nearby on
        // the same axis.
        let covered = slabs.iter().any(|&(taken, (taken_low, taken_high))| {
            (0..3).all(|side| {
                let reach = if side == axis && taken == axis {
                    NEAR
                } else {
                    0
                };

                taken_low[side] <= low[side] + reach && high[side] <= taken_high[side] + reach
            })
        });

        if covered {
            continue;
        }

        let filled = |layer: usize, low: [usize; 3], high: [usize; 3]| {
            let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);

            (low[u]..high[u]).all(|x| (low[v]..high[v]).all(|y| at(cell(axis, layer, [x, y]))))
        };

        while high[axis] < bounds[axis] && filled(high[axis], low, high) {
            high[axis] += 1;
        }

        while low[axis] > 0 && filled(low[axis] - 1, low, high) {
            low[axis] -= 1;
        }

        slabs.push((axis, (low, high)));
    }

    let corner = |corner: [usize; 3]| {
        let [x, y, z] = corner.map(|side| side as u32);

        TyVector3U32::new(x, y, z)
    };

    slabs
        .into_iter()
        .map(|(_, (low, high))| (corner(low), corner(high)))
        .collect()
}

/// The largest rectangle of the cells of `filled`, a layer `size` cells wide
/// and high row by row: its cells, its low cell, and the corner past its
/// high cell. An empty layer gives no cells.
fn largest_rectangle(size: [usize; 2], filled: &[bool]) -> (usize, [usize; 2], [usize; 2]) {
    let [width, height] = size;

    // The filled cells that end at the row so far in each column.
    let mut runs = vec![0; width];
    let mut best = (0, [0; 2], [0; 2]);

    // The columns whose runs rise from one to the next.
    let mut rising: Vec<usize> = Vec::new();

    for y in 0..height {
        for (run, &filled) in runs.iter_mut().zip(&filled[y * width..]) {
            *run = if filled { *run + 1 } else { 0 };
        }

        rising.clear();

        for x in 0..=width {
            let run = runs.get(x).copied().unwrap_or(0);

            // A column no taller than the last ends each rectangle as tall
            // as that one.
            while let Some(&last) = rising.last().filter(|&&last| runs[last] >= run) {
                rising.pop();

                let left = rising.last().map_or(0, |&before| before + 1);
                let area = runs[last] * (x - left);

                if area > best.0 {
                    best = (area, [left, y + 1 - runs[last]], [x, y + 1]);
                }
            }

            rising.push(x);
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use crate::{painted_object, solid_slabs};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;

    fn slabs(
        bounds: [u32; 3],
        paint: impl Fn(TyVector3U32) -> Option<u32>,
    ) -> Vec<([u32; 3], [u32; 3])> {
        let [x, y, z] = bounds;

        let object = painted_object("slabs", TyVector3U32::new(x, y, z), |cell| {
            paint(cell).map(U32Id::from_u32)
        });

        solid_slabs(&object, &|material| material == 0)
            .into_iter()
            .map(|(low, high)| (low.to_array(), high.to_array()))
            .collect()
    }

    #[test]
    fn a_wall_gives_its_whole_layer() {
        // A wall two cells thick whose outer layer has a window.
        let found = slabs([2, 16, 16], |cell| {
            let window = cell.x == 0 && (6..10).contains(&cell.y) && (6..10).contains(&cell.z);

            (!window).then_some(0)
        });

        // The inner layer comes first. The outer layer's rectangles lie
        // within two layers of it and add nothing.
        assert_eq!(found, [([1, 0, 0], [2, 16, 16])]);
    }

    #[test]
    fn a_slab_grows_through_the_layers_that_fill_it() {
        let found = slabs([10, 9, 8], |_| Some(0));

        assert_eq!(found, [([0, 0, 0], [10, 9, 8])]);
    }

    #[test]
    fn cells_that_pass_light_break_a_slab() {
        // A wall with a strip of glass across its middle.
        let found = slabs([1, 20, 16], |cell| Some(u32::from(cell.y == 9)));

        assert_eq!(found, [([0, 10, 0], [1, 20, 16]), ([0, 0, 0], [1, 9, 16])]);
    }

    #[test]
    fn a_small_face_gives_no_slab() {
        assert_eq!(slabs([7, 9, 1], |_| Some(0)), []);
    }
}
