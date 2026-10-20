use crate::{
    BSdfMaterial, BSdfNode, BSdfObject, BSdfPattern, BSdfShades, BSdfShape2d, BSdfShape3d,
    BSdfStep, Error, Result, SdfEntryId, SdfMaterial, SdfNames, SdfNode, SdfObject, SdfPattern,
    SdfPropertyValue, SdfShades, SdfShape2d, SdfShape3d, SdfStep, SdfStepMaterial, SdfValue,
};
use branded_id::{IdVec, IteratorExt, U32Id, UsizeId};
use std::{collections::HashSet, iter};
use ty_math::{TyVector2F64, TyVector3F64};

/// A model's tables, whose entries reference each other by id.
/// [`validate`](SdfState::validate) checks every reference.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SdfState {
    /// The 3D shapes.
    pub shapes3d: IdVec<BSdfShape3d, SdfShape3d>,

    /// The 2D shapes.
    pub shapes2d: IdVec<BSdfShape2d, SdfShape2d>,

    /// The materials.
    pub materials: IdVec<BSdfMaterial, SdfMaterial>,

    /// The `shades` calls.
    pub shades: IdVec<BSdfShades, SdfShades>,

    /// The patterns.
    pub patterns: IdVec<BSdfPattern, SdfPattern>,

    /// The steps.
    pub steps: IdVec<BSdfStep, SdfStep>,

    /// The parts' objects.
    pub objects: IdVec<BSdfObject, SdfObject>,

    /// The parts' nodes.
    pub nodes: IdVec<BSdfNode, SdfNode>,

    /// The nodes at the top of the hierarchy.
    pub root_node_ids: Vec<U32Id<BSdfNode>>,

    /// The names a model reads the entries by when the document serves as a
    /// library.
    pub names: SdfNames,
}

impl SdfState {
    /// Checks the rules a document follows. Each [`Error`] variant reports one
    /// rule.
    pub fn validate(&self) -> Result<()> {
        for (table, length) in [
            ("shapes3d", self.shapes3d.len()),
            ("shapes2d", self.shapes2d.len()),
            ("materials", self.materials.len()),
            ("shades", self.shades.len()),
            ("patterns", self.patterns.len()),
            ("steps", self.steps.len()),
            ("objects", self.objects.len()),
            ("nodes", self.nodes.len()),
        ] {
            if u32::try_from(length).is_err() {
                return Err(Error::TooManyEntries { table, length });
            }
        }

        for (shape3d_id, shape) in self.shapes3d.iter().enumerate_ids() {
            check_shape3d(&self.entry_check(SdfEntryId::Shape3d(shape3d_id)), shape)?;
        }

        for (shape2d_id, shape) in self.shapes2d.iter().enumerate_ids() {
            check_shape2d(&self.entry_check(SdfEntryId::Shape2d(shape2d_id)), shape)?;
        }

        // A shade compares its index against its shades entry's count, and the
        // shades therefore validate first.
        for (shades_id, shades) in self.shades.iter().enumerate_ids() {
            check_shades(&self.entry_check(SdfEntryId::Shades(shades_id)), shades)?;
        }

        for (material_id, material) in self.materials.iter().enumerate_ids() {
            check_material(
                &self.entry_check(SdfEntryId::Material(material_id)),
                material_id,
                material,
            )?;
        }

        for (pattern_id, pattern) in self.patterns.iter().enumerate_ids() {
            check_pattern(&self.entry_check(SdfEntryId::Pattern(pattern_id)), pattern)?;
        }

        for (step_id, step) in self.steps.iter().enumerate_ids() {
            check_step(&self.entry_check(SdfEntryId::Step(step_id)), step)?;
        }

        for (object_id, object) in self.objects.iter().enumerate_ids() {
            check_object(&self.entry_check(SdfEntryId::Object(object_id)), object)?;
        }

        for (node_id, node) in self.nodes.iter().enumerate_ids() {
            check_node(&self.entry_check(SdfEntryId::Node(node_id)), node)?;
        }

        let mut root_node_ids = HashSet::with_capacity(self.root_node_ids.len());

        for &node_id in &self.root_node_ids {
            if node_id.to_usize_id() >= self.nodes.end() {
                return Err(Error::RootNodeMissing { node_id });
            }

            if !root_node_ids.insert(node_id) {
                return Err(Error::RootNodeRepeated { node_id });
            }
        }

        for (parent_node_id, node) in self.nodes.iter().enumerate_ids() {
            if let Some(&node_id) = node
                .child_node_ids
                .iter()
                .find(|node_id| root_node_ids.contains(node_id))
            {
                return Err(Error::RootNodeChild {
                    node_id,
                    parent_node_id,
                });
            }
        }

        let names = &self.names;

        check_names(
            "shapes3d",
            &names.shapes3d,
            self.shapes3d.end(),
            SdfEntryId::Shape3d,
        )?;

        check_names(
            "shapes2d",
            &names.shapes2d,
            self.shapes2d.end(),
            SdfEntryId::Shape2d,
        )?;

        check_names(
            "materials",
            &names.materials,
            self.materials.end(),
            SdfEntryId::Material,
        )?;

        check_names(
            "patterns",
            &names.patterns,
            self.patterns.end(),
            SdfEntryId::Pattern,
        )?;

        check_names("steps", &names.steps, self.steps.end(), SdfEntryId::Step)?;

        check_names("parts", &names.parts, self.nodes.end(), SdfEntryId::Node)
    }

    /// The check of the entry at `entry_id`.
    fn entry_check(&self, entry_id: SdfEntryId) -> EntryCheck<'_> {
        EntryCheck {
            state: self,
            entry_id,
        }
    }
}

/// Checks the numbers and references of one entry.
struct EntryCheck<'a> {
    state: &'a SdfState,

    entry_id: SdfEntryId,
}

impl EntryCheck<'_> {
    /// Errors on a number that is not finite.
    fn numbers(&self, numbers: impl IntoIterator<Item = f64>) -> Result<()> {
        match numbers.into_iter().find(|number| !number.is_finite()) {
            Some(number) => Err(Error::NonFiniteNumber {
                entry_id: self.entry_id,
                number,
            }),

            None => Ok(()),
        }
    }

    /// Errors on an option that holds a number that is not finite.
    fn optional_numbers(&self, numbers: impl IntoIterator<Item = Option<f64>>) -> Result<()> {
        self.numbers(numbers.into_iter().flatten())
    }

    /// Errors on a vector with a component that is not finite.
    fn vector2s(&self, vectors: impl IntoIterator<Item = TyVector2F64>) -> Result<()> {
        self.numbers(vectors.into_iter().flat_map(|vector| vector.to_array()))
    }

    /// Errors on a vector with a component that is not finite.
    fn vector3s(&self, vectors: impl IntoIterator<Item = TyVector3F64>) -> Result<()> {
        self.numbers(vectors.into_iter().flat_map(|vector| vector.to_array()))
    }

    /// Errors on a 3D shape that is not earlier in `shapes3d` when this entry
    /// is a 3D shape, or past the end of `shapes3d` otherwise.
    fn shape3d_ids(&self, shape3d_ids: impl IntoIterator<Item = U32Id<BSdfShape3d>>) -> Result<()> {
        let end_id = match self.entry_id {
            SdfEntryId::Shape3d(shape3d_id) => shape3d_id.to_usize_id(),
            _ => self.state.shapes3d.end(),
        };

        shape3d_ids.into_iter().try_for_each(|shape3d_id| {
            self.reference(SdfEntryId::Shape3d(shape3d_id), shape3d_id, end_id)
        })
    }

    /// Errors on a 2D shape that is not earlier in `shapes2d` when this entry
    /// is a 2D shape, or past the end of `shapes2d` otherwise.
    fn shape2d_ids(&self, shape2d_ids: impl IntoIterator<Item = U32Id<BSdfShape2d>>) -> Result<()> {
        let end_id = match self.entry_id {
            SdfEntryId::Shape2d(shape2d_id) => shape2d_id.to_usize_id(),
            _ => self.state.shapes2d.end(),
        };

        shape2d_ids.into_iter().try_for_each(|shape2d_id| {
            self.reference(SdfEntryId::Shape2d(shape2d_id), shape2d_id, end_id)
        })
    }

    /// Errors on a node that is not earlier in `nodes`.
    fn node_ids(&self, node_ids: impl IntoIterator<Item = U32Id<BSdfNode>>) -> Result<()> {
        let end_id = match self.entry_id {
            SdfEntryId::Node(node_id) => node_id.to_usize_id(),
            _ => self.state.nodes.end(),
        };

        node_ids
            .into_iter()
            .try_for_each(|node_id| self.reference(SdfEntryId::Node(node_id), node_id, end_id))
    }

    /// Errors on a material past the end of `materials`.
    fn material_ids(
        &self,
        material_ids: impl IntoIterator<Item = U32Id<BSdfMaterial>>,
    ) -> Result<()> {
        material_ids.into_iter().try_for_each(|material_id| {
            self.reference(
                SdfEntryId::Material(material_id),
                material_id,
                self.state.materials.end(),
            )
        })
    }

    /// Errors on an object past the end of `objects`.
    fn object_ids(&self, object_ids: impl IntoIterator<Item = U32Id<BSdfObject>>) -> Result<()> {
        object_ids.into_iter().try_for_each(|object_id| {
            self.reference(
                SdfEntryId::Object(object_id),
                object_id,
                self.state.objects.end(),
            )
        })
    }

    /// Errors on a pattern past the end of `patterns`.
    fn pattern_ids(&self, pattern_ids: impl IntoIterator<Item = U32Id<BSdfPattern>>) -> Result<()> {
        pattern_ids.into_iter().try_for_each(|pattern_id| {
            self.reference(
                SdfEntryId::Pattern(pattern_id),
                pattern_id,
                self.state.patterns.end(),
            )
        })
    }

    /// Errors on a shades entry past the end of `shades`.
    fn shades_ids(&self, shades_ids: impl IntoIterator<Item = U32Id<BSdfShades>>) -> Result<()> {
        shades_ids.into_iter().try_for_each(|shades_id| {
            self.reference(
                SdfEntryId::Shades(shades_id),
                shades_id,
                self.state.shades.end(),
            )
        })
    }

    /// Errors on a step past the end of `steps`.
    fn step_ids(&self, step_ids: impl IntoIterator<Item = U32Id<BSdfStep>>) -> Result<()> {
        step_ids.into_iter().try_for_each(|step_id| {
            self.reference(SdfEntryId::Step(step_id), step_id, self.state.steps.end())
        })
    }

    /// Errors unless `id` comes before `end_id`.
    fn reference<TBrand>(
        &self,
        referenced_entry_id: SdfEntryId,
        id: U32Id<TBrand>,
        end_id: UsizeId<TBrand>,
    ) -> Result<()> {
        if id.to_usize_id() < end_id {
            return Ok(());
        }

        Err(Error::Reference {
            entry_id: self.entry_id,
            referenced_entry_id,
        })
    }
}

/// Checks a 3D shape's numbers and references.
fn check_shape3d(check: &EntryCheck, shape: &SdfShape3d) -> Result<()> {
    match shape {
        SdfShape3d::Bend {
            shape_id,
            radius,
            pivot,
            ..
        } => {
            check.shape3d_ids([*shape_id])?;
            check.numbers([*radius])?;
            check.vector3s(*pivot)
        }

        SdfShape3d::Box { min, max, round } => {
            check.vector3s([*min, *max])?;
            check.optional_numbers([*round])
        }

        SdfShape3d::BoxFrame {
            min,
            max,
            thickness,
        } => {
            check.vector3s([*min, *max])?;
            check.numbers([*thickness])
        }

        SdfShape3d::Capsule { a, b, radius } => {
            check.vector3s([*a, *b])?;
            check.numbers([*radius])
        }

        SdfShape3d::Cone {
            a,
            b,
            radius_a,
            radius_b,
        }
        | SdfShape3d::RoundCone {
            a,
            b,
            radius_a,
            radius_b,
        } => {
            check.vector3s([*a, *b])?;
            check.numbers([*radius_a, *radius_b])
        }

        SdfShape3d::Cylinder {
            a,
            b,
            radius,
            round,
        } => {
            check.vector3s([*a, *b])?;
            check.numbers([*radius])?;
            check.optional_numbers([*round])
        }

        SdfShape3d::Displace {
            shape_id,
            amplitude,
            scale,
            octaves,
            seed,
        } => {
            check.shape3d_ids([*shape_id])?;
            check.numbers([*amplitude, *scale, *seed])?;
            check.optional_numbers([*octaves])
        }

        SdfShape3d::Elongate {
            shape_id,
            lengths,
            center,
        } => {
            check.shape3d_ids([*shape_id])?;
            check.vector3s(iter::once(*lengths).chain(*center))
        }

        SdfShape3d::Ellipsoid { center, radii } => check.vector3s([*center, *radii]),

        SdfShape3d::Extrude {
            profile_id,
            from,
            to,
            ..
        } => {
            check.shape2d_ids([*profile_id])?;
            check.numbers([*from, *to])
        }

        SdfShape3d::HalfSpace { at, .. } => check.numbers([*at]),

        SdfShape3d::Intersect { shape_ids } | SdfShape3d::Union { shape_ids } => {
            check.shape3d_ids(shape_ids.iter().copied())
        }

        SdfShape3d::Lathe { points, center, .. } => {
            check.vector2s(points.iter().copied())?;
            check.vector3s(*center)
        }

        SdfShape3d::Mirror {
            shape_id, center, ..
        } => {
            check.shape3d_ids([*shape_id])?;
            check.vector3s(*center)
        }

        SdfShape3d::Octahedron { center, radius } | SdfShape3d::Sphere { center, radius } => {
            check.vector3s([*center])?;
            check.numbers([*radius])
        }

        SdfShape3d::Offset { shape_id, distance } => {
            check.shape3d_ids([*shape_id])?;
            check.numbers([*distance])
        }

        SdfShape3d::Orient {
            shape_id,
            from,
            to,
            pivot,
        } => {
            check.shape3d_ids([*shape_id])?;
            check.vector3s([*from, *to].into_iter().chain(*pivot))
        }

        SdfShape3d::Pyramid {
            base_center,
            width,
            height,
        } => {
            check.vector3s([*base_center])?;
            check.numbers([*width, *height])
        }

        SdfShape3d::Repeat {
            shape_id,
            step,
            count,
        } => {
            check.shape3d_ids([*shape_id])?;
            check.vector3s([*step, *count])
        }

        SdfShape3d::RepeatPolar {
            shape_id,
            count,
            center,
            ..
        } => {
            check.shape3d_ids([*shape_id])?;
            check.numbers([*count])?;
            check.vector3s(*center)
        }

        SdfShape3d::Revolve {
            profile_id, center, ..
        } => {
            check.shape2d_ids([*profile_id])?;
            check.vector3s(*center)
        }

        SdfShape3d::Rotate {
            shape_id,
            degrees,
            pivot,
            ..
        } => {
            check.shape3d_ids([*shape_id])?;
            check.numbers([*degrees])?;
            check.vector3s(*pivot)
        }

        SdfShape3d::Scale {
            shape_id,
            factor,
            pivot,
        } => {
            check.shape3d_ids([*shape_id])?;
            check.vector3s(iter::once(*factor).chain(*pivot))
        }

        SdfShape3d::Shell {
            shape_id,
            thickness,
        } => {
            check.shape3d_ids([*shape_id])?;
            check.numbers([*thickness])
        }

        SdfShape3d::SmoothIntersect { radius, shape_ids }
        | SdfShape3d::SmoothUnion { radius, shape_ids } => {
            check.numbers([*radius])?;
            check.shape3d_ids(shape_ids.iter().copied())
        }

        SdfShape3d::SmoothSubtract {
            radius,
            base_id,
            cutter_ids,
        } => {
            check.numbers([*radius])?;
            check.shape3d_ids(iter::once(*base_id).chain(cutter_ids.iter().copied()))
        }

        SdfShape3d::Subtract {
            base_id,
            cutter_ids,
        } => check.shape3d_ids(iter::once(*base_id).chain(cutter_ids.iter().copied())),

        SdfShape3d::Sweep {
            profile_id,
            points,
            scales,
            up,
        } => {
            check.shape2d_ids([*profile_id])?;
            check.vector3s(points.iter().copied())?;
            check.numbers(scales.iter().flatten().copied())?;
            check.vector3s(*up)
        }

        SdfShape3d::Torus {
            center,
            ring_radius,
            tube_radius,
            from,
            to,
            ..
        } => {
            check.vector3s([*center])?;
            check.numbers([*ring_radius, *tube_radius])?;
            check.optional_numbers([*from, *to])
        }

        SdfShape3d::Translate { shape_id, offset } => {
            check.shape3d_ids([*shape_id])?;
            check.vector3s([*offset])
        }

        SdfShape3d::Twist {
            shape_id,
            degrees_per_meter,
            center,
            ..
        } => {
            check.shape3d_ids([*shape_id])?;
            check.numbers([*degrees_per_meter])?;
            check.vector3s(*center)
        }
    }
}

/// Checks a 2D shape's numbers and references.
fn check_shape2d(check: &EntryCheck, shape: &SdfShape2d) -> Result<()> {
    match shape {
        SdfShape2d::Arc {
            center,
            radius,
            from_degrees,
            to_degrees,
            width,
            ..
        } => {
            check.vector2s([*center])?;
            check.numbers([*radius, *from_degrees, *to_degrees, *width])
        }

        SdfShape2d::Arch { min, max } => check.vector2s([*min, *max]),

        SdfShape2d::Circle { center, radius } => {
            check.vector2s([*center])?;
            check.numbers([*radius])
        }

        SdfShape2d::Ellipse { center, radii } => check.vector2s([*center, *radii]),

        SdfShape2d::Intersect { shape_ids } | SdfShape2d::Union { shape_ids } => {
            check.shape2d_ids(shape_ids.iter().copied())
        }

        SdfShape2d::Mirror {
            shape_id, center, ..
        } => {
            check.shape2d_ids([*shape_id])?;
            check.vector2s(*center)
        }

        SdfShape2d::Ngon {
            center,
            sides,
            radius,
        } => {
            check.vector2s([*center])?;
            check.numbers([*sides, *radius])
        }

        SdfShape2d::Offset { shape_id, distance } => {
            check.shape2d_ids([*shape_id])?;
            check.numbers([*distance])
        }

        SdfShape2d::Polygon { points } => check.vector2s(points.iter().copied()),

        SdfShape2d::Polyline { points, width } => {
            check.vector2s(points.iter().copied())?;
            check.numbers([*width])
        }

        SdfShape2d::Rect {
            min,
            max,
            chamfer,
            round,
        } => {
            check.vector2s([*min, *max])?;
            check.optional_numbers([*chamfer, *round])
        }

        SdfShape2d::Repeat {
            shape_id,
            step,
            count,
        } => {
            check.shape2d_ids([*shape_id])?;
            check.vector2s([*step, *count])
        }

        SdfShape2d::RepeatPolar {
            shape_id,
            count,
            center,
        } => {
            check.shape2d_ids([*shape_id])?;
            check.numbers([*count])?;
            check.vector2s(*center)
        }

        SdfShape2d::Rotate {
            shape_id,
            degrees,
            pivot,
        } => {
            check.shape2d_ids([*shape_id])?;
            check.numbers([*degrees])?;
            check.vector2s(*pivot)
        }

        SdfShape2d::Scale {
            shape_id,
            factor,
            pivot,
        } => {
            check.shape2d_ids([*shape_id])?;
            check.vector2s(iter::once(*factor).chain(*pivot))
        }

        SdfShape2d::Sector {
            center,
            radius,
            from_degrees,
            to_degrees,
        } => {
            check.vector2s([*center])?;
            check.numbers([*radius, *from_degrees, *to_degrees])
        }

        SdfShape2d::Shell {
            shape_id,
            thickness,
        } => {
            check.shape2d_ids([*shape_id])?;
            check.numbers([*thickness])
        }

        SdfShape2d::SmoothIntersect { radius, shape_ids }
        | SdfShape2d::SmoothUnion { radius, shape_ids } => {
            check.numbers([*radius])?;
            check.shape2d_ids(shape_ids.iter().copied())
        }

        SdfShape2d::SmoothSubtract {
            radius,
            base_id,
            cutter_ids,
        } => {
            check.numbers([*radius])?;
            check.shape2d_ids(iter::once(*base_id).chain(cutter_ids.iter().copied()))
        }

        SdfShape2d::Spline { points, width } => {
            check.vector2s(points.iter().copied())?;
            check.optional_numbers([*width])
        }

        SdfShape2d::Star {
            center,
            points,
            outer_radius,
            inner_radius,
        } => {
            check.vector2s([*center])?;
            check.numbers([*points, *outer_radius, *inner_radius])
        }

        SdfShape2d::Subtract {
            base_id,
            cutter_ids,
        } => check.shape2d_ids(iter::once(*base_id).chain(cutter_ids.iter().copied())),

        SdfShape2d::Translate { shape_id, offset } => {
            check.shape2d_ids([*shape_id])?;
            check.vector2s([*offset])
        }

        SdfShape2d::Vesica { a, b, width } => {
            check.vector2s([*a, *b])?;
            check.numbers([*width])
        }
    }
}

/// Checks a shades entry's numbers and base.
fn check_shades(check: &EntryCheck, shades: &SdfShades) -> Result<()> {
    check.material_ids([shades.base_id])?;
    check.numbers([shades.count])?;
    check.optional_numbers([shades.spread])
}

/// Checks a material's properties, or a shade's place in its shades entry.
fn check_material(
    check: &EntryCheck,
    material_id: U32Id<BSdfMaterial>,
    material: &SdfMaterial,
) -> Result<()> {
    match material {
        SdfMaterial::Material { properties } => {
            let mut names = HashSet::with_capacity(properties.len());

            for property in properties {
                if !names.insert(property.name.as_str()) {
                    return Err(Error::RepeatedPropertyName {
                        material_id,
                        name: property.name.clone(),
                    });
                }

                match &property.value {
                    SdfPropertyValue::Bool(_) | SdfPropertyValue::Text(_) => {}

                    SdfPropertyValue::Int(number) | SdfPropertyValue::Number(number) => {
                        check.numbers([*number])?
                    }

                    SdfPropertyValue::IntArray(numbers)
                    | SdfPropertyValue::NumberArray(numbers) => {
                        check.numbers(numbers.iter().copied())?
                    }

                    SdfPropertyValue::Json(value) => check_value(check, material_id, value)?,
                }
            }

            Ok(())
        }

        SdfMaterial::Shade { shades_id, index } => {
            check.shades_ids([*shades_id])?;

            let shades = &check.state.shades[shades_id.to_usize_id()];

            if f64::from(*index) >= shades.count {
                return Err(Error::ShadeIndex {
                    material_id,
                    shades_id: *shades_id,
                    index: *index,
                    count: shades.count,
                });
            }

            if shades.base_id >= material_id {
                return Err(Error::ShadeBase {
                    material_id,
                    shades_id: *shades_id,
                    base_id: shades.base_id,
                });
            }

            Ok(())
        }
    }
}

/// Checks a `json` value's numbers and keys.
fn check_value(
    check: &EntryCheck,
    material_id: U32Id<BSdfMaterial>,
    value: &SdfValue,
) -> Result<()> {
    match value {
        SdfValue::Array(values) => values
            .iter()
            .try_for_each(|value| check_value(check, material_id, value)),

        SdfValue::Bool(_) | SdfValue::Null | SdfValue::Text(_) => Ok(()),

        SdfValue::Number(number) => check.numbers([*number]),

        SdfValue::Object(map) => {
            let mut keys = HashSet::with_capacity(map.entries().len());

            for entry in map.entries() {
                if !keys.insert(entry.key.as_str()) {
                    return Err(Error::RepeatedMapKey {
                        material_id,
                        key: entry.key.clone(),
                    });
                }

                check_value(check, material_id, &entry.value)?;
            }

            Ok(())
        }
    }
}

/// Checks a pattern's numbers and materials.
fn check_pattern(check: &EntryCheck, pattern: &SdfPattern) -> Result<()> {
    match pattern {
        SdfPattern::Bands {
            material_ids,
            period,
            warp,
            seed,
            ..
        } => {
            check.material_ids(material_ids.iter().copied())?;
            check.numbers([*period])?;
            check.optional_numbers([*warp, *seed])
        }

        SdfPattern::Cells {
            material_ids,
            size,
            seed,
            border,
        } => {
            check.material_ids(
                material_ids
                    .iter()
                    .copied()
                    .chain(border.map(|border| border.material_id)),
            )?;
            check.numbers(
                [*size, *seed]
                    .into_iter()
                    .chain(border.map(|border| border.width)),
            )
        }

        SdfPattern::Checker { material_ids, size } => {
            check.material_ids(material_ids.iter().copied())?;
            check.numbers([*size])
        }

        SdfPattern::Gradient {
            material_ids,
            from,
            to,
            warp,
            seed,
            ..
        } => {
            check.material_ids(material_ids.iter().copied())?;
            check.numbers([*from, *to])?;
            check.optional_numbers([*warp, *seed])
        }

        SdfPattern::Grain {
            material_ids,
            period,
            warp,
            seed,
            ..
        } => {
            check.material_ids(material_ids.iter().copied())?;
            check.numbers([*period, *seed])?;
            check.optional_numbers([*warp])
        }

        SdfPattern::Noise {
            material_ids,
            scale,
            octaves,
            seed,
        } => {
            check.material_ids(material_ids.iter().copied())?;
            check.numbers([*scale, *seed])?;
            check.optional_numbers([*octaves])
        }

        SdfPattern::Speckle {
            base_id,
            accent_ids,
            size,
            density,
            seed,
        } => {
            check.material_ids(iter::once(*base_id).chain(accent_ids.iter().copied()))?;
            check.numbers([*size, *density, *seed])
        }
    }
}

/// Checks a step's numbers, shapes, material, and steps.
fn check_step(check: &EntryCheck, step: &SdfStep) -> Result<()> {
    match step {
        SdfStep::Add {
            shape_id, material, ..
        } => {
            check.shape3d_ids([*shape_id])?;
            check_step_material(check, *material)
        }

        SdfStep::Carve { shape_id, .. } => check.shape3d_ids([*shape_id]),

        SdfStep::Coat {
            material,
            depth,
            within_id,
            onto_ids,
            ..
        } => {
            check_step_material(check, *material)?;
            check.numbers([*depth])?;
            check.shape3d_ids(*within_id)?;
            check.step_ids(onto_ids.iter().flatten().copied())
        }

        SdfStep::Paint {
            shape_id,
            material,
            onto_ids,
            ..
        } => {
            check.shape3d_ids([*shape_id])?;
            check_step_material(check, *material)?;
            check.step_ids(onto_ids.iter().flatten().copied())
        }

        SdfStep::Set {
            points,
            size,
            material,
            ..
        } => {
            check.vector3s(points.iter().copied())?;
            check.numbers([*size])?;
            check_step_material(check, *material)
        }
    }
}

/// Checks the material or pattern a step writes.
fn check_step_material(check: &EntryCheck, material: SdfStepMaterial) -> Result<()> {
    match material {
        SdfStepMaterial::Material(material_id) => check.material_ids([material_id]),
        SdfStepMaterial::Pattern(pattern_id) => check.pattern_ids([pattern_id]),
    }
}

/// Checks an object's steps.
fn check_object(check: &EntryCheck, object: &SdfObject) -> Result<()> {
    check.step_ids(object.step_ids.iter().copied())
}

/// Checks a node's numbers and children.
fn check_node(check: &EntryCheck, node: &SdfNode) -> Result<()> {
    check.vector3s(node.pivot.into_iter().chain(node.offset))?;
    check.object_ids(node.child_object_ids.iter().copied())?;
    check.node_ids(node.child_node_ids.iter().copied())
}

/// Checks that each of `names` appears once and points before `end_id`.
fn check_names<TBrand>(
    table: &'static str,
    names: &[(String, U32Id<TBrand>)],
    end_id: UsizeId<TBrand>,
    to_entry_id: fn(U32Id<TBrand>) -> SdfEntryId,
) -> Result<()> {
    let mut seen = HashSet::with_capacity(names.len());

    for (name, named_id) in names {
        if !seen.insert(name.as_str()) {
            return Err(Error::RepeatedName {
                table,
                name: name.clone(),
            });
        }

        if named_id.to_usize_id() >= end_id {
            return Err(Error::NameMissing {
                table,
                name: name.clone(),
                entry_id: to_entry_id(*named_id),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        Error, SdfEntryId, SdfMap, SdfMapEntry, SdfMaterial, SdfNames, SdfNode, SdfObject,
        SdfPattern, SdfProperty, SdfPropertyValue, SdfShades, SdfShape3d, SdfState, SdfStep,
        SdfStepMaterial, SdfValue,
    };
    use branded_id::{IdVec, U32Id};
    use ty_math::{TyAxis3, TyVector3F64};

    /// A seat painted with grain over two shades of one material, as one part.
    fn state() -> SdfState {
        SdfState {
            shapes3d: IdVec::from(vec![SdfShape3d::Box {
                min: TyVector3F64::ZERO,
                max: TyVector3F64::ONE,
                round: None,
            }]),
            materials: IdVec::from(vec![
                SdfMaterial::Material {
                    properties: vec![SdfProperty {
                        name: "baseColor".to_owned(),
                        value: SdfPropertyValue::Text("#8A5A2B".to_owned()),
                    }],
                },
                SdfMaterial::Shade {
                    shades_id: U32Id::from_u32(0),
                    index: 0,
                },
                SdfMaterial::Shade {
                    shades_id: U32Id::from_u32(0),
                    index: 1,
                },
            ]),
            shades: IdVec::from(vec![SdfShades {
                base_id: U32Id::from_u32(0),
                count: 2.0,
                spread: None,
            }]),
            patterns: IdVec::from(vec![SdfPattern::Grain {
                material_ids: vec![U32Id::from_u32(1), U32Id::from_u32(2)],
                axis: TyAxis3::Y,
                period: 0.02,
                warp: None,
                seed: 1.0,
            }]),
            steps: IdVec::from(vec![SdfStep::Add {
                name: "seat".to_owned(),
                shape_id: U32Id::from_u32(0),
                material: SdfStepMaterial::Pattern(U32Id::from_u32(0)),
            }]),
            objects: IdVec::from(vec![SdfObject {
                name: "chair".to_owned(),
                step_ids: vec![U32Id::from_u32(0)],
            }]),
            nodes: IdVec::from(vec![SdfNode {
                name: "chair".to_owned(),
                pivot: None,
                offset: None,
                rotation: Vec::new(),
                child_object_ids: vec![U32Id::from_u32(0)],
                child_node_ids: Vec::new(),
            }]),
            root_node_ids: vec![U32Id::from_u32(0)],
            ..SdfState::default()
        }
    }

    /// The state with a second node holding the first.
    fn state_with_parent_node() -> SdfState {
        let mut state = state();

        state.nodes.push(SdfNode {
            name: "room".to_owned(),
            pivot: None,
            offset: None,
            rotation: Vec::new(),
            child_object_ids: Vec::new(),
            child_node_ids: vec![U32Id::from_u32(0)],
        });

        state.root_node_ids = vec![U32Id::from_u32(1)];

        state
    }

    /// The state with `value` as a second property of its first material.
    fn state_with_property(value: SdfPropertyValue) -> SdfState {
        let mut state = state();

        let SdfMaterial::Material { properties } =
            &mut state.materials[U32Id::from_u32(0).to_usize_id()]
        else {
            unreachable!("the first material holds properties");
        };

        properties.push(SdfProperty {
            name: "custom".to_owned(),
            value,
        });

        state
    }

    #[test]
    fn a_state_following_the_rules_validates() {
        state().validate().unwrap();

        state_with_parent_node().validate().unwrap();

        SdfState::default().validate().unwrap();
    }

    #[test]
    fn a_reference_past_the_end_of_its_table_errors() {
        let mut state = state();

        state.objects[U32Id::from_u32(0).to_usize_id()]
            .step_ids
            .push(U32Id::from_u32(1));

        let error = state.validate().unwrap_err();

        assert_eq!(
            error,
            Error::Reference {
                entry_id: SdfEntryId::Object(U32Id::from_u32(0)),
                referenced_entry_id: SdfEntryId::Step(U32Id::from_u32(1)),
            }
        );

        assert_eq!(
            error.to_string(),
            "objects[0] references steps[1], past the end of steps"
        );
    }

    #[test]
    fn a_shape_referencing_itself_or_a_later_shape_errors() {
        for shape_id in [U32Id::from_u32(1), U32Id::from_u32(2)] {
            let mut state = state();

            state.shapes3d.push(SdfShape3d::Offset {
                shape_id,
                distance: 0.1,
            });

            state.shapes3d.push(SdfShape3d::Offset {
                shape_id: U32Id::from_u32(0),
                distance: 0.1,
            });

            let error = state.validate().unwrap_err();

            assert_eq!(
                error,
                Error::Reference {
                    entry_id: SdfEntryId::Shape3d(U32Id::from_u32(1)),
                    referenced_entry_id: SdfEntryId::Shape3d(shape_id),
                }
            );

            assert!(
                error.to_string().ends_with("which does not come before it"),
                "{error}"
            );
        }
    }

    #[test]
    fn a_node_referencing_a_later_node_errors() {
        let mut state = state_with_parent_node();

        state.nodes = state.nodes.into_iter().rev().collect();

        state.nodes[U32Id::from_u32(0).to_usize_id()].child_node_ids = vec![U32Id::from_u32(1)];

        state.root_node_ids = vec![U32Id::from_u32(0)];

        assert_eq!(
            state.validate(),
            Err(Error::Reference {
                entry_id: SdfEntryId::Node(U32Id::from_u32(0)),
                referenced_entry_id: SdfEntryId::Node(U32Id::from_u32(1)),
            })
        );
    }

    #[test]
    fn a_non_finite_number_errors() {
        for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut state = state();

            state.shapes3d.push(SdfShape3d::Sphere {
                center: TyVector3F64::new(0.0, number, 0.0),
                radius: 1.0,
            });

            assert!(matches!(
                state.validate(),
                Err(Error::NonFiniteNumber { entry_id, .. })
                    if entry_id == SdfEntryId::Shape3d(U32Id::from_u32(1))
            ));
        }
    }

    #[test]
    fn a_non_finite_property_number_errors() {
        let state = state_with_property(SdfPropertyValue::Json(SdfValue::Array(vec![
            SdfValue::Number(f64::NAN),
        ])));

        assert!(matches!(
            state.validate(),
            Err(Error::NonFiniteNumber { entry_id, .. })
                if entry_id == SdfEntryId::Material(U32Id::from_u32(0))
        ));
    }

    #[test]
    fn a_shade_index_reaching_its_count_errors() {
        let mut state = state();

        state.shades[U32Id::from_u32(0).to_usize_id()].count = 1.0;

        assert_eq!(
            state.validate(),
            Err(Error::ShadeIndex {
                material_id: U32Id::from_u32(2),
                shades_id: U32Id::from_u32(0),
                index: 1,
                count: 1.0,
            })
        );
    }

    #[test]
    fn a_shade_whose_base_does_not_come_before_it_errors() {
        let mut state = state();

        state.shades[U32Id::from_u32(0).to_usize_id()].base_id = U32Id::from_u32(1);

        assert_eq!(
            state.validate(),
            Err(Error::ShadeBase {
                material_id: U32Id::from_u32(1),
                shades_id: U32Id::from_u32(0),
                base_id: U32Id::from_u32(1),
            })
        );
    }

    #[test]
    fn a_repeated_property_name_errors() {
        let mut state = state_with_property(SdfPropertyValue::Bool(true));

        let SdfMaterial::Material { properties } =
            &mut state.materials[U32Id::from_u32(0).to_usize_id()]
        else {
            unreachable!("the first material holds properties");
        };

        properties[1].name = "baseColor".to_owned();

        assert_eq!(
            state.validate(),
            Err(Error::RepeatedPropertyName {
                material_id: U32Id::from_u32(0),
                name: "baseColor".to_owned(),
            })
        );
    }

    #[test]
    fn a_repeated_json_key_errors() {
        let entry = SdfMapEntry {
            key: "k".to_owned(),
            value: SdfValue::Null,
        };

        let state =
            state_with_property(SdfPropertyValue::Json(SdfValue::Object(SdfMap::new(vec![
                entry.clone(),
                entry,
            ]))));

        assert_eq!(
            state.validate(),
            Err(Error::RepeatedMapKey {
                material_id: U32Id::from_u32(0),
                key: "k".to_owned(),
            })
        );
    }

    #[test]
    fn root_nodes_are_unique_present_and_parentless() {
        let mut state = state();

        state.root_node_ids.push(U32Id::from_u32(0));

        assert_eq!(
            state.validate(),
            Err(Error::RootNodeRepeated {
                node_id: U32Id::from_u32(0)
            })
        );

        state.root_node_ids = vec![U32Id::from_u32(1)];

        assert_eq!(
            state.validate(),
            Err(Error::RootNodeMissing {
                node_id: U32Id::from_u32(1)
            })
        );

        let mut state = state_with_parent_node();

        state.root_node_ids.push(U32Id::from_u32(0));

        assert_eq!(
            state.validate(),
            Err(Error::RootNodeChild {
                node_id: U32Id::from_u32(0),
                parent_node_id: U32Id::from_u32(1),
            })
        );
    }

    #[test]
    fn a_name_points_into_its_table() {
        let mut state = state();

        state.names = SdfNames {
            materials: vec![
                ("walnut".to_owned(), U32Id::from_u32(0)),
                ("wood".to_owned(), U32Id::from_u32(0)),
            ],
            parts: vec![("chair".to_owned(), U32Id::from_u32(0))],
            ..SdfNames::default()
        };

        state.validate().unwrap();

        state
            .names
            .parts
            .push(("arm".to_owned(), U32Id::from_u32(1)));

        let error = state.validate().unwrap_err();

        assert_eq!(
            error,
            Error::NameMissing {
                table: "parts",
                name: "arm".to_owned(),
                entry_id: SdfEntryId::Node(U32Id::from_u32(1)),
            }
        );

        assert_eq!(
            error.to_string(),
            "names.parts gives `arm` to nodes[1], past the end of nodes"
        );
    }

    #[test]
    fn a_repeated_name_errors() {
        let mut state = state();

        state.names.steps = vec![("seat".to_owned(), U32Id::from_u32(0)); 2];

        let error = state.validate().unwrap_err();

        assert_eq!(
            error,
            Error::RepeatedName {
                table: "steps",
                name: "seat".to_owned(),
            }
        );

        assert_eq!(error.to_string(), "names.steps holds `seat` twice");
    }
}
