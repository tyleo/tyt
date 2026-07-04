use crate::{Check, Failures, VoxjDecodedObject, decode_voxj_object, voxj_palette_cell_counts};
use std::collections::HashSet;
use voxj::{VoxjMain, VoxjObject};

/// Decodes each object whose palette refs resolve and runs the geometry checks
/// on the result: the blocks decode, samples index real cells, positions are
/// unique, and bounds are tight. Objects with an out-of-range ref are skipped;
/// [`check_indices`](crate::check_indices()) already reported the ref.
pub fn check_geometry(main: &VoxjMain, failures: &mut Failures) {
    let state = &main.runtime_state;
    for (index, object) in state.objects.iter().enumerate() {
        if !failures.go() {
            return;
        }
        let Ok(cell_counts) = voxj_palette_cell_counts(&object.palette_refs, &state.palettes)
        else {
            continue;
        };
        let decoded = match decode_voxj_object(object, &cell_counts) {
            Ok(decoded) => decoded,
            Err(error) => {
                failures.report(
                    Check::Blocks,
                    format!("object {index} has a malformed position or sample block: {error}"),
                );
                continue;
            }
        };

        check_sample_cells(index, object, &decoded, main, failures);
        if !failures.go() {
            return;
        }
        check_positions(index, object, &decoded, failures);
    }
}

/// Each decoded sample indexes a real cell of the palette it samples.
fn check_sample_cells(
    index: usize,
    object: &VoxjObject,
    decoded: &VoxjDecodedObject,
    main: &VoxjMain,
    failures: &mut Failures,
) {
    for (voxel, row) in decoded.samples.iter().enumerate() {
        // Decoding guarantees one sample per referenced palette, so each
        // channel indexes the palette named at that position.
        for (channel, &cell) in row.iter().enumerate() {
            let palette = object.palette_refs[channel];
            let cell_count = main.runtime_state.palettes[palette].data.len();
            if cell as usize >= cell_count {
                failures.report(
                    Check::SampleCells,
                    format!(
                        "object {index} voxel {voxel} samples cell {cell} of palette {palette}, \
                         which has {cell_count} cells"
                    ),
                );
                if !failures.go() {
                    return;
                }
            }
        }
    }
}

/// Decoded positions lie within bounds, do not repeat, and bounds are exactly
/// tight around them. Tightness is checked only when every position is in
/// bounds, since an out-of-bounds voxel makes the extent meaningless.
fn check_positions(
    index: usize,
    object: &VoxjObject,
    decoded: &VoxjDecodedObject,
    failures: &mut Failures,
) {
    let [bound_x, bound_y, bound_z] = object.bounds;
    let mut seen = HashSet::with_capacity(decoded.positions.len());
    let mut min = [u32::MAX; 3];
    let mut max = [0u32; 3];
    let mut out_of_bounds = false;

    for &[x, y, z] in &decoded.positions {
        if x >= bound_x || y >= bound_y || z >= bound_z {
            failures.report(
                Check::Bounds,
                format!(
                    "object {index} voxel position [{x}, {y}, {z}] lies outside \
                     bounds [{bound_x}, {bound_y}, {bound_z}]"
                ),
            );
            out_of_bounds = true;
            if !failures.go() {
                return;
            }
        }
        if !seen.insert([x, y, z]) {
            failures.report(
                Check::UniquePositions,
                format!("object {index} repeats voxel position [{x}, {y}, {z}]"),
            );
            if !failures.go() {
                return;
            }
        }
        for (axis, coordinate) in [x, y, z].into_iter().enumerate() {
            min[axis] = min[axis].min(coordinate);
            max[axis] = max[axis].max(coordinate);
        }
    }

    if out_of_bounds {
        return;
    }

    if decoded.positions.is_empty() {
        if object.bounds != [0, 0, 0] {
            failures.report(
                Check::Bounds,
                format!(
                    "object {index} is empty, so its bounds must be [0, 0, 0], \
                     not [{bound_x}, {bound_y}, {bound_z}]"
                ),
            );
        }
        return;
    }

    for (axis, name) in ["x", "y", "z"].into_iter().enumerate() {
        if min[axis] != 0 || object.bounds[axis] != max[axis] + 1 {
            failures.report(
                Check::Bounds,
                format!(
                    "object {index} bounds are not tight on {name}: its voxels span \
                     [{}, {}], so the bound must be {}, not {}",
                    min[axis],
                    max[axis],
                    max[axis] + 1,
                    object.bounds[axis]
                ),
            );
            if !failures.go() {
                return;
            }
        }
    }
}
