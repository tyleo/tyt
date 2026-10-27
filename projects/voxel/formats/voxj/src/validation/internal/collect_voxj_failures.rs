use crate::{
    DecodeBase64, VoxjFile, VoxjMain, VoxjObject, VoxjPalette, VoxjRuntimeState,
    objects::{VoxjDecodedObject, decode_voxj_object, voxj_palette_material_counts},
    validation::{Check, Failures},
};
use std::collections::HashSet;

/// The only Voxel Json document version this codec understands.
const SUPPORTED_VERSION: u32 = 1;

/// How far a rotation quaternion's length-squared may stray from `1` and still
/// count as a unit quaternion.
const ROTATION_TOLERANCE: f64 = 1e-6;

/// Runs every check over `file`, returning the failures it found in discovery
/// order. With `fail_fast`, scanning stops at the first failure; otherwise it
/// continues so a caller can report them all.
///
/// The checks below the version are independent except where one decodes data
/// an earlier check guards: an object's blocks decode only when its palette
/// refs resolve, and the acyclicity walk treats an out-of-range child edge,
/// already reported by [`Check::Indices`], as absent. Such checks skip the work
/// rather than double-report.
pub fn collect_voxj_failures<D: DecodeBase64>(
    dependencies: &D,
    file: &VoxjFile,
    fail_fast: bool,
) -> Vec<(Check, String)> {
    let mut failures = Failures::new(fail_fast);

    check_version(file, &mut failures);
    if failures.go() {
        check_palettes(&file.main, &mut failures);
    }
    if failures.go() {
        check_indices(&file.main, &mut failures);
    }
    if failures.go() {
        check_geometry(dependencies, &file.main, &mut failures);
    }
    if failures.go() {
        check_acyclic(&file.main, &mut failures);
    }
    if failures.go() {
        check_transforms(&file.main, &mut failures);
    }
    if failures.go() {
        check_edit_state(&file.main, &mut failures);
    }

    failures.into_items()
}

/// The version is one this codec understands.
fn check_version(file: &VoxjFile, failures: &mut Failures) {
    if file.version != SUPPORTED_VERSION {
        failures.report(
            Check::Version,
            format!(
                "unrecognized version {}, expected {SUPPORTED_VERSION}",
                file.version
            ),
        );
    }
}

/// Per palette:
/// 1. every property has a non-empty name, distinct within the palette, and
///    an in-range value pool;
/// 2. materials hold one row per material, each of exactly one value-index
///    per property, within the value pool that property binds.
fn check_palettes(main: &VoxjMain, failures: &mut Failures) {
    let state = &main.runtime_state;
    for (palette_index, palette) in state.palettes.iter().enumerate() {
        if !failures.go() {
            return;
        }

        // One namespace per palette (rule 10.2).
        let mut seen = HashSet::with_capacity(palette.properties.len());

        for (property_index, property) in palette.properties.iter().enumerate() {
            check_name(
                palette_index,
                property_index,
                &property.name,
                &mut seen,
                failures,
            );
            if property.value_pool >= state.value_pools.len() {
                failures.report(
                    Check::Palettes,
                    format!(
                        "palette {palette_index} property {property_index} references value pool {}, but the document has {} value pools",
                        property.value_pool,
                        state.value_pools.len()
                    ),
                );
            }
            if !failures.go() {
                return;
            }
        }

        check_materials(palette_index, palette, state, failures);
    }
}

/// A property's name is non-empty and not yet taken within the palette.
fn check_name<'a>(
    palette_index: usize,
    property_index: usize,
    name: &'a str,
    seen: &mut HashSet<&'a str>,
    failures: &mut Failures,
) {
    if name.is_empty() {
        failures.report(
            Check::Palettes,
            format!("palette {palette_index} property {property_index} has an empty name"),
        );
    } else if !seen.insert(name) {
        failures.report(
            Check::Palettes,
            format!("palette {palette_index} lists property {name:?} more than once"),
        );
    }
}

/// Every materials row holds exactly one value-index per property, and every
/// value-index falls within the values of the value pool its property names.
fn check_materials(
    palette_index: usize,
    palette: &VoxjPalette,
    state: &VoxjRuntimeState,
    failures: &mut Failures,
) {
    let width = palette.properties.len();
    for (material_index, row) in palette.materials.iter().enumerate() {
        if row.len() != width {
            failures.report(
                Check::Palettes,
                format!(
                    "palette {palette_index} material {material_index} has {} value-indices but the \
                     palette has {width} properties",
                    row.len()
                ),
            );
            if !failures.go() {
                return;
            }
            // With the row misaligned, cells no longer pair with properties.
            continue;
        }

        for (property_index, &value_index) in row.iter().enumerate() {
            // The row arity matches here, so the property always resolves; its
            // value pool resolves only when the reference is in range, already
            // reported above when it is not.
            let Some(value_pool) = state
                .value_pools
                .get(palette.properties[property_index].value_pool)
            else {
                continue;
            };
            let value_pool_len = value_pool.len();
            if value_index >= value_pool_len {
                failures.report(
                    Check::Palettes,
                    format!(
                        "palette {palette_index} material {material_index} value-index {value_index} \
                         is out of range for a value pool with {value_pool_len} values"
                    ),
                );
                if !failures.go() {
                    return;
                }
            }
        }
    }
}

/// Object layers, node children, child objects, and roots all resolve; node
/// children, child objects, and roots each list no index twice. Two layers may
/// reference the same palette, so a repeated layer entry is allowed.
fn check_indices(main: &VoxjMain, failures: &mut Failures) {
    let state = &main.runtime_state;

    for (object_index, object) in state.objects.iter().enumerate() {
        for &palette_index in &object.layers {
            if !failures.go() {
                return;
            }
            if palette_index >= state.palettes.len() {
                failures.report(
                    Check::Indices,
                    format!(
                        "object {object_index} references palette {palette_index}, but the document has {} palettes",
                        state.palettes.len()
                    ),
                );
            }
        }
    }

    for (node_index, node) in state.nodes.iter().enumerate() {
        if !failures.go() {
            return;
        }
        let mut seen_nodes = HashSet::with_capacity(node.child_nodes.len());
        for &child_node_index in &node.child_nodes {
            if child_node_index >= state.nodes.len() {
                failures.report(
                    Check::Indices,
                    format!(
                        "hierarchy node {node_index} lists child node {child_node_index}, but the document has {} nodes",
                        state.nodes.len()
                    ),
                );
            } else if !seen_nodes.insert(child_node_index) {
                failures.report(
                    Check::Indices,
                    format!(
                        "hierarchy node {node_index} lists child node {child_node_index} more than once"
                    ),
                );
            }
            if !failures.go() {
                return;
            }
        }

        let mut seen_objects = HashSet::with_capacity(node.child_objects.len());
        for &child_object_index in &node.child_objects {
            if child_object_index >= state.objects.len() {
                failures.report(
                    Check::Indices,
                    format!(
                        "hierarchy node {node_index} places object {child_object_index}, but the document has {} objects",
                        state.objects.len()
                    ),
                );
            } else if !seen_objects.insert(child_object_index) {
                failures.report(
                    Check::Indices,
                    format!(
                        "hierarchy node {node_index} places object {child_object_index} more than once"
                    ),
                );
            }
            if !failures.go() {
                return;
            }
        }
    }

    let mut seen_roots = HashSet::with_capacity(state.root_nodes.len());
    for &root_node_index in &state.root_nodes {
        if !failures.go() {
            return;
        }
        if root_node_index >= state.nodes.len() {
            failures.report(
                Check::Indices,
                format!(
                    "root references hierarchy node {root_node_index}, but the document has {} nodes",
                    state.nodes.len()
                ),
            );
        } else if !seen_roots.insert(root_node_index) {
            failures.report(
                Check::Indices,
                format!("root lists hierarchy node {root_node_index} more than once"),
            );
        }
    }
}

/// Decodes each object whose layers resolve and runs the geometry checks on
/// the result: the blocks decode, samples index real materials, positions are
/// unique, and bounds are tight. Objects with an out-of-range layer are
/// skipped; [`check_indices`] already reported the layer.
fn check_geometry<D: DecodeBase64>(dependencies: &D, main: &VoxjMain, failures: &mut Failures) {
    let state = &main.runtime_state;
    for (object_index, object) in state.objects.iter().enumerate() {
        if !failures.go() {
            return;
        }
        let Ok(material_counts) = voxj_palette_material_counts(&object.layers, &state.palettes)
        else {
            continue;
        };
        let decoded = match decode_voxj_object(dependencies, object, &material_counts) {
            Ok(decoded) => decoded,

            Err(error) => {
                failures.report(
                    Check::Blocks,
                    format!(
                        "object {object_index} has a malformed position or sample block: {error}"
                    ),
                );
                continue;
            }
        };

        check_sample_materials(object_index, object, &decoded, &material_counts, failures);
        if !failures.go() {
            return;
        }
        check_positions(object_index, object, &decoded, failures);
    }
}

/// Each decoded sample indexes a real material of its layer's palette.
fn check_sample_materials(
    object_index: usize,
    object: &VoxjObject,
    decoded: &VoxjDecodedObject,
    material_counts: &[usize],
    failures: &mut Failures,
) {
    // Decoding guarantees one row entry per layer.
    for (voxel_index, row) in decoded.samples.iter().enumerate() {
        for (channel_index, &material_index) in row.iter().enumerate() {
            let palette_index = object.layers[channel_index];
            let material_count = material_counts[channel_index];
            if material_index as usize >= material_count {
                failures.report(
                    Check::SampleMaterials,
                    format!(
                        "object {object_index} voxel {voxel_index} samples material {material_index} of \
                         palette {palette_index}, which has {material_count} materials"
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
    object_index: usize,
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
                    "object {object_index} voxel position [{x}, {y}, {z}] lies outside \
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
                format!("object {object_index} repeats voxel position [{x}, {y}, {z}]"),
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
                    "object {object_index} is empty, so its bounds must be [0, 0, 0], \
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
                    "object {object_index} bounds are not tight on {name}: its voxels span \
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

/// The hierarchy is acyclic.
fn check_acyclic(main: &VoxjMain, failures: &mut Failures) {
    if let Some(node_index) = first_cycle_node_index(main) {
        failures.report(
            Check::Acyclic,
            format!("hierarchy is not acyclic: a cycle reaches node {node_index}"),
        );
    }
}

/// The index of a node on a `child_nodes` cycle, or `None` if the hierarchy is
/// acyclic. An iterative three-colour DFS, so a deep chain cannot overflow the
/// stack: a back edge into an in-progress node is a cycle, revisiting a
/// finished node is not. An out-of-range child edge is treated as absent, since
/// [`check_indices`] reports it; this keeps the walk safe to run regardless.
fn first_cycle_node_index(main: &VoxjMain) -> Option<usize> {
    const WHITE: u8 = 0;
    const GREY: u8 = 1;
    const BLACK: u8 = 2;

    let nodes = &main.runtime_state.nodes;
    let mut colour = vec![WHITE; nodes.len()];
    for start_index in 0..nodes.len() {
        if colour[start_index] != WHITE {
            continue;
        }
        colour[start_index] = GREY;
        // Each frame is a node index plus how many of that node's children we
        // have walked.
        let mut stack: Vec<(usize, usize)> = vec![(start_index, 0)];
        while let Some(&(node_index, cursor)) = stack.last() {
            let children = &nodes[node_index].child_nodes;
            if cursor < children.len() {
                stack.last_mut().unwrap().1 += 1;
                let child_index = children[cursor];
                if child_index >= nodes.len() {
                    continue;
                }
                match colour[child_index] {
                    WHITE => {
                        colour[child_index] = GREY;
                        stack.push((child_index, 0));
                    }

                    GREY => return Some(child_index),

                    _ => {}
                }
            } else {
                colour[node_index] = BLACK;
                stack.pop();
            }
        }
    }
    None
}

/// No transform scale component is zero and every rotation is a unit
/// quaternion.
fn check_transforms(main: &VoxjMain, failures: &mut Failures) {
    for (node_index, node) in main.runtime_state.nodes.iter().enumerate() {
        if !failures.go() {
            return;
        }
        let [scale_x, scale_y, scale_z] = node.transform.scale;
        if scale_x == 0.0 || scale_y == 0.0 || scale_z == 0.0 {
            failures.report(
                Check::Scale,
                format!("hierarchy node {node_index} has a transform scale component of zero"),
            );
            if !failures.go() {
                return;
            }
        }

        let [rotation_x, rotation_y, rotation_z, rotation_w] = node.transform.rotation;
        let length_squared = rotation_x * rotation_x
            + rotation_y * rotation_y
            + rotation_z * rotation_z
            + rotation_w * rotation_w;
        if (length_squared - 1.0).abs() > ROTATION_TOLERANCE {
            failures.report(
                Check::Rotation,
                format!(
                    "hierarchy node {node_index} rotation is not a unit quaternion \
                     (length squared {length_squared})"
                ),
            );
            if !failures.go() {
                return;
            }
        }
    }
}

/// When present, edit state lists one edit grid per runtime object and each
/// edit grid contains its object's runtime grid on every axis.
fn check_edit_state(main: &VoxjMain, failures: &mut Failures) {
    let Some(edit_state) = &main.edit_state else {
        return;
    };
    let objects = &main.runtime_state.objects;
    if edit_state.objects.len() != objects.len() {
        failures.report(
            Check::EditState,
            format!(
                "edit state lists {} objects, but the document has {} runtime objects",
                edit_state.objects.len(),
                objects.len()
            ),
        );
        return;
    }
    for (object_index, (edit, object)) in edit_state.objects.iter().zip(objects).enumerate() {
        if !failures.go() {
            return;
        }
        for axis in 0..3 {
            let edit_min = i64::from(edit.origin[axis]);
            let edit_max = edit_min + i64::from(edit.bounds[axis]);
            let run_min = i64::from(object.origin[axis]);
            let run_max = run_min + i64::from(object.bounds[axis]);
            if edit_min > run_min || edit_max < run_max {
                failures.report(
                    Check::EditState,
                    format!(
                        "edit grid {object_index} on axis {axis} ([{edit_min}, {edit_max})) does not \
                         contain the runtime grid ([{run_min}, {run_max}))"
                    ),
                );
                if !failures.go() {
                    return;
                }
            }
        }
    }
}
