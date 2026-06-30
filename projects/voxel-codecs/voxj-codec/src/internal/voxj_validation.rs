use crate::{
    VoxjCheck, VoxjCheckStatus, VoxjDecodedObject, decode_voxj_object, voxj_palette_cell_counts,
};
use std::collections::HashSet;
use voxj::{VoxjFile, VoxjMain, VoxjObject, VoxjValue};

/// The only Voxel Json document version this codec understands.
const SUPPORTED_VERSION: u32 = 1;

/// How far a rotation quaternion's length-squared may stray from `1` and still
/// count as a unit quaternion.
const ROTATION_TOLERANCE: f64 = 1e-6;

/// One spec validation check, the unit a failure is attributed to and the unit
/// [`check_voxj_file`](crate::check_voxj_file()) reports a result for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Check {
    /// The version is recognized.
    Version,
    /// Palettes are rectangular, have distinct attribute keys, and any `rgba`
    /// value is a `#RRGGBBAA` string.
    Palettes,
    /// Palette refs, node children, and roots resolve and are listed at most
    /// once.
    Indices,
    /// Each object's position and sample blocks decode, with matching arity and
    /// per-channel lengths.
    Blocks,
    /// Voxel positions within an object are unique.
    UniquePositions,
    /// Positions lie within bounds and bounds are exactly tight around them.
    Bounds,
    /// Each sample indexes a real cell of the palette it samples.
    SampleCells,
    /// The hierarchy is acyclic.
    Acyclic,
    /// No transform scale component is zero.
    Scale,
    /// Every transform rotation is a unit quaternion.
    Rotation,
    /// Each edit grid contains its object's runtime grid.
    EditState,
    /// Sample order matches the position block's voxel order: an authoring
    /// invariant no document can witness.
    SampleOrder,
}

impl Check {
    /// The short stable identifier reported as [`VoxjCheck::name`].
    fn name(self) -> &'static str {
        match self {
            Check::Version => "version",
            Check::Palettes => "palettes",
            Check::Indices => "indices",
            Check::Blocks => "blocks",
            Check::UniquePositions => "unique-positions",
            Check::Bounds => "bounds",
            Check::SampleCells => "sample-cells",
            Check::Acyclic => "acyclic",
            Check::Scale => "scale",
            Check::Rotation => "rotation",
            Check::EditState => "edit-state",
            Check::SampleOrder => "sample-order",
        }
    }
}

/// Every check, in the order [`check_voxj_file`](crate::check_voxj_file())
/// reports them.
const REPORT_ORDER: [Check; 12] = [
    Check::Version,
    Check::Palettes,
    Check::Indices,
    Check::Blocks,
    Check::UniquePositions,
    Check::Bounds,
    Check::SampleCells,
    Check::Acyclic,
    Check::Scale,
    Check::Rotation,
    Check::EditState,
    Check::SampleOrder,
];

/// Accumulates validation failures, each tagged with the check it violates. In
/// fail-fast mode the driver stops once the first failure lands; in report mode
/// it gathers them all. A check consults [`go`](Self::go) before further work a
/// prior failure has already made moot.
struct Failures {
    fail_fast: bool,
    items: Vec<(Check, String)>,
}

impl Failures {
    /// Records one failure.
    fn report(&mut self, check: Check, message: String) {
        self.items.push((check, message));
    }

    /// Whether to keep scanning: false once a fail-fast run holds its first
    /// failure.
    fn go(&self) -> bool {
        !self.fail_fast || self.items.is_empty()
    }
}

/// Runs every check over `file`, returning the failures it found in discovery
/// order. With `fail_fast`, scanning stops at the first failure; otherwise it
/// continues so a caller can report them all.
///
/// The checks below the version are independent except where one decodes data
/// an earlier check guards: an object's blocks decode only when its palette
/// refs resolve, and the acyclicity walk treats an out-of-range child edge,
/// already reported by [`Check::Indices`], as absent. Such checks skip the work
/// rather than double-report.
pub(crate) fn collect_voxj_failures(file: &VoxjFile, fail_fast: bool) -> Vec<(Check, String)> {
    let mut failures = Failures {
        fail_fast,
        items: Vec::new(),
    };

    check_version(file, &mut failures);
    if failures.go() {
        check_palettes(&file.main, &mut failures);
    }
    if failures.go() {
        check_indices(&file.main, &mut failures);
    }
    if failures.go() {
        check_geometry(&file.main, &mut failures);
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

    failures.items
}

/// Groups tagged failures into one [`VoxjCheck`] per check, in
/// [`REPORT_ORDER`]. A check with no failures passed; [`Check::SampleOrder`] is
/// always unverifiable.
pub(crate) fn build_voxj_report(failures: Vec<(Check, String)>) -> Vec<VoxjCheck> {
    REPORT_ORDER
        .iter()
        .map(|&check| {
            let status = if check == Check::SampleOrder {
                VoxjCheckStatus::Unverifiable
            } else {
                let messages: Vec<String> = failures
                    .iter()
                    .filter(|(c, _)| *c == check)
                    .map(|(_, message)| message.clone())
                    .collect();
                if messages.is_empty() {
                    VoxjCheckStatus::Passed
                } else {
                    VoxjCheckStatus::Failed(messages)
                }
            };
            VoxjCheck {
                name: check.name(),
                status,
            }
        })
        .collect()
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

/// Every palette is rectangular, declares each attribute key once, and stores
/// any `rgba` value as a `#RRGGBBAA` string.
fn check_palettes(main: &VoxjMain, failures: &mut Failures) {
    for (index, palette) in main.runtime_state.palettes.iter().enumerate() {
        if !failures.go() {
            return;
        }

        let mut seen = HashSet::with_capacity(palette.attributes.len());
        for attribute in &palette.attributes {
            if !seen.insert(attribute.as_str()) {
                failures.report(
                    Check::Palettes,
                    format!("palette {index} declares attribute key {attribute:?} more than once"),
                );
                if !failures.go() {
                    return;
                }
            }
        }

        for (cell, row) in palette.data.iter().enumerate() {
            if row.len() != palette.attributes.len() {
                failures.report(
                    Check::Palettes,
                    format!(
                        "palette {index} cell {cell} has {} values but the palette has {} attributes",
                        row.len(),
                        palette.attributes.len()
                    ),
                );
                if !failures.go() {
                    return;
                }
            }
        }

        if let Some(rgba) = palette.attributes.iter().position(|key| key == "rgba") {
            for (cell, row) in palette.data.iter().enumerate() {
                let Some(value) = row.get(rgba) else {
                    // Short row already reported above.
                    continue;
                };
                if !is_rgba(value) {
                    failures.report(
                        Check::Palettes,
                        format!(
                            "palette {index} cell {cell} rgba value {} is not #RRGGBBAA with uppercase hex",
                            describe_value(value)
                        ),
                    );
                    if !failures.go() {
                        return;
                    }
                }
            }
        }
    }
}

/// Palette refs, node children, child objects, and roots all resolve and none
/// repeats within its list.
fn check_indices(main: &VoxjMain, failures: &mut Failures) {
    let state = &main.runtime_state;

    for (index, object) in state.objects.iter().enumerate() {
        if !failures.go() {
            return;
        }
        let mut seen = HashSet::with_capacity(object.palette_refs.len());
        for &palette_ref in &object.palette_refs {
            if palette_ref >= state.palettes.len() {
                failures.report(
                    Check::Indices,
                    format!(
                        "object {index} references palette {palette_ref}, but the document has {} palettes",
                        state.palettes.len()
                    ),
                );
            } else if !seen.insert(palette_ref) {
                failures.report(
                    Check::Indices,
                    format!("object {index} references palette {palette_ref} more than once"),
                );
            }
            if !failures.go() {
                return;
            }
        }
    }

    for (index, node) in state.hierarchy_nodes.iter().enumerate() {
        if !failures.go() {
            return;
        }
        let mut seen_nodes = HashSet::with_capacity(node.child_nodes.len());
        for &child in &node.child_nodes {
            if child >= state.hierarchy_nodes.len() {
                failures.report(
                    Check::Indices,
                    format!(
                        "hierarchy node {index} lists child node {child}, but the document has {} nodes",
                        state.hierarchy_nodes.len()
                    ),
                );
            } else if !seen_nodes.insert(child) {
                failures.report(
                    Check::Indices,
                    format!("hierarchy node {index} lists child node {child} more than once"),
                );
            }
            if !failures.go() {
                return;
            }
        }

        let mut seen_objects = HashSet::with_capacity(node.child_objects.len());
        for &object in &node.child_objects {
            if object >= state.objects.len() {
                failures.report(
                    Check::Indices,
                    format!(
                        "hierarchy node {index} places object {object}, but the document has {} objects",
                        state.objects.len()
                    ),
                );
            } else if !seen_objects.insert(object) {
                failures.report(
                    Check::Indices,
                    format!("hierarchy node {index} places object {object} more than once"),
                );
            }
            if !failures.go() {
                return;
            }
        }
    }

    let mut seen_roots = HashSet::with_capacity(state.root_hierarchy_nodes.len());
    for &root in &state.root_hierarchy_nodes {
        if !failures.go() {
            return;
        }
        if root >= state.hierarchy_nodes.len() {
            failures.report(
                Check::Indices,
                format!(
                    "root references hierarchy node {root}, but the document has {} nodes",
                    state.hierarchy_nodes.len()
                ),
            );
        } else if !seen_roots.insert(root) {
            failures.report(
                Check::Indices,
                format!("root lists hierarchy node {root} more than once"),
            );
        }
    }
}

/// Decodes each object whose palette refs resolve and runs the geometry checks
/// on the result: the blocks decode, samples index real cells, positions are
/// unique, and bounds are tight. Objects with an out-of-range ref are skipped;
/// [`check_indices`] already reported the ref.
fn check_geometry(main: &VoxjMain, failures: &mut Failures) {
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

/// The hierarchy is acyclic.
fn check_acyclic(main: &VoxjMain, failures: &mut Failures) {
    if let Some(node) = first_cycle_node(main) {
        failures.report(
            Check::Acyclic,
            format!("hierarchy is not acyclic: a cycle reaches node {node}"),
        );
    }
}

/// No transform scale component is zero and every rotation is a unit
/// quaternion.
fn check_transforms(main: &VoxjMain, failures: &mut Failures) {
    for (index, node) in main.runtime_state.hierarchy_nodes.iter().enumerate() {
        if !failures.go() {
            return;
        }
        let [scale_x, scale_y, scale_z] = node.transform.scale;
        if scale_x == 0.0 || scale_y == 0.0 || scale_z == 0.0 {
            failures.report(
                Check::Scale,
                format!("hierarchy node {index} has a transform scale component of zero"),
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
                    "hierarchy node {index} rotation is not a unit quaternion \
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
    for (index, (edit, object)) in edit_state.objects.iter().zip(objects).enumerate() {
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
                        "edit grid {index} on axis {axis} ([{edit_min}, {edit_max})) does not \
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

/// A node on a `child_nodes` cycle, or `None` if the hierarchy is acyclic. An
/// iterative three-colour DFS, so a deep chain cannot overflow the stack: a
/// back edge into an in-progress node is a cycle, revisiting a finished node is
/// not. An out-of-range child edge is treated as absent, since
/// [`check_indices`] reports it; this keeps the walk safe to run regardless.
fn first_cycle_node(main: &VoxjMain) -> Option<usize> {
    const WHITE: u8 = 0;
    const GREY: u8 = 1;
    const BLACK: u8 = 2;

    let nodes = &main.runtime_state.hierarchy_nodes;
    let mut colour = vec![WHITE; nodes.len()];
    for start in 0..nodes.len() {
        if colour[start] != WHITE {
            continue;
        }
        colour[start] = GREY;
        // Each frame is a node plus how many of its children we have walked.
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        while let Some(&(node, cursor)) = stack.last() {
            let children = &nodes[node].child_nodes;
            if cursor < children.len() {
                stack.last_mut().unwrap().1 += 1;
                let child = children[cursor];
                if child >= nodes.len() {
                    continue;
                }
                match colour[child] {
                    WHITE => {
                        colour[child] = GREY;
                        stack.push((child, 0));
                    }
                    GREY => return Some(child),
                    _ => {}
                }
            } else {
                colour[node] = BLACK;
                stack.pop();
            }
        }
    }
    None
}

/// Whether `value` is a `#RRGGBBAA` string: a leading `#` then exactly eight
/// uppercase hex digits, matching `^#[0-9A-F]{8}$`.
fn is_rgba(value: &VoxjValue) -> bool {
    let VoxjValue::Text(text) = value else {
        return false;
    };
    let Some(hex) = text.strip_prefix('#') else {
        return false;
    };
    hex.len() == 8 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'A'..=b'F'))
}

/// A short description of a value for a failure message.
fn describe_value(value: &VoxjValue) -> String {
    match value {
        VoxjValue::Text(text) => format!("{text:?}"),
        VoxjValue::Number(number) => number.to_string(),
        VoxjValue::Bool(boolean) => boolean.to_string(),
        VoxjValue::Null => "null".to_owned(),
        VoxjValue::Array(_) => "an array".to_owned(),
        VoxjValue::Object(_) => "an object".to_owned(),
    }
}
