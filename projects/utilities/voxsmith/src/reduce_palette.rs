use crate::{ColorSpace, Dither, Error, ReductionMethod, Result};
use branded_id::U32Id;
use std::{cmp::Ordering, collections::HashMap};
use ty_math::TySrgbaColor;
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

        ReductionMethod::Octree => {
            return Err(unsupported(
                "reduction method `octree` is not yet implemented; only median-cut is available",
            ));
        }

        ReductionMethod::Kmeans => {
            return Err(unsupported(
                "reduction method `kmeans` is not yet implemented; only median-cut is available",
            ));
        }
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

    coords: [f64; 3],

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
            a.coords[axis]
                .partial_cmp(&b.coords[axis])
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
    let mut low = [f64::INFINITY; 3];
    let mut high = [f64::NEG_INFINITY; 3];

    for point in cell_box {
        for axis in 0..3 {
            low[axis] = low[axis].min(point.coords[axis]);
            high[axis] = high[axis].max(point.coords[axis]);
        }
    }

    let mut axis = 0;
    let mut extent = high[0] - low[0];

    for candidate in 1..3 {
        let spread = high[candidate] - low[candidate];
        if spread > extent {
            extent = spread;
            axis = candidate;
        }
    }

    (axis, extent)
}

/// A cell's color as a point in the chosen space; alpha is not a clustering
/// dimension (it rides along with the representative's row). The `rgb` space is
/// the naive distance on the stored sRGB components; `oklab` and `lab` decode to
/// linear light first.
fn to_space(rgba: [u8; 4], space: ColorSpace) -> [f64; 3] {
    let [r, g, b, a] = rgba;
    let color = TySrgbaColor::new(r, g, b, a);

    match space {
        ColorSpace::Rgb => [r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0],

        ColorSpace::Oklab => {
            let oklab = color.to_linear_rgba().to_oklab();
            [oklab.l, oklab.a, oklab.b]
        }

        ColorSpace::Lab => {
            let lab = color.to_linear_rgba().to_cielab();
            [lab.l, lab.a, lab.b]
        }
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
    fn octree_kmeans_and_dither_error_only_when_the_reduction_fires() {
        // Under the cap: the controls are inert, so no error.
        let (mut state, palette, _) = state_with_colors(&["#FF0000FF", "#00FF00FF"], &[]);
        assert!(
            reduce_palette(
                &mut state,
                palette,
                5,
                ReductionMethod::Octree,
                ColorSpace::Oklab,
                Dither::None
            )
            .unwrap()
            .is_none()
        );

        // Over the cap: octree, kmeans, and dither are not yet built.
        let over = || state_with_colors(&["#FF0000FF", "#00FF00FF", "#0000FFFF"], &[]).0;
        assert!(
            reduce_palette(
                &mut over(),
                palette,
                2,
                ReductionMethod::Octree,
                ColorSpace::Oklab,
                Dither::None
            )
            .is_err()
        );
        assert!(
            reduce_palette(
                &mut over(),
                palette,
                2,
                ReductionMethod::Kmeans,
                ColorSpace::Oklab,
                Dither::None
            )
            .is_err()
        );
        assert!(
            reduce_palette(
                &mut over(),
                palette,
                2,
                ReductionMethod::MedianCut,
                ColorSpace::Oklab,
                Dither::FloydSteinberg
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
