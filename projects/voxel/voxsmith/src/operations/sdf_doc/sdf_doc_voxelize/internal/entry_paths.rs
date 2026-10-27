use crate::operations::sdf_doc::{SdfPlace, shape2d_references, shape3d_references};
use branded_id::{IdVec, U32Id};
use sdfcore::{
    BSdfMaterial, BSdfPattern, BSdfShades, BSdfShape2d, BSdfShape3d, BSdfStep, SdfEntryId,
    SdfMaterial, SdfPattern, SdfState, SdfStep, SdfStepMaterial,
};

/// The path of part names and step name of the first step that reaches each
/// entry. An error about an entry starts with the entry's path.
#[derive(Clone, Debug, Default)]
pub struct EntryPaths {
    materials: IdVec<BSdfMaterial, Option<String>>,

    shades: IdVec<BSdfShades, Option<String>>,

    shapes3d: IdVec<BSdfShape3d, Option<String>>,

    shapes2d: IdVec<BSdfShape2d, Option<String>>,

    patterns: IdVec<BSdfPattern, Option<String>>,

    steps: IdVec<BSdfStep, Option<String>>,
}

impl EntryPaths {
    /// The paths of the entries of `state`, whose places are `places`.
    pub fn new(state: &SdfState, places: &[SdfPlace]) -> Self {
        let mut paths = Self {
            materials: state.materials.iter().map(|_| None).collect(),
            shades: state.shades.iter().map(|_| None).collect(),
            shapes3d: state.shapes3d.iter().map(|_| None).collect(),
            shapes2d: state.shapes2d.iter().map(|_| None).collect(),
            patterns: state.patterns.iter().map(|_| None).collect(),
            steps: state.steps.iter().map(|_| None).collect(),
        };

        for place in places {
            let node = &state.nodes[place.node_id.to_usize_id()];

            for object_id in &node.child_object_ids {
                for &step_id in &state.objects[object_id.to_usize_id()].step_ids {
                    let step = &state.steps[step_id.to_usize_id()];
                    let path = format!("{}/{}", place.path.join("/"), step.name());

                    let (shape_ids, material) = match step {
                        SdfStep::Add {
                            shape_id, material, ..
                        }
                        | SdfStep::Paint {
                            shape_id, material, ..
                        } => (vec![*shape_id], Some(*material)),

                        SdfStep::Carve { shape_id, .. } => (vec![*shape_id], None),

                        SdfStep::Coat {
                            material,
                            within_id,
                            ..
                        } => (within_id.iter().copied().collect(), Some(*material)),

                        SdfStep::Set { material, .. } => (Vec::new(), Some(*material)),
                    };

                    for shape_id in shape_ids {
                        paths.reach3d(state, shape_id, &path);
                    }

                    match material {
                        Some(SdfStepMaterial::Material(material_id)) => {
                            paths.reach_material(state, material_id, &path);
                        }

                        Some(SdfStepMaterial::Pattern(pattern_id)) => {
                            paths.patterns[pattern_id.to_usize_id()]
                                .get_or_insert_with(|| path.clone());

                            let pattern = &state.patterns[pattern_id.to_usize_id()];

                            for material_id in pattern_material_ids(pattern) {
                                paths.reach_material(state, material_id, &path);
                            }
                        }

                        None => {}
                    }

                    paths.steps[step_id.to_usize_id()].get_or_insert(path);
                }
            }
        }

        paths
    }

    /// Where an error about the material at `material_id` points.
    pub fn material(&self, material_id: U32Id<BSdfMaterial>) -> String {
        located(
            &self.materials[material_id.to_usize_id()],
            SdfEntryId::Material(material_id),
        )
    }

    /// Where an error about the `shades` call at `shades_id` points.
    pub fn shades(&self, shades_id: U32Id<BSdfShades>) -> String {
        located(
            &self.shades[shades_id.to_usize_id()],
            SdfEntryId::Shades(shades_id),
        )
    }

    /// Where an error about the 3D shape at `shape3d_id` points.
    pub fn shape3d(&self, shape3d_id: U32Id<BSdfShape3d>) -> String {
        located(
            &self.shapes3d[shape3d_id.to_usize_id()],
            SdfEntryId::Shape3d(shape3d_id),
        )
    }

    /// Where an error about the 2D shape at `shape2d_id` points.
    pub fn shape2d(&self, shape2d_id: U32Id<BSdfShape2d>) -> String {
        located(
            &self.shapes2d[shape2d_id.to_usize_id()],
            SdfEntryId::Shape2d(shape2d_id),
        )
    }

    /// Where an error about the pattern at `pattern_id` points.
    pub fn pattern(&self, pattern_id: U32Id<BSdfPattern>) -> String {
        located(
            &self.patterns[pattern_id.to_usize_id()],
            SdfEntryId::Pattern(pattern_id),
        )
    }

    /// Where an error about the step at `step_id` points.
    pub fn step(&self, step_id: U32Id<BSdfStep>) -> String {
        located(
            &self.steps[step_id.to_usize_id()],
            SdfEntryId::Step(step_id),
        )
    }

    /// Records `path` for the material at `material_id` and, for a shade, its
    /// `shades` call and base material, unless an earlier step reached them.
    fn reach_material(&mut self, state: &SdfState, material_id: U32Id<BSdfMaterial>, path: &str) {
        let entry = &mut self.materials[material_id.to_usize_id()];

        if entry.is_some() {
            return;
        }

        *entry = Some(path.to_string());

        if let SdfMaterial::Shade { shades_id, .. } = &state.materials[material_id.to_usize_id()] {
            self.shades[shades_id.to_usize_id()].get_or_insert_with(|| path.to_string());

            let base_id = state.shades[shades_id.to_usize_id()].base_id;
            self.reach_material(state, base_id, path);
        }
    }

    /// Records `path` for the 3D shape at `shape3d_id` and the shapes it
    /// references, unless an earlier step reached them.
    fn reach3d(&mut self, state: &SdfState, shape3d_id: U32Id<BSdfShape3d>, path: &str) {
        let entry = &mut self.shapes3d[shape3d_id.to_usize_id()];

        if entry.is_some() {
            return;
        }

        *entry = Some(path.to_string());

        let (shape_ids, profile_id) = shape3d_references(&state.shapes3d[shape3d_id.to_usize_id()]);

        for shape_id in shape_ids {
            self.reach3d(state, shape_id, path);
        }

        if let Some(profile_id) = profile_id {
            self.reach2d(state, profile_id, path);
        }
    }

    /// Records `path` for the 2D shape at `shape2d_id` and the shapes it
    /// references, unless an earlier step reached them.
    fn reach2d(&mut self, state: &SdfState, shape2d_id: U32Id<BSdfShape2d>, path: &str) {
        let entry = &mut self.shapes2d[shape2d_id.to_usize_id()];

        if entry.is_some() {
            return;
        }

        *entry = Some(path.to_string());

        for shape_id in shape2d_references(&state.shapes2d[shape2d_id.to_usize_id()]) {
            self.reach2d(state, shape_id, path);
        }
    }
}

/// The materials `pattern` picks among.
fn pattern_material_ids(pattern: &SdfPattern) -> Vec<U32Id<BSdfMaterial>> {
    match pattern {
        SdfPattern::Bands { material_ids, .. }
        | SdfPattern::Checker { material_ids, .. }
        | SdfPattern::Gradient { material_ids, .. }
        | SdfPattern::Grain { material_ids, .. }
        | SdfPattern::Noise { material_ids, .. } => material_ids.clone(),

        SdfPattern::Bricks {
            material_ids,
            border,
            ..
        }
        | SdfPattern::Cells {
            material_ids,
            border,
            ..
        } => material_ids
            .iter()
            .copied()
            .chain(border.map(|border| border.material_id))
            .collect(),

        SdfPattern::Speckle {
            base_id,
            accent_ids,
            ..
        } => [*base_id]
            .into_iter()
            .chain(accent_ids.iter().copied())
            .collect(),
    }
}

/// `path`, or the entry's place in its table when no step reaches it.
fn located(path: &Option<String>, entry_id: SdfEntryId) -> String {
    match path {
        Some(path) => path.clone(),
        None => entry_id.to_string(),
    }
}
