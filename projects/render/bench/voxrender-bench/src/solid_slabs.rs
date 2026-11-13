use crate::CellGrid;
use ty_math::TyVector3U32;

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

/// Up to [`MOST`] boxes of `grid`'s opaque cells alone, widest face first,
/// each as its low cell and the corner past its high cell. `opaque` says
/// whether a material passes no light. What lies behind such a box
/// cannot show, which lets a view leave it out.
///
/// Each layer of cells across an axis offers its largest rectangles of
/// opaque cells. The widest offers win. One grows along its axis for as
/// long as the layers beside it fill its rectangle.
pub fn solid_slabs(
    grid: &CellGrid,
    opaque: &dyn Fn(u32) -> bool,
) -> Vec<(TyVector3U32, TyVector3U32)> {
    let bounds = grid.bounds.to_array().map(|side| side as usize);

    let solid: Vec<bool> = grid
        .cells
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

    // Each axis's layers as rows of bits, 64 cells a word. A layer's rows
    // run along the axis after its own, and follow one another along the
    // axis after that. A search then reads a layer's opaque cells alone,
    // where most of a landscape's are empty.
    let words = [0, 1, 2].map(|axis| bounds[(axis + 1) % 3].div_ceil(64));

    let mut layers =
        [0, 1, 2].map(|axis| vec![0u64; bounds[axis] * bounds[(axis + 2) % 3] * words[axis]]);

    let mut index = 0;

    for x in 0..bounds[0] {
        for y in 0..bounds[1] {
            for z in 0..bounds[2] {
                if solid[index] {
                    let at = [x, y, z];

                    for axis in 0..3 {
                        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
                        let row = (at[axis] * bounds[v] + at[v]) * words[axis];

                        layers[axis][row + at[u] / 64] |= 1 << (at[u] % 64);
                    }
                }

                index += 1;
            }
        }
    }

    for axis in 0..3 {
        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
        let size = [bounds[u], bounds[v]];
        let words = words[axis];

        for layer in 0..bounds[axis] {
            // The cells of the layer that no offer has taken.
            let first = layer * size[1] * words;
            let left = &mut layers[axis][first..first + size[1] * words];

            let mut count: usize = left.iter().map(|word| word.count_ones() as usize).sum();

            // Each offer takes its cells from the layer's next.
            for _ in 0..OFFERS {
                if count < LEAST {
                    break;
                }

                let (area, low, high) = largest_rectangle(size, left);

                if area < LEAST {
                    break;
                }

                for y in low[1]..high[1] {
                    for x in low[0]..high[0] {
                        left[y * words + x / 64] &= !(1 << (x % 64));
                    }
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

/// The largest rectangle of filled cells in a layer `size` cells wide and
/// high: its cells, its low cell, and the corner past its high cell. `rows`
/// holds the layer row by row, a bit a cell from each word's low bit, and
/// each row starts a word. An empty layer gives no cells.
///
/// The search reads each stretch of filled cells along a row and passes
/// over the empty cells between, so it takes time with the filled cells.
fn largest_rectangle(size: [usize; 2], rows: &[u64]) -> (usize, [usize; 2], [usize; 2]) {
    let [width, height] = size;
    let words = width.div_ceil(64);

    // The filled cells that end at the row so far in each column.
    let mut runs = vec![0; width];
    let mut best = (0, [0; 2], [0; 2]);

    // The columns whose runs rise from one to the next.
    let mut rising: Vec<usize> = Vec::new();

    // The first cell from `from` on in `row` that is filled, or empty when
    // `filled` is not. The row's width when none is.
    let next = |row: &[u64], from: usize, filled: bool| {
        let mut at = from;

        while at < width {
            let word = if filled { row[at / 64] } else { !row[at / 64] } >> (at % 64);

            if word != 0 {
                return (at + word.trailing_zeros() as usize).min(width);
            }

            at = (at / 64 + 1) * 64;
        }

        width
    };

    for y in 0..height {
        let row = &rows[y * words..(y + 1) * words];

        // A column the row leaves empty ends its run.
        if y > 0 {
            for (word, above) in rows[(y - 1) * words..y * words].iter().enumerate() {
                let mut ended = above & !row[word];

                while ended != 0 {
                    runs[word * 64 + ended.trailing_zeros() as usize] = 0;
                    ended &= ended - 1;
                }
            }
        }

        let mut low = next(row, 0, true);

        while low < width {
            let high = next(row, low, false);

            for run in &mut runs[low..high] {
                *run += 1;
            }

            rising.clear();

            for x in low..=high {
                let run = if x < high { runs[x] } else { 0 };

                // A column no taller than the last ends each rectangle as
                // tall as that one.
                while let Some(&last) = rising.last().filter(|&&last| runs[last] >= run) {
                    rising.pop();

                    let left = rising.last().map_or(low, |&before| before + 1);
                    let area = runs[last] * (x - left);

                    if area > best.0 {
                        best = (area, [left, y + 1 - runs[last]], [x, y + 1]);
                    }
                }

                if x < high {
                    rising.push(x);
                }
            }

            low = next(row, high, true);
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use crate::{CellGrid, painted_object, solid_slabs, solid_slabs::largest_rectangle};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;

    /// The largest rectangle of `filled`'s cells, a layer row by row, by
    /// a search that reads every cell.
    fn plain_rectangle(size: [usize; 2], filled: &[bool]) -> (usize, [usize; 2], [usize; 2]) {
        let [width, height] = size;
        let mut runs = vec![0; width];
        let mut best = (0, [0; 2], [0; 2]);
        let mut rising: Vec<usize> = Vec::new();

        for y in 0..height {
            for (run, &filled) in runs.iter_mut().zip(&filled[y * width..]) {
                *run = if filled { *run + 1 } else { 0 };
            }

            rising.clear();

            for x in 0..=width {
                let run = runs.get(x).copied().unwrap_or(0);

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

    #[test]
    fn a_search_over_bits_finds_what_a_search_of_every_cell_does() {
        let mut state = 0x2545_f491_4f6c_dd1du64;

        let mut random = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;

            state
        };

        // Layers narrower and wider than a word, from sparse to full.
        for (width, height) in [
            (1usize, 1usize),
            (7, 5),
            (64, 9),
            (65, 12),
            (130, 40),
            (256, 31),
        ] {
            for fill in [1, 4, 8, 15, 16] {
                let filled: Vec<bool> = (0..width * height).map(|_| random() % 16 < fill).collect();
                let words = width.div_ceil(64);
                let mut rows = vec![0u64; words * height];

                for (index, _) in filled.iter().enumerate().filter(|&(_, &filled)| filled) {
                    rows[index / width * words + index % width / 64] |= 1 << (index % width % 64);
                }

                assert_eq!(
                    largest_rectangle([width, height], &rows),
                    plain_rectangle([width, height], &filled),
                    "{width} by {height}, {fill} of 16 filled"
                );
            }
        }
    }

    fn slabs(
        bounds: [u32; 3],
        paint: impl Fn(TyVector3U32) -> Option<u32>,
    ) -> Vec<([u32; 3], [u32; 3])> {
        let [x, y, z] = bounds;

        let object = painted_object("slabs", TyVector3U32::new(x, y, z), |cell| {
            paint(cell).map(U32Id::from_u32)
        });

        solid_slabs(&CellGrid::new(&object, &|id| id), &|material| material == 0)
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
