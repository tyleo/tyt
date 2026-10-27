use crate::{
    Error, Result,
    operations::sdf_doc::{
        EntryPaths, SdfPlace, check_pattern, check_shape2d, check_shape3d, check_step,
        shape3d_references,
    },
};
use branded_id::{IdVec, IteratorExt, U32Id};
use sdfcore::{BSdfShape3d, BSdfStep, SdfShape3d, SdfState, SdfStep};
use std::collections::HashSet;

/// Runs the sdfj format's validation of the entries' arguments and of the
/// names over `state`, whose places are `places`. An error starts with the
/// path of the first step in `paths` that reaches the failing entry.
pub fn check_document(state: &SdfState, places: &[SdfPlace], paths: &EntryPaths) -> Result<()> {
    let at = |path: String| move |message: String| Error::invalid(format!("{path}: {message}"));

    for (shape2d_id, shape) in state.shapes2d.iter().enumerate_ids() {
        check_shape2d(shape).map_err(at(paths.shape2d(shape2d_id)))?;
    }

    let mut has_box: IdVec<BSdfShape3d, bool> = IdVec::with_capacity(state.shapes3d.len());

    for (shape3d_id, shape) in state.shapes3d.iter().enumerate_ids() {
        check_shape3d(shape, &has_box).map_err(at(paths.shape3d(shape3d_id)))?;
        has_box.push(shape_has_box(shape, &has_box));
    }

    for (pattern_id, pattern) in state.patterns.iter().enumerate_ids() {
        check_pattern(pattern).map_err(at(paths.pattern(pattern_id)))?;
    }

    for (step_id, step) in state.steps.iter().enumerate_ids() {
        check_step(step).map_err(at(paths.step(step_id)))?;
    }

    check_names(
        "the root parts",
        "part",
        state
            .root_node_ids
            .iter()
            .map(|node_id| state.nodes[node_id.to_usize_id()].name.as_str()),
    )?;

    let mut checked_node_ids = HashSet::new();

    for place in places {
        if !checked_node_ids.insert(place.node_id) {
            continue;
        }

        let node = &state.nodes[place.node_id.to_usize_id()];
        let path = place.path.join("/");

        check_names(
            &path,
            "part",
            node.child_node_ids
                .iter()
                .map(|node_id| state.nodes[node_id.to_usize_id()].name.as_str()),
        )?;

        for object_id in &node.child_object_ids {
            let step_ids = &state.objects[object_id.to_usize_id()].step_ids;

            check_names(
                &path,
                "step",
                step_ids
                    .iter()
                    .map(|step_id| state.steps[step_id.to_usize_id()].name()),
            )?;

            check_onto(state, &path, step_ids)?;
        }
    }

    Ok(())
}

/// Whether `shape` has a box. `has_box` reads whether each earlier shape has
/// one.
fn shape_has_box(shape: &SdfShape3d, has_box: &IdVec<BSdfShape3d, bool>) -> bool {
    let (shape_ids, _) = shape3d_references(shape);
    let mut boxed = shape_ids
        .iter()
        .map(|shape_id| has_box[shape_id.to_usize_id()]);

    match shape {
        SdfShape3d::HalfSpace { .. } => false,

        SdfShape3d::Intersect { .. } | SdfShape3d::SmoothIntersect { .. } => {
            boxed.any(|boxed| boxed)
        }

        SdfShape3d::SmoothSubtract { .. } | SdfShape3d::Subtract { .. } => {
            boxed.next().expect("a subtract has a base")
        }

        _ => boxed.all(|boxed| boxed),
    }
}

/// Errors unless the `onto` of each step in the list `step_ids` at `path` lists
/// `add` steps earlier in the list.
fn check_onto(state: &SdfState, path: &str, step_ids: &[U32Id<BSdfStep>]) -> Result<()> {
    for (index, step_id) in step_ids.iter().enumerate() {
        let step = &state.steps[step_id.to_usize_id()];

        let (call, onto_ids) = match step {
            SdfStep::Coat { onto_ids, .. } => ("coat", onto_ids),
            SdfStep::Paint { onto_ids, .. } => ("paint", onto_ids),
            SdfStep::Add { .. } | SdfStep::Carve { .. } | SdfStep::Set { .. } => continue,
        };

        for onto_id in onto_ids.iter().flatten() {
            let onto = &state.steps[onto_id.to_usize_id()];

            if !matches!(onto, SdfStep::Add { .. }) || !step_ids[..index].contains(onto_id) {
                return Err(Error::invalid(format!(
                    "{path}/{}: {call} onto must list add steps earlier in the list, not {:?}",
                    step.name(),
                    onto.name()
                )));
            }
        }
    }

    Ok(())
}

/// Errors unless every name of `kind` in the list `names` at `path` holds a
/// character and differs from the others.
fn check_names<'a>(path: &str, kind: &str, names: impl Iterator<Item = &'a str>) -> Result<()> {
    let mut seen = HashSet::new();

    for name in names {
        if name.is_empty() {
            return Err(Error::invalid(format!(
                "{path}: {kind} name must hold a character, not \"\""
            )));
        }

        if !seen.insert(name) {
            return Err(Error::invalid(format!(
                "{path}: {kind} name must be unique in its list, not {name:?} twice"
            )));
        }
    }

    Ok(())
}
