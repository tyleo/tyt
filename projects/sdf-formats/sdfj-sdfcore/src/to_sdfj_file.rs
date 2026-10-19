use branded_id::U32Id;
use sdfcore::{
    SdfAxes2d, SdfAxes3d, SdfCaps, SdfMain, SdfMaterial, SdfNode, SdfObject, SdfPattern,
    SdfPropertyValue, SdfShades, SdfShape2d, SdfShape3d, SdfSide, SdfStep, SdfStepMaterial,
    SdfValue,
};
use sdfj::{
    SDFJ_VERSION, SdfjAxes2d, SdfjAxes3d, SdfjAxis, SdfjAxisRotation, SdfjCaps, SdfjFile,
    SdfjIntValue, SdfjMap, SdfjMapEntry, SdfjMaterial, SdfjNames, SdfjNode, SdfjObject,
    SdfjPattern, SdfjPropertyValue, SdfjShades, SdfjShape2d, SdfjShape3d, SdfjSide, SdfjStep,
    SdfjTaggedValue, SdfjValue,
};
use ty_math::{TyAxis3, TyVector2F64, TyVector3F64};

/// Encodes an [`SdfMain`] as an [`SdfjFile`] of [`SDFJ_VERSION`]. Every
/// entry keeps its place, so the document reads back as the same state.
pub fn to_sdfj_file(main: &SdfMain) -> SdfjFile {
    let state = main.state();

    SdfjFile {
        version: SDFJ_VERSION,
        shapes3d: state
            .shapes3d
            .iter()
            .map(sdfj_shape3d_from_sdf_shape3d)
            .collect(),
        shapes2d: state
            .shapes2d
            .iter()
            .map(sdfj_shape2d_from_sdf_shape2d)
            .collect(),
        materials: state
            .materials
            .iter()
            .map(sdfj_material_from_sdf_material)
            .collect(),
        shades: state
            .shades
            .iter()
            .map(sdfj_shades_from_sdf_shades)
            .collect(),
        patterns: state
            .patterns
            .iter()
            .map(sdfj_pattern_from_sdf_pattern)
            .collect(),
        steps: state.steps.iter().map(sdfj_step_from_sdf_step).collect(),
        objects: state
            .objects
            .iter()
            .map(sdfj_object_from_sdf_object)
            .collect(),
        nodes: state.nodes.iter().map(sdfj_node_from_sdf_node).collect(),
        root_nodes: wire_indices(&state.root_node_ids),
        names: SdfjNames {
            shapes3d: sdfj_names(&state.names.shapes3d),
            shapes2d: sdfj_names(&state.names.shapes2d),
            materials: sdfj_names(&state.names.materials),
            patterns: sdfj_names(&state.names.patterns),
            steps: sdfj_names(&state.names.steps),
            parts: sdfj_names(&state.names.parts),
        },
    }
}

/// A names table as a document holds it.
fn sdfj_names<TBrand>(names: &[(String, U32Id<TBrand>)]) -> SdfjMap<usize> {
    SdfjMap::new(
        names
            .iter()
            .map(|(name, named_id)| SdfjMapEntry {
                key: name.clone(),
                value: wire_index(*named_id),
            })
            .collect(),
    )
}

/// The wire index of `id`.
fn wire_index<TBrand>(id: U32Id<TBrand>) -> usize {
    id.to_usize_id().to_usize()
}

/// The wire indices of `ids`.
fn wire_indices<TBrand>(ids: &[U32Id<TBrand>]) -> Vec<usize> {
    ids.iter().map(|&id| wire_index(id)).collect()
}

/// A 3D shape as a document holds it.
fn sdfj_shape3d_from_sdf_shape3d(shape: &SdfShape3d) -> SdfjShape3d {
    match shape {
        SdfShape3d::Bend {
            shape_id,
            along,
            toward,
            radius,
            pivot,
        } => SdfjShape3d::Bend {
            shape: wire_index(*shape_id),
            along: sdfj_axis_from_ty_axis3(*along),
            toward: sdfj_side_from_sdf_side(*toward),
            radius: *radius,
            pivot: pivot.as_ref().map(TyVector3F64::to_array),
        },

        SdfShape3d::Box { min, max, round } => SdfjShape3d::Box {
            min: min.to_array(),
            max: max.to_array(),
            round: *round,
        },

        SdfShape3d::BoxFrame {
            min,
            max,
            thickness,
        } => SdfjShape3d::BoxFrame {
            min: min.to_array(),
            max: max.to_array(),
            thickness: *thickness,
        },

        SdfShape3d::Capsule { a, b, radius } => SdfjShape3d::Capsule {
            a: a.to_array(),
            b: b.to_array(),
            radius: *radius,
        },

        SdfShape3d::Cone {
            a,
            b,
            radius_a,
            radius_b,
        } => SdfjShape3d::Cone {
            a: a.to_array(),
            b: b.to_array(),
            radius_a: *radius_a,
            radius_b: *radius_b,
        },

        SdfShape3d::Cylinder {
            a,
            b,
            radius,
            round,
        } => SdfjShape3d::Cylinder {
            a: a.to_array(),
            b: b.to_array(),
            radius: *radius,
            round: *round,
        },

        SdfShape3d::Displace {
            shape_id,
            amplitude,
            scale,
            octaves,
            seed,
        } => SdfjShape3d::Displace {
            shape: wire_index(*shape_id),
            amplitude: *amplitude,
            scale: *scale,
            octaves: *octaves,
            seed: *seed,
        },

        SdfShape3d::Elongate {
            shape_id,
            lengths,
            center,
        } => SdfjShape3d::Elongate {
            shape: wire_index(*shape_id),
            lengths: lengths.to_array(),
            center: center.as_ref().map(TyVector3F64::to_array),
        },

        SdfShape3d::Ellipsoid { center, radii } => SdfjShape3d::Ellipsoid {
            center: center.to_array(),
            radii: radii.to_array(),
        },

        SdfShape3d::Extrude {
            profile_id,
            axis,
            from,
            to,
        } => SdfjShape3d::Extrude {
            profile: wire_index(*profile_id),
            axis: axis.map(sdfj_axis_from_ty_axis3),
            from: *from,
            to: *to,
        },

        SdfShape3d::HalfSpace { side, at } => SdfjShape3d::HalfSpace {
            side: sdfj_side_from_sdf_side(*side),
            at: *at,
        },

        SdfShape3d::Intersect { shape_ids } => SdfjShape3d::Intersect {
            shapes: wire_indices(shape_ids),
        },

        SdfShape3d::Lathe {
            points,
            axis,
            center,
        } => SdfjShape3d::Lathe {
            points: points.iter().map(TyVector2F64::to_array).collect(),
            axis: axis.map(sdfj_axis_from_ty_axis3),
            center: center.as_ref().map(TyVector3F64::to_array),
        },

        SdfShape3d::Mirror {
            shape_id,
            axes,
            center,
        } => SdfjShape3d::Mirror {
            shape: wire_index(*shape_id),
            axes: sdfj_axes3d_from_sdf_axes3d(*axes),
            center: center.as_ref().map(TyVector3F64::to_array),
        },

        SdfShape3d::Octahedron { center, radius } => SdfjShape3d::Octahedron {
            center: center.to_array(),
            radius: *radius,
        },

        SdfShape3d::Offset { shape_id, distance } => SdfjShape3d::Offset {
            shape: wire_index(*shape_id),
            distance: *distance,
        },

        SdfShape3d::Orient {
            shape_id,
            from,
            to,
            pivot,
        } => SdfjShape3d::Orient {
            shape: wire_index(*shape_id),
            from: from.to_array(),
            to: to.to_array(),
            pivot: pivot.as_ref().map(TyVector3F64::to_array),
        },

        SdfShape3d::Pyramid {
            base_center,
            width,
            height,
        } => SdfjShape3d::Pyramid {
            base_center: base_center.to_array(),
            width: *width,
            height: *height,
        },

        SdfShape3d::Repeat {
            shape_id,
            step,
            count,
        } => SdfjShape3d::Repeat {
            shape: wire_index(*shape_id),
            step: step.to_array(),
            count: count.to_array(),
        },

        SdfShape3d::RepeatPolar {
            shape_id,
            axis,
            count,
            center,
        } => SdfjShape3d::RepeatPolar {
            shape: wire_index(*shape_id),
            axis: sdfj_axis_from_ty_axis3(*axis),
            count: *count,
            center: center.as_ref().map(TyVector3F64::to_array),
        },

        SdfShape3d::Revolve {
            profile_id,
            axis,
            center,
        } => SdfjShape3d::Revolve {
            profile: wire_index(*profile_id),
            axis: axis.map(sdfj_axis_from_ty_axis3),
            center: center.as_ref().map(TyVector3F64::to_array),
        },

        SdfShape3d::Rotate {
            shape_id,
            axis,
            degrees,
            pivot,
        } => SdfjShape3d::Rotate {
            shape: wire_index(*shape_id),
            axis: sdfj_axis_from_ty_axis3(*axis),
            degrees: *degrees,
            pivot: pivot.as_ref().map(TyVector3F64::to_array),
        },

        SdfShape3d::RoundCone {
            a,
            b,
            radius_a,
            radius_b,
        } => SdfjShape3d::RoundCone {
            a: a.to_array(),
            b: b.to_array(),
            radius_a: *radius_a,
            radius_b: *radius_b,
        },

        SdfShape3d::Scale {
            shape_id,
            factor,
            pivot,
        } => SdfjShape3d::Scale {
            shape: wire_index(*shape_id),
            factor: factor.to_array(),
            pivot: pivot.as_ref().map(TyVector3F64::to_array),
        },

        SdfShape3d::Shell {
            shape_id,
            thickness,
        } => SdfjShape3d::Shell {
            shape: wire_index(*shape_id),
            thickness: *thickness,
        },

        SdfShape3d::SmoothIntersect { radius, shape_ids } => SdfjShape3d::SmoothIntersect {
            radius: *radius,
            shapes: wire_indices(shape_ids),
        },

        SdfShape3d::SmoothSubtract {
            radius,
            base_id,
            cutter_ids,
        } => SdfjShape3d::SmoothSubtract {
            radius: *radius,
            base: wire_index(*base_id),
            cutters: wire_indices(cutter_ids),
        },

        SdfShape3d::SmoothUnion { radius, shape_ids } => SdfjShape3d::SmoothUnion {
            radius: *radius,
            shapes: wire_indices(shape_ids),
        },

        SdfShape3d::Sphere { center, radius } => SdfjShape3d::Sphere {
            center: center.to_array(),
            radius: *radius,
        },

        SdfShape3d::Subtract {
            base_id,
            cutter_ids,
        } => SdfjShape3d::Subtract {
            base: wire_index(*base_id),
            cutters: wire_indices(cutter_ids),
        },

        SdfShape3d::Sweep {
            profile_id,
            points,
            scales,
            up,
        } => SdfjShape3d::Sweep {
            profile: wire_index(*profile_id),
            points: points.iter().map(TyVector3F64::to_array).collect(),
            scales: scales.clone(),
            up: up.as_ref().map(TyVector3F64::to_array),
        },

        SdfShape3d::Torus {
            center,
            ring_radius,
            tube_radius,
            axis,
            from,
            to,
        } => SdfjShape3d::Torus {
            center: center.to_array(),
            ring_radius: *ring_radius,
            tube_radius: *tube_radius,
            axis: axis.map(sdfj_axis_from_ty_axis3),
            from: *from,
            to: *to,
        },

        SdfShape3d::Translate { shape_id, offset } => SdfjShape3d::Translate {
            shape: wire_index(*shape_id),
            offset: offset.to_array(),
        },

        SdfShape3d::Twist {
            shape_id,
            axis,
            degrees_per_meter,
            center,
        } => SdfjShape3d::Twist {
            shape: wire_index(*shape_id),
            axis: sdfj_axis_from_ty_axis3(*axis),
            degrees_per_meter: *degrees_per_meter,
            center: center.as_ref().map(TyVector3F64::to_array),
        },

        SdfShape3d::Union { shape_ids } => SdfjShape3d::Union {
            shapes: wire_indices(shape_ids),
        },
    }
}

/// A 2D shape as a document holds it.
fn sdfj_shape2d_from_sdf_shape2d(shape: &SdfShape2d) -> SdfjShape2d {
    match shape {
        SdfShape2d::Arc {
            center,
            radius,
            from_degrees,
            to_degrees,
            width,
            caps,
        } => SdfjShape2d::Arc {
            center: center.to_array(),
            radius: *radius,
            from_degrees: *from_degrees,
            to_degrees: *to_degrees,
            width: *width,
            caps: caps.map(sdfj_caps_from_sdf_caps),
        },

        SdfShape2d::Arch { min, max } => SdfjShape2d::Arch {
            min: min.to_array(),
            max: max.to_array(),
        },

        SdfShape2d::Circle { center, radius } => SdfjShape2d::Circle {
            center: center.to_array(),
            radius: *radius,
        },

        SdfShape2d::Ellipse { center, radii } => SdfjShape2d::Ellipse {
            center: center.to_array(),
            radii: radii.to_array(),
        },

        SdfShape2d::Intersect { shape_ids } => SdfjShape2d::Intersect {
            shapes: wire_indices(shape_ids),
        },

        SdfShape2d::Mirror {
            shape_id,
            axes,
            center,
        } => SdfjShape2d::Mirror {
            shape: wire_index(*shape_id),
            axes: sdfj_axes2d_from_sdf_axes2d(*axes),
            center: center.as_ref().map(TyVector2F64::to_array),
        },

        SdfShape2d::Ngon {
            center,
            sides,
            radius,
        } => SdfjShape2d::Ngon {
            center: center.to_array(),
            sides: *sides,
            radius: *radius,
        },

        SdfShape2d::Offset { shape_id, distance } => SdfjShape2d::Offset {
            shape: wire_index(*shape_id),
            distance: *distance,
        },

        SdfShape2d::Polygon { points } => SdfjShape2d::Polygon {
            points: points.iter().map(TyVector2F64::to_array).collect(),
        },

        SdfShape2d::Polyline { points, width } => SdfjShape2d::Polyline {
            points: points.iter().map(TyVector2F64::to_array).collect(),
            width: *width,
        },

        SdfShape2d::Rect {
            min,
            max,
            chamfer,
            round,
        } => SdfjShape2d::Rect {
            min: min.to_array(),
            max: max.to_array(),
            chamfer: *chamfer,
            round: *round,
        },

        SdfShape2d::Repeat {
            shape_id,
            step,
            count,
        } => SdfjShape2d::Repeat {
            shape: wire_index(*shape_id),
            step: step.to_array(),
            count: count.to_array(),
        },

        SdfShape2d::RepeatPolar {
            shape_id,
            count,
            center,
        } => SdfjShape2d::RepeatPolar {
            shape: wire_index(*shape_id),
            count: *count,
            center: center.as_ref().map(TyVector2F64::to_array),
        },

        SdfShape2d::Rotate {
            shape_id,
            degrees,
            pivot,
        } => SdfjShape2d::Rotate {
            shape: wire_index(*shape_id),
            degrees: *degrees,
            pivot: pivot.as_ref().map(TyVector2F64::to_array),
        },

        SdfShape2d::Scale {
            shape_id,
            factor,
            pivot,
        } => SdfjShape2d::Scale {
            shape: wire_index(*shape_id),
            factor: factor.to_array(),
            pivot: pivot.as_ref().map(TyVector2F64::to_array),
        },

        SdfShape2d::Sector {
            center,
            radius,
            from_degrees,
            to_degrees,
        } => SdfjShape2d::Sector {
            center: center.to_array(),
            radius: *radius,
            from_degrees: *from_degrees,
            to_degrees: *to_degrees,
        },

        SdfShape2d::Shell {
            shape_id,
            thickness,
        } => SdfjShape2d::Shell {
            shape: wire_index(*shape_id),
            thickness: *thickness,
        },

        SdfShape2d::SmoothIntersect { radius, shape_ids } => SdfjShape2d::SmoothIntersect {
            radius: *radius,
            shapes: wire_indices(shape_ids),
        },

        SdfShape2d::SmoothSubtract {
            radius,
            base_id,
            cutter_ids,
        } => SdfjShape2d::SmoothSubtract {
            radius: *radius,
            base: wire_index(*base_id),
            cutters: wire_indices(cutter_ids),
        },

        SdfShape2d::SmoothUnion { radius, shape_ids } => SdfjShape2d::SmoothUnion {
            radius: *radius,
            shapes: wire_indices(shape_ids),
        },

        SdfShape2d::Spline { points, width } => SdfjShape2d::Spline {
            points: points.iter().map(TyVector2F64::to_array).collect(),
            width: *width,
        },

        SdfShape2d::Star {
            center,
            points,
            outer_radius,
            inner_radius,
        } => SdfjShape2d::Star {
            center: center.to_array(),
            points: *points,
            outer_radius: *outer_radius,
            inner_radius: *inner_radius,
        },

        SdfShape2d::Subtract {
            base_id,
            cutter_ids,
        } => SdfjShape2d::Subtract {
            base: wire_index(*base_id),
            cutters: wire_indices(cutter_ids),
        },

        SdfShape2d::Translate { shape_id, offset } => SdfjShape2d::Translate {
            shape: wire_index(*shape_id),
            offset: offset.to_array(),
        },

        SdfShape2d::Union { shape_ids } => SdfjShape2d::Union {
            shapes: wire_indices(shape_ids),
        },

        SdfShape2d::Vesica { a, b, width } => SdfjShape2d::Vesica {
            a: a.to_array(),
            b: b.to_array(),
            width: *width,
        },
    }
}

/// A material as a document holds it.
fn sdfj_material_from_sdf_material(material: &SdfMaterial) -> SdfjMaterial {
    match material {
        SdfMaterial::Material { properties } => SdfjMaterial::Material {
            properties: SdfjMap::new(
                properties
                    .iter()
                    .map(|property| SdfjMapEntry {
                        key: property.name.clone(),
                        value: sdfj_property_value_from_sdf_property_value(&property.value),
                    })
                    .collect(),
            ),
        },

        SdfMaterial::Shade { shades_id, index } => SdfjMaterial::Shade {
            shades: wire_index(*shades_id),
            index: usize::try_from(*index).expect("a u32 fits a usize"),
        },
    }
}

/// A property value as a document holds it.
fn sdfj_property_value_from_sdf_property_value(value: &SdfPropertyValue) -> SdfjPropertyValue {
    match value {
        SdfPropertyValue::Bool(bool) => SdfjPropertyValue::Bool(*bool),

        SdfPropertyValue::Int(number) => {
            SdfjPropertyValue::Tagged(SdfjTaggedValue::Int(SdfjIntValue::Number(*number)))
        }

        SdfPropertyValue::IntArray(numbers) => SdfjPropertyValue::Tagged(SdfjTaggedValue::Int(
            SdfjIntValue::NumberArray(numbers.clone()),
        )),

        SdfPropertyValue::Json(value) => {
            SdfjPropertyValue::Tagged(SdfjTaggedValue::Json(sdfj_value_from_sdf_value(value)))
        }

        SdfPropertyValue::Number(number) => SdfjPropertyValue::Number(*number),

        SdfPropertyValue::NumberArray(numbers) => SdfjPropertyValue::NumberArray(numbers.clone()),

        SdfPropertyValue::Text(text) => SdfjPropertyValue::Text(text.clone()),
    }
}

/// A `json` value as a document holds it.
fn sdfj_value_from_sdf_value(value: &SdfValue) -> SdfjValue {
    match value {
        SdfValue::Array(values) => {
            SdfjValue::Array(values.iter().map(sdfj_value_from_sdf_value).collect())
        }

        SdfValue::Bool(bool) => SdfjValue::Bool(*bool),

        SdfValue::Null => SdfjValue::Null,

        SdfValue::Number(number) => SdfjValue::Number(*number),

        SdfValue::Object(map) => SdfjValue::Object(SdfjMap::new(
            map.entries()
                .iter()
                .map(|entry| SdfjMapEntry {
                    key: entry.key.clone(),
                    value: sdfj_value_from_sdf_value(&entry.value),
                })
                .collect(),
        )),

        SdfValue::Text(text) => SdfjValue::Text(text.clone()),
    }
}

/// A shades entry as a document holds it.
fn sdfj_shades_from_sdf_shades(shades: &SdfShades) -> SdfjShades {
    SdfjShades {
        base: wire_index(shades.base_id),
        count: shades.count,
        spread: shades.spread,
    }
}

/// A pattern as a document holds it.
fn sdfj_pattern_from_sdf_pattern(pattern: &SdfPattern) -> SdfjPattern {
    match pattern {
        SdfPattern::Bands {
            material_ids,
            axis,
            period,
            warp,
            seed,
        } => SdfjPattern::Bands {
            materials: wire_indices(material_ids),
            axis: sdfj_axis_from_ty_axis3(*axis),
            period: *period,
            warp: *warp,
            seed: *seed,
        },

        SdfPattern::Cells {
            material_ids,
            size,
            seed,
            border_id,
        } => SdfjPattern::Cells {
            materials: wire_indices(material_ids),
            size: *size,
            seed: *seed,
            border: border_id.map(wire_index),
        },

        SdfPattern::Checker { material_ids, size } => SdfjPattern::Checker {
            materials: wire_indices(material_ids),
            size: *size,
        },

        SdfPattern::Gradient {
            material_ids,
            axis,
            from,
            to,
            warp,
            seed,
        } => SdfjPattern::Gradient {
            materials: wire_indices(material_ids),
            axis: sdfj_axis_from_ty_axis3(*axis),
            from: *from,
            to: *to,
            warp: *warp,
            seed: *seed,
        },

        SdfPattern::Grain {
            material_ids,
            axis,
            period,
            warp,
            seed,
        } => SdfjPattern::Grain {
            materials: wire_indices(material_ids),
            axis: sdfj_axis_from_ty_axis3(*axis),
            period: *period,
            warp: *warp,
            seed: *seed,
        },

        SdfPattern::Noise {
            material_ids,
            scale,
            octaves,
            seed,
        } => SdfjPattern::Noise {
            materials: wire_indices(material_ids),
            scale: *scale,
            octaves: *octaves,
            seed: *seed,
        },

        SdfPattern::Speckle {
            base_id,
            accent_ids,
            density,
            seed,
        } => SdfjPattern::Speckle {
            base: wire_index(*base_id),
            accents: wire_indices(accent_ids),
            density: *density,
            seed: *seed,
        },
    }
}

/// A step as a document holds it.
fn sdfj_step_from_sdf_step(step: &SdfStep) -> SdfjStep {
    match step {
        SdfStep::Add {
            name,
            shape_id,
            material,
        } => SdfjStep::Add {
            name: name.clone(),
            shape: wire_index(*shape_id),
            material: sdfj_material_index(*material),
            pattern: sdfj_pattern_index(*material),
        },

        SdfStep::Carve { name, shape_id } => SdfjStep::Carve {
            name: name.clone(),
            shape: wire_index(*shape_id),
        },

        SdfStep::Coat {
            name,
            material,
            sides,
            depth,
            within_id,
            onto_ids,
        } => SdfjStep::Coat {
            name: name.clone(),
            material: sdfj_material_index(*material),
            pattern: sdfj_pattern_index(*material),
            sides: sides
                .as_ref()
                .map(|sides| sides.iter().copied().map(sdfj_side_from_sdf_side).collect()),
            depth: *depth,
            within: within_id.map(wire_index),
            onto: onto_ids.as_deref().map(wire_indices),
        },

        SdfStep::Paint {
            name,
            shape_id,
            material,
            onto_ids,
        } => SdfjStep::Paint {
            name: name.clone(),
            shape: wire_index(*shape_id),
            material: sdfj_material_index(*material),
            pattern: sdfj_pattern_index(*material),
            onto: onto_ids.as_deref().map(wire_indices),
        },

        SdfStep::Set {
            name,
            points,
            material,
        } => SdfjStep::Set {
            name: name.clone(),
            points: points.iter().map(TyVector3F64::to_array).collect(),
            material: sdfj_material_index(*material),
            pattern: sdfj_pattern_index(*material),
        },
    }
}

/// The `material` key of a step that writes `material`.
fn sdfj_material_index(material: SdfStepMaterial) -> Option<usize> {
    match material {
        SdfStepMaterial::Material(material_id) => Some(wire_index(material_id)),
        SdfStepMaterial::Pattern(_) => None,
    }
}

/// The `pattern` key of a step that writes `material`.
fn sdfj_pattern_index(material: SdfStepMaterial) -> Option<usize> {
    match material {
        SdfStepMaterial::Material(_) => None,
        SdfStepMaterial::Pattern(pattern_id) => Some(wire_index(pattern_id)),
    }
}

/// An object as a document holds it.
fn sdfj_object_from_sdf_object(object: &SdfObject) -> SdfjObject {
    SdfjObject {
        name: object.name.clone(),
        steps: wire_indices(&object.step_ids),
    }
}

/// A node as a document holds it.
fn sdfj_node_from_sdf_node(node: &SdfNode) -> SdfjNode {
    SdfjNode {
        name: node.name.clone(),
        pivot: node.pivot.as_ref().map(TyVector3F64::to_array),
        offset: node.offset.as_ref().map(TyVector3F64::to_array),
        rotation: node
            .rotation
            .iter()
            .map(|axis_rotation| SdfjAxisRotation {
                axis: sdfj_axis_from_ty_axis3(axis_rotation.axis),
                degrees: axis_rotation.degrees,
            })
            .collect(),
        child_objects: wire_indices(&node.child_object_ids),
        child_nodes: wire_indices(&node.child_node_ids),
    }
}

/// An axis as a document holds it.
fn sdfj_axis_from_ty_axis3(axis: TyAxis3) -> SdfjAxis {
    match axis {
        TyAxis3::X => SdfjAxis::X,
        TyAxis3::Y => SdfjAxis::Y,
        TyAxis3::Z => SdfjAxis::Z,
    }
}

/// 2D mirror axes as a document holds them.
fn sdfj_axes2d_from_sdf_axes2d(axes: SdfAxes2d) -> SdfjAxes2d {
    match axes {
        SdfAxes2d::U => SdfjAxes2d::U,
        SdfAxes2d::Uv => SdfjAxes2d::Uv,
        SdfAxes2d::V => SdfjAxes2d::V,
    }
}

/// 3D mirror axes as a document holds them.
fn sdfj_axes3d_from_sdf_axes3d(axes: SdfAxes3d) -> SdfjAxes3d {
    match axes {
        SdfAxes3d::X => SdfjAxes3d::X,
        SdfAxes3d::Xy => SdfjAxes3d::Xy,
        SdfAxes3d::Xyz => SdfjAxes3d::Xyz,
        SdfAxes3d::Xz => SdfjAxes3d::Xz,
        SdfAxes3d::Y => SdfjAxes3d::Y,
        SdfAxes3d::Yz => SdfjAxes3d::Yz,
        SdfAxes3d::Z => SdfjAxes3d::Z,
    }
}

/// Arc caps as a document holds them.
fn sdfj_caps_from_sdf_caps(caps: SdfCaps) -> SdfjCaps {
    match caps {
        SdfCaps::Flat => SdfjCaps::Flat,
        SdfCaps::Round => SdfjCaps::Round,
    }
}

/// A side as a document holds it.
fn sdfj_side_from_sdf_side(side: SdfSide) -> SdfjSide {
    match side {
        SdfSide::NegativeX => SdfjSide::NegativeX,
        SdfSide::NegativeY => SdfjSide::NegativeY,
        SdfSide::NegativeZ => SdfjSide::NegativeZ,
        SdfSide::PositiveX => SdfjSide::PositiveX,
        SdfSide::PositiveY => SdfjSide::PositiveY,
        SdfSide::PositiveZ => SdfjSide::PositiveZ,
    }
}
