use crate::{Error, Result};
use branded_id::{IdVec, U32Id};
use sdfcore::{
    SdfAxes2d, SdfAxes3d, SdfAxisRotation, SdfBorder, SdfCaps, SdfMain, SdfMap, SdfMapEntry,
    SdfMaterial, SdfNames, SdfNode, SdfObject, SdfPattern, SdfProperty, SdfPropertyValue,
    SdfShades, SdfShape2d, SdfShape3d, SdfSide, SdfState, SdfStep, SdfStepMaterial, SdfValue,
};
use sdfj::{
    SDFJ_VERSION, SdfjAxes2d, SdfjAxes3d, SdfjAxis, SdfjBorder, SdfjCaps, SdfjFile, SdfjIntValue,
    SdfjMap, SdfjMapEntry, SdfjMaterial, SdfjNode, SdfjObject, SdfjPattern, SdfjPropertyValue,
    SdfjShades, SdfjShape2d, SdfjShape3d, SdfjSide, SdfjStep, SdfjTaggedValue, SdfjValue,
};
use std::result::Result as StdResult;
use ty_math::{TyAxis3, TyVector2F64, TyVector3F64};

/// Loads an [`SdfjFile`] into an [`SdfMain`]. An error about one entry starts
/// with the entry's place.
pub fn from_sdfj_file(file: &SdfjFile) -> Result<SdfMain> {
    if file.version != SDFJ_VERSION {
        return Err(Error::invalid(format!(
            "version {} is not {SDFJ_VERSION}",
            file.version
        )));
    }

    let root_node_ids = file
        .root_nodes
        .iter()
        .map(|&index| wire_id(index))
        .collect::<StdResult<_, _>>()
        .map_err(|message| Error::invalid(format!("rootNodes {message}")))?;

    let state = SdfState {
        shapes3d: entries("shapes3d", &file.shapes3d, sdf_shape3d_from_sdfj_shape3d)?,
        shapes2d: entries("shapes2d", &file.shapes2d, sdf_shape2d_from_sdfj_shape2d)?,
        materials: entries(
            "materials",
            &file.materials,
            sdf_material_from_sdfj_material,
        )?,
        shades: entries("shades", &file.shades, sdf_shades_from_sdfj_shades)?,
        patterns: entries("patterns", &file.patterns, sdf_pattern_from_sdfj_pattern)?,
        steps: entries("steps", &file.steps, sdf_step_from_sdfj_step)?,
        objects: entries("objects", &file.objects, sdf_object_from_sdfj_object)?,
        nodes: entries("nodes", &file.nodes, sdf_node_from_sdfj_node)?,
        root_node_ids,
        names: SdfNames {
            shapes3d: names("shapes3d", &file.names.shapes3d)?,
            shapes2d: names("shapes2d", &file.names.shapes2d)?,
            materials: names("materials", &file.names.materials)?,
            patterns: names("patterns", &file.names.patterns)?,
            steps: names("steps", &file.names.steps)?,
            parts: names("parts", &file.names.parts)?,
        },
    };

    Ok(SdfMain::new(state)?)
}

/// Converts each entry of the table `table`. A failure starts with the
/// entry's place.
fn entries<TSdfj, TBrand, TSdf>(
    table: &str,
    sdfj_entries: &[TSdfj],
    convert: impl Fn(&TSdfj) -> StdResult<TSdf, String>,
) -> Result<IdVec<TBrand, TSdf>> {
    sdfj_entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            convert(entry).map_err(|message| Error::invalid(format!("{table}[{index}] {message}")))
        })
        .collect()
}

/// Converts each name of the names table `table`. A failure starts with the
/// name's place.
fn names<TBrand>(table: &str, map: &SdfjMap<usize>) -> Result<Vec<(String, U32Id<TBrand>)>> {
    map.entries()
        .iter()
        .map(|SdfjMapEntry { key, value }| {
            let named_id = wire_id(*value)
                .map_err(|message| Error::invalid(format!("names.{table} `{key}` {message}")))?;

            Ok((key.clone(), named_id))
        })
        .collect()
}

/// The id that wire index `index` references. Errors when `index` is past the
/// id space. A cast would wrap such an index onto a real entry.
fn wire_id<TBrand>(index: usize) -> StdResult<U32Id<TBrand>, String> {
    let Ok(id) = u32::try_from(index) else {
        return Err(format!("references index {index}, past the id space"));
    };

    Ok(U32Id::from_u32(id))
}

/// The ids that wire indices `indices` reference.
fn wire_ids<TBrand>(indices: &[usize]) -> StdResult<Vec<U32Id<TBrand>>, String> {
    indices.iter().map(|&index| wire_id(index)).collect()
}

/// A 3D shape as sdfcore holds it.
fn sdf_shape3d_from_sdfj_shape3d(shape: &SdfjShape3d) -> StdResult<SdfShape3d, String> {
    Ok(match shape {
        SdfjShape3d::Bend {
            shape,
            along,
            toward,
            radius,
            pivot,
        } => SdfShape3d::Bend {
            shape_id: wire_id(*shape)?,
            along: ty_axis3_from_sdfj_axis(*along),
            toward: sdf_side_from_sdfj_side(*toward),
            radius: *radius,
            pivot: pivot.map(TyVector3F64::from_array),
        },

        SdfjShape3d::Box { min, max, round } => SdfShape3d::Box {
            min: TyVector3F64::from_array(*min),
            max: TyVector3F64::from_array(*max),
            round: *round,
        },

        SdfjShape3d::BoxFrame {
            min,
            max,
            thickness,
        } => SdfShape3d::BoxFrame {
            min: TyVector3F64::from_array(*min),
            max: TyVector3F64::from_array(*max),
            thickness: *thickness,
        },

        SdfjShape3d::Capsule { a, b, radius } => SdfShape3d::Capsule {
            a: TyVector3F64::from_array(*a),
            b: TyVector3F64::from_array(*b),
            radius: *radius,
        },

        SdfjShape3d::Cone {
            a,
            b,
            radius_a,
            radius_b,
        } => SdfShape3d::Cone {
            a: TyVector3F64::from_array(*a),
            b: TyVector3F64::from_array(*b),
            radius_a: *radius_a,
            radius_b: *radius_b,
        },

        SdfjShape3d::Cylinder {
            a,
            b,
            radius,
            round,
        } => SdfShape3d::Cylinder {
            a: TyVector3F64::from_array(*a),
            b: TyVector3F64::from_array(*b),
            radius: *radius,
            round: *round,
        },

        SdfjShape3d::Displace {
            shape,
            amplitude,
            scale,
            octaves,
            seed,
        } => SdfShape3d::Displace {
            shape_id: wire_id(*shape)?,
            amplitude: *amplitude,
            scale: *scale,
            octaves: *octaves,
            seed: *seed,
        },

        SdfjShape3d::Elongate {
            shape,
            lengths,
            center,
        } => SdfShape3d::Elongate {
            shape_id: wire_id(*shape)?,
            lengths: TyVector3F64::from_array(*lengths),
            center: center.map(TyVector3F64::from_array),
        },

        SdfjShape3d::Ellipsoid { center, radii } => SdfShape3d::Ellipsoid {
            center: TyVector3F64::from_array(*center),
            radii: TyVector3F64::from_array(*radii),
        },

        SdfjShape3d::Extrude {
            profile,
            axis,
            from,
            to,
        } => SdfShape3d::Extrude {
            profile_id: wire_id(*profile)?,
            axis: axis.map(ty_axis3_from_sdfj_axis),
            from: *from,
            to: *to,
        },

        SdfjShape3d::HalfSpace { side, at } => SdfShape3d::HalfSpace {
            side: sdf_side_from_sdfj_side(*side),
            at: *at,
        },

        SdfjShape3d::Intersect { shapes } => SdfShape3d::Intersect {
            shape_ids: wire_ids(shapes)?,
        },

        SdfjShape3d::Lathe {
            points,
            axis,
            center,
        } => SdfShape3d::Lathe {
            points: points
                .iter()
                .copied()
                .map(TyVector2F64::from_array)
                .collect(),
            axis: axis.map(ty_axis3_from_sdfj_axis),
            center: center.map(TyVector3F64::from_array),
        },

        SdfjShape3d::Mirror {
            shape,
            axes,
            center,
        } => SdfShape3d::Mirror {
            shape_id: wire_id(*shape)?,
            axes: sdf_axes3d_from_sdfj_axes3d(*axes),
            center: center.map(TyVector3F64::from_array),
        },

        SdfjShape3d::Octahedron { center, radius } => SdfShape3d::Octahedron {
            center: TyVector3F64::from_array(*center),
            radius: *radius,
        },

        SdfjShape3d::Offset { shape, distance } => SdfShape3d::Offset {
            shape_id: wire_id(*shape)?,
            distance: *distance,
        },

        SdfjShape3d::Orient {
            shape,
            from,
            to,
            pivot,
        } => SdfShape3d::Orient {
            shape_id: wire_id(*shape)?,
            from: TyVector3F64::from_array(*from),
            to: TyVector3F64::from_array(*to),
            pivot: pivot.map(TyVector3F64::from_array),
        },

        SdfjShape3d::Pyramid {
            base_center,
            width,
            height,
        } => SdfShape3d::Pyramid {
            base_center: TyVector3F64::from_array(*base_center),
            width: *width,
            height: *height,
        },

        SdfjShape3d::Repeat { shape, step, count } => SdfShape3d::Repeat {
            shape_id: wire_id(*shape)?,
            step: TyVector3F64::from_array(*step),
            count: TyVector3F64::from_array(*count),
        },

        SdfjShape3d::RepeatPolar {
            shape,
            axis,
            count,
            center,
        } => SdfShape3d::RepeatPolar {
            shape_id: wire_id(*shape)?,
            axis: ty_axis3_from_sdfj_axis(*axis),
            count: *count,
            center: center.map(TyVector3F64::from_array),
        },

        SdfjShape3d::Revolve {
            profile,
            axis,
            center,
        } => SdfShape3d::Revolve {
            profile_id: wire_id(*profile)?,
            axis: axis.map(ty_axis3_from_sdfj_axis),
            center: center.map(TyVector3F64::from_array),
        },

        SdfjShape3d::Rotate {
            shape,
            axis,
            degrees,
            pivot,
        } => SdfShape3d::Rotate {
            shape_id: wire_id(*shape)?,
            axis: ty_axis3_from_sdfj_axis(*axis),
            degrees: *degrees,
            pivot: pivot.map(TyVector3F64::from_array),
        },

        SdfjShape3d::RoundCone {
            a,
            b,
            radius_a,
            radius_b,
        } => SdfShape3d::RoundCone {
            a: TyVector3F64::from_array(*a),
            b: TyVector3F64::from_array(*b),
            radius_a: *radius_a,
            radius_b: *radius_b,
        },

        SdfjShape3d::Scale {
            shape,
            factor,
            pivot,
        } => SdfShape3d::Scale {
            shape_id: wire_id(*shape)?,
            factor: TyVector3F64::from_array(*factor),
            pivot: pivot.map(TyVector3F64::from_array),
        },

        SdfjShape3d::Shell { shape, thickness } => SdfShape3d::Shell {
            shape_id: wire_id(*shape)?,
            thickness: *thickness,
        },

        SdfjShape3d::SmoothIntersect { radius, shapes } => SdfShape3d::SmoothIntersect {
            radius: *radius,
            shape_ids: wire_ids(shapes)?,
        },

        SdfjShape3d::SmoothSubtract {
            radius,
            base,
            cutters,
        } => SdfShape3d::SmoothSubtract {
            radius: *radius,
            base_id: wire_id(*base)?,
            cutter_ids: wire_ids(cutters)?,
        },

        SdfjShape3d::SmoothUnion { radius, shapes } => SdfShape3d::SmoothUnion {
            radius: *radius,
            shape_ids: wire_ids(shapes)?,
        },

        SdfjShape3d::Sphere { center, radius } => SdfShape3d::Sphere {
            center: TyVector3F64::from_array(*center),
            radius: *radius,
        },

        SdfjShape3d::Subtract { base, cutters } => SdfShape3d::Subtract {
            base_id: wire_id(*base)?,
            cutter_ids: wire_ids(cutters)?,
        },

        SdfjShape3d::Sweep {
            profile,
            points,
            scales,
            up,
        } => SdfShape3d::Sweep {
            profile_id: wire_id(*profile)?,
            points: points
                .iter()
                .copied()
                .map(TyVector3F64::from_array)
                .collect(),
            scales: scales.clone(),
            up: up.map(TyVector3F64::from_array),
        },

        SdfjShape3d::Torus {
            center,
            ring_radius,
            tube_radius,
            axis,
            from,
            to,
        } => SdfShape3d::Torus {
            center: TyVector3F64::from_array(*center),
            ring_radius: *ring_radius,
            tube_radius: *tube_radius,
            axis: axis.map(ty_axis3_from_sdfj_axis),
            from: *from,
            to: *to,
        },

        SdfjShape3d::Translate { shape, offset } => SdfShape3d::Translate {
            shape_id: wire_id(*shape)?,
            offset: TyVector3F64::from_array(*offset),
        },

        SdfjShape3d::Twist {
            shape,
            axis,
            degrees_per_meter,
            center,
        } => SdfShape3d::Twist {
            shape_id: wire_id(*shape)?,
            axis: ty_axis3_from_sdfj_axis(*axis),
            degrees_per_meter: *degrees_per_meter,
            center: center.map(TyVector3F64::from_array),
        },

        SdfjShape3d::Union { shapes } => SdfShape3d::Union {
            shape_ids: wire_ids(shapes)?,
        },
    })
}

/// A 2D shape as sdfcore holds it.
fn sdf_shape2d_from_sdfj_shape2d(shape: &SdfjShape2d) -> StdResult<SdfShape2d, String> {
    Ok(match shape {
        SdfjShape2d::Arc {
            center,
            radius,
            from_degrees,
            to_degrees,
            width,
            caps,
        } => SdfShape2d::Arc {
            center: TyVector2F64::from_array(*center),
            radius: *radius,
            from_degrees: *from_degrees,
            to_degrees: *to_degrees,
            width: *width,
            caps: caps.map(sdf_caps_from_sdfj_caps),
        },

        SdfjShape2d::Arch { min, max } => SdfShape2d::Arch {
            min: TyVector2F64::from_array(*min),
            max: TyVector2F64::from_array(*max),
        },

        SdfjShape2d::Circle { center, radius } => SdfShape2d::Circle {
            center: TyVector2F64::from_array(*center),
            radius: *radius,
        },

        SdfjShape2d::Ellipse { center, radii } => SdfShape2d::Ellipse {
            center: TyVector2F64::from_array(*center),
            radii: TyVector2F64::from_array(*radii),
        },

        SdfjShape2d::Intersect { shapes } => SdfShape2d::Intersect {
            shape_ids: wire_ids(shapes)?,
        },

        SdfjShape2d::Mirror {
            shape,
            axes,
            center,
        } => SdfShape2d::Mirror {
            shape_id: wire_id(*shape)?,
            axes: sdf_axes2d_from_sdfj_axes2d(*axes),
            center: center.map(TyVector2F64::from_array),
        },

        SdfjShape2d::Ngon {
            center,
            sides,
            radius,
        } => SdfShape2d::Ngon {
            center: TyVector2F64::from_array(*center),
            sides: *sides,
            radius: *radius,
        },

        SdfjShape2d::Offset { shape, distance } => SdfShape2d::Offset {
            shape_id: wire_id(*shape)?,
            distance: *distance,
        },

        SdfjShape2d::Polygon { points } => SdfShape2d::Polygon {
            points: points
                .iter()
                .copied()
                .map(TyVector2F64::from_array)
                .collect(),
        },

        SdfjShape2d::Polyline { points, width } => SdfShape2d::Polyline {
            points: points
                .iter()
                .copied()
                .map(TyVector2F64::from_array)
                .collect(),
            width: *width,
        },

        SdfjShape2d::Rect {
            min,
            max,
            chamfer,
            round,
        } => SdfShape2d::Rect {
            min: TyVector2F64::from_array(*min),
            max: TyVector2F64::from_array(*max),
            chamfer: *chamfer,
            round: *round,
        },

        SdfjShape2d::Repeat { shape, step, count } => SdfShape2d::Repeat {
            shape_id: wire_id(*shape)?,
            step: TyVector2F64::from_array(*step),
            count: TyVector2F64::from_array(*count),
        },

        SdfjShape2d::RepeatPolar {
            shape,
            count,
            center,
        } => SdfShape2d::RepeatPolar {
            shape_id: wire_id(*shape)?,
            count: *count,
            center: center.map(TyVector2F64::from_array),
        },

        SdfjShape2d::Rotate {
            shape,
            degrees,
            pivot,
        } => SdfShape2d::Rotate {
            shape_id: wire_id(*shape)?,
            degrees: *degrees,
            pivot: pivot.map(TyVector2F64::from_array),
        },

        SdfjShape2d::Scale {
            shape,
            factor,
            pivot,
        } => SdfShape2d::Scale {
            shape_id: wire_id(*shape)?,
            factor: TyVector2F64::from_array(*factor),
            pivot: pivot.map(TyVector2F64::from_array),
        },

        SdfjShape2d::Sector {
            center,
            radius,
            from_degrees,
            to_degrees,
        } => SdfShape2d::Sector {
            center: TyVector2F64::from_array(*center),
            radius: *radius,
            from_degrees: *from_degrees,
            to_degrees: *to_degrees,
        },

        SdfjShape2d::Shell { shape, thickness } => SdfShape2d::Shell {
            shape_id: wire_id(*shape)?,
            thickness: *thickness,
        },

        SdfjShape2d::SmoothIntersect { radius, shapes } => SdfShape2d::SmoothIntersect {
            radius: *radius,
            shape_ids: wire_ids(shapes)?,
        },

        SdfjShape2d::SmoothSubtract {
            radius,
            base,
            cutters,
        } => SdfShape2d::SmoothSubtract {
            radius: *radius,
            base_id: wire_id(*base)?,
            cutter_ids: wire_ids(cutters)?,
        },

        SdfjShape2d::SmoothUnion { radius, shapes } => SdfShape2d::SmoothUnion {
            radius: *radius,
            shape_ids: wire_ids(shapes)?,
        },

        SdfjShape2d::Spline { points, width } => SdfShape2d::Spline {
            points: points
                .iter()
                .copied()
                .map(TyVector2F64::from_array)
                .collect(),
            width: *width,
        },

        SdfjShape2d::Star {
            center,
            points,
            outer_radius,
            inner_radius,
        } => SdfShape2d::Star {
            center: TyVector2F64::from_array(*center),
            points: *points,
            outer_radius: *outer_radius,
            inner_radius: *inner_radius,
        },

        SdfjShape2d::Subtract { base, cutters } => SdfShape2d::Subtract {
            base_id: wire_id(*base)?,
            cutter_ids: wire_ids(cutters)?,
        },

        SdfjShape2d::Translate { shape, offset } => SdfShape2d::Translate {
            shape_id: wire_id(*shape)?,
            offset: TyVector2F64::from_array(*offset),
        },

        SdfjShape2d::Union { shapes } => SdfShape2d::Union {
            shape_ids: wire_ids(shapes)?,
        },

        SdfjShape2d::Vesica { a, b, width } => SdfShape2d::Vesica {
            a: TyVector2F64::from_array(*a),
            b: TyVector2F64::from_array(*b),
            width: *width,
        },
    })
}

/// A material as sdfcore holds it. Errors on a shade index past the `u32`
/// range.
fn sdf_material_from_sdfj_material(material: &SdfjMaterial) -> StdResult<SdfMaterial, String> {
    Ok(match material {
        SdfjMaterial::Material { properties } => SdfMaterial::Material {
            properties: properties
                .entries()
                .iter()
                .map(|entry| SdfProperty {
                    name: entry.key.clone(),
                    value: sdf_property_value_from_sdfj_property_value(&entry.value),
                })
                .collect(),
        },

        SdfjMaterial::Shade { shades, index } => SdfMaterial::Shade {
            shades_id: wire_id(*shades)?,
            index: u32::try_from(*index)
                .map_err(|_| format!("takes shade {index}, past the u32 range"))?,
        },
    })
}

/// A property value as sdfcore holds it.
fn sdf_property_value_from_sdfj_property_value(value: &SdfjPropertyValue) -> SdfPropertyValue {
    match value {
        SdfjPropertyValue::Bool(bool) => SdfPropertyValue::Bool(*bool),

        SdfjPropertyValue::Number(number) => SdfPropertyValue::Number(*number),

        SdfjPropertyValue::NumberArray(numbers) => SdfPropertyValue::NumberArray(numbers.clone()),

        SdfjPropertyValue::Tagged(SdfjTaggedValue::Int(SdfjIntValue::Number(number))) => {
            SdfPropertyValue::Int(*number)
        }

        SdfjPropertyValue::Tagged(SdfjTaggedValue::Int(SdfjIntValue::NumberArray(numbers))) => {
            SdfPropertyValue::IntArray(numbers.clone())
        }

        SdfjPropertyValue::Tagged(SdfjTaggedValue::Json(value)) => {
            SdfPropertyValue::Json(sdf_value_from_sdfj_value(value))
        }

        SdfjPropertyValue::Text(text) => SdfPropertyValue::Text(text.clone()),
    }
}

/// A `json` value as sdfcore holds it.
fn sdf_value_from_sdfj_value(value: &SdfjValue) -> SdfValue {
    match value {
        SdfjValue::Array(values) => {
            SdfValue::Array(values.iter().map(sdf_value_from_sdfj_value).collect())
        }

        SdfjValue::Bool(bool) => SdfValue::Bool(*bool),

        SdfjValue::Null => SdfValue::Null,

        SdfjValue::Number(number) => SdfValue::Number(*number),

        SdfjValue::Object(map) => SdfValue::Object(SdfMap::new(
            map.entries()
                .iter()
                .map(|entry| SdfMapEntry {
                    key: entry.key.clone(),
                    value: sdf_value_from_sdfj_value(&entry.value),
                })
                .collect(),
        )),

        SdfjValue::Text(text) => SdfValue::Text(text.clone()),
    }
}

/// A shades entry as sdfcore holds it.
fn sdf_shades_from_sdfj_shades(shades: &SdfjShades) -> StdResult<SdfShades, String> {
    Ok(SdfShades {
        base_id: wire_id(shades.base)?,
        count: shades.count,
        spread: shades.spread,
    })
}

/// A pattern as sdfcore holds it.
fn sdf_pattern_from_sdfj_pattern(pattern: &SdfjPattern) -> StdResult<SdfPattern, String> {
    Ok(match pattern {
        SdfjPattern::Bands {
            materials,
            axis,
            period,
            warp,
            seed,
        } => SdfPattern::Bands {
            material_ids: wire_ids(materials)?,
            axis: ty_axis3_from_sdfj_axis(*axis),
            period: *period,
            warp: *warp,
            seed: *seed,
        },

        SdfjPattern::Cells {
            materials,
            size,
            seed,
            border,
        } => SdfPattern::Cells {
            material_ids: wire_ids(materials)?,
            size: *size,
            seed: *seed,
            border: match border {
                Some(SdfjBorder { material, width }) => Some(SdfBorder {
                    material_id: wire_id(*material)?,
                    width: *width,
                }),

                None => None,
            },
        },

        SdfjPattern::Checker { materials, size } => SdfPattern::Checker {
            material_ids: wire_ids(materials)?,
            size: *size,
        },

        SdfjPattern::Gradient {
            materials,
            axis,
            from,
            to,
            warp,
            seed,
        } => SdfPattern::Gradient {
            material_ids: wire_ids(materials)?,
            axis: ty_axis3_from_sdfj_axis(*axis),
            from: *from,
            to: *to,
            warp: *warp,
            seed: *seed,
        },

        SdfjPattern::Grain {
            materials,
            axis,
            period,
            warp,
            seed,
        } => SdfPattern::Grain {
            material_ids: wire_ids(materials)?,
            axis: ty_axis3_from_sdfj_axis(*axis),
            period: *period,
            warp: *warp,
            seed: *seed,
        },

        SdfjPattern::Noise {
            materials,
            scale,
            octaves,
            seed,
        } => SdfPattern::Noise {
            material_ids: wire_ids(materials)?,
            scale: *scale,
            octaves: *octaves,
            seed: *seed,
        },

        SdfjPattern::Speckle {
            base,
            accents,
            size,
            density,
            seed,
        } => SdfPattern::Speckle {
            base_id: wire_id(*base)?,
            accent_ids: wire_ids(accents)?,
            size: *size,
            density: *density,
            seed: *seed,
        },
    })
}

/// A step as sdfcore holds it.
fn sdf_step_from_sdfj_step(step: &SdfjStep) -> StdResult<SdfStep, String> {
    Ok(match step {
        SdfjStep::Add {
            name,
            shape,
            material,
            pattern,
        } => SdfStep::Add {
            name: name.clone(),
            shape_id: wire_id(*shape)?,
            material: sdf_step_material_from_material_and_pattern(*material, *pattern)?,
        },

        SdfjStep::Carve { name, shape } => SdfStep::Carve {
            name: name.clone(),
            shape_id: wire_id(*shape)?,
        },

        SdfjStep::Coat {
            name,
            material,
            pattern,
            sides,
            depth,
            within,
            onto,
        } => SdfStep::Coat {
            name: name.clone(),
            material: sdf_step_material_from_material_and_pattern(*material, *pattern)?,
            sides: sides
                .as_ref()
                .map(|sides| sides.iter().copied().map(sdf_side_from_sdfj_side).collect()),
            depth: *depth,
            within_id: within.map(wire_id).transpose()?,
            onto_ids: onto.as_deref().map(wire_ids).transpose()?,
        },

        SdfjStep::Paint {
            name,
            shape,
            material,
            pattern,
            onto,
        } => SdfStep::Paint {
            name: name.clone(),
            shape_id: wire_id(*shape)?,
            material: sdf_step_material_from_material_and_pattern(*material, *pattern)?,
            onto_ids: onto.as_deref().map(wire_ids).transpose()?,
        },

        SdfjStep::Set {
            name,
            points,
            size,
            material,
            pattern,
        } => SdfStep::Set {
            name: name.clone(),
            points: points
                .iter()
                .copied()
                .map(TyVector3F64::from_array)
                .collect(),
            size: *size,
            material: sdf_step_material_from_material_and_pattern(*material, *pattern)?,
        },
    })
}

/// The material or pattern a step's `material` and `pattern` keys hold.
/// Errors unless exactly one holds an index.
fn sdf_step_material_from_material_and_pattern(
    material: Option<usize>,
    pattern: Option<usize>,
) -> StdResult<SdfStepMaterial, String> {
    match (material, pattern) {
        (Some(material), None) => Ok(SdfStepMaterial::Material(wire_id(material)?)),
        (None, Some(pattern)) => Ok(SdfStepMaterial::Pattern(wire_id(pattern)?)),
        (Some(_), Some(_)) => Err("holds both material and pattern".to_owned()),
        (None, None) => Err("holds neither material nor pattern".to_owned()),
    }
}

/// An object as sdfcore holds it.
fn sdf_object_from_sdfj_object(object: &SdfjObject) -> StdResult<SdfObject, String> {
    Ok(SdfObject {
        name: object.name.clone(),
        step_ids: wire_ids(&object.steps)?,
    })
}

/// A node as sdfcore holds it.
fn sdf_node_from_sdfj_node(node: &SdfjNode) -> StdResult<SdfNode, String> {
    Ok(SdfNode {
        name: node.name.clone(),
        pivot: node.pivot.map(TyVector3F64::from_array),
        offset: node.offset.map(TyVector3F64::from_array),
        rotation: node
            .rotation
            .iter()
            .map(|axis_rotation| SdfAxisRotation {
                axis: ty_axis3_from_sdfj_axis(axis_rotation.axis),
                degrees: axis_rotation.degrees,
            })
            .collect(),
        child_object_ids: wire_ids(&node.child_objects)?,
        child_node_ids: wire_ids(&node.child_nodes)?,
    })
}

/// An axis as ty-math holds it.
fn ty_axis3_from_sdfj_axis(axis: SdfjAxis) -> TyAxis3 {
    match axis {
        SdfjAxis::X => TyAxis3::X,
        SdfjAxis::Y => TyAxis3::Y,
        SdfjAxis::Z => TyAxis3::Z,
    }
}

/// 2D mirror axes as sdfcore holds them.
fn sdf_axes2d_from_sdfj_axes2d(axes: SdfjAxes2d) -> SdfAxes2d {
    match axes {
        SdfjAxes2d::U => SdfAxes2d::U,
        SdfjAxes2d::Uv => SdfAxes2d::Uv,
        SdfjAxes2d::V => SdfAxes2d::V,
    }
}

/// 3D mirror axes as sdfcore holds them.
fn sdf_axes3d_from_sdfj_axes3d(axes: SdfjAxes3d) -> SdfAxes3d {
    match axes {
        SdfjAxes3d::X => SdfAxes3d::X,
        SdfjAxes3d::Xy => SdfAxes3d::Xy,
        SdfjAxes3d::Xyz => SdfAxes3d::Xyz,
        SdfjAxes3d::Xz => SdfAxes3d::Xz,
        SdfjAxes3d::Y => SdfAxes3d::Y,
        SdfjAxes3d::Yz => SdfAxes3d::Yz,
        SdfjAxes3d::Z => SdfAxes3d::Z,
    }
}

/// Arc caps as sdfcore holds them.
fn sdf_caps_from_sdfj_caps(caps: SdfjCaps) -> SdfCaps {
    match caps {
        SdfjCaps::Flat => SdfCaps::Flat,
        SdfjCaps::Round => SdfCaps::Round,
    }
}

/// A side as sdfcore holds it.
fn sdf_side_from_sdfj_side(side: SdfjSide) -> SdfSide {
    match side {
        SdfjSide::NegativeX => SdfSide::NegativeX,
        SdfjSide::NegativeY => SdfSide::NegativeY,
        SdfjSide::NegativeZ => SdfSide::NegativeZ,
        SdfjSide::PositiveX => SdfSide::PositiveX,
        SdfjSide::PositiveY => SdfSide::PositiveY,
        SdfjSide::PositiveZ => SdfSide::PositiveZ,
    }
}

#[cfg(test)]
mod tests {
    use crate::{CHAIR_SDFJ, EVERY_KIND_SDFJ, Error, FOREST_SDFJ, from_sdfj_file, to_sdfj_file};
    use branded_id::U32Id;
    use sdfcore::{Error as SdfError, SdfAxes3d, SdfEntryId, SdfShape3d, SdfStep, SdfStepMaterial};
    use sdfj::{SdfjFile, SdfjMap, SdfjMapEntry, SdfjStep};
    use sdfj_codec::{DependenciesImpl, from_sdfj_file_bytes};

    /// The chair document.
    fn chair() -> SdfjFile {
        from_sdfj_file_bytes(&DependenciesImpl, CHAIR_SDFJ.as_bytes()).unwrap()
    }

    #[test]
    fn each_example_round_trips() {
        for text in [CHAIR_SDFJ, EVERY_KIND_SDFJ, FOREST_SDFJ] {
            let file = from_sdfj_file_bytes(&DependenciesImpl, text.as_bytes()).unwrap();

            let main = from_sdfj_file(&file).unwrap();

            assert_eq!(to_sdfj_file(&main), file);
        }
    }

    #[test]
    fn entries_keep_their_places_and_references() {
        let main = from_sdfj_file(&chair()).unwrap();

        let state = main.state();

        assert_eq!(
            state.shapes3d[U32Id::from_u32(2).to_usize_id()],
            SdfShape3d::Mirror {
                shape_id: U32Id::from_u32(1),
                axes: SdfAxes3d::Xz,
                center: None,
            }
        );

        assert_eq!(
            state.steps[U32Id::from_u32(0).to_usize_id()],
            SdfStep::Add {
                name: "legs".to_owned(),
                shape_id: U32Id::from_u32(2),
                material: SdfStepMaterial::Pattern(U32Id::from_u32(0)),
            }
        );
    }

    #[test]
    fn a_version_other_than_one_errors() {
        let mut file = chair();

        file.version = 2;

        let error = from_sdfj_file(&file).unwrap_err();

        assert!(matches!(error, Error::Invalid(_)));

        assert_eq!(error.to_string(), "version 2 is not 1");
    }

    #[test]
    fn a_step_holds_exactly_one_of_material_and_pattern() {
        for (material, pattern, message) in [
            (Some(0), Some(0), "steps[1] holds both material and pattern"),
            (None, None, "steps[1] holds neither material nor pattern"),
        ] {
            let mut file = chair();

            file.steps[1] = SdfjStep::Add {
                name: "seat".to_owned(),
                shape: 3,
                material,
                pattern,
            };

            assert_eq!(from_sdfj_file(&file).unwrap_err().to_string(), message);
        }
    }

    #[test]
    fn an_index_past_the_id_space_errors() {
        let mut file = chair();

        file.objects[0].steps[0] += 1 << 32;

        assert_eq!(
            from_sdfj_file(&file).unwrap_err().to_string(),
            "objects[0] references index 4294967296, past the id space"
        );

        let mut file = chair();

        file.root_nodes[0] += 1 << 32;

        assert_eq!(
            from_sdfj_file(&file).unwrap_err().to_string(),
            "rootNodes references index 4294967296, past the id space"
        );

        let mut file = chair();

        file.names.parts = SdfjMap::new(vec![SdfjMapEntry {
            key: "chair".to_owned(),
            value: 1 << 32,
        }]);

        assert_eq!(
            from_sdfj_file(&file).unwrap_err().to_string(),
            "names.parts `chair` references index 4294967296, past the id space"
        );
    }

    #[test]
    fn names_keep_their_order_and_entries() {
        let file = from_sdfj_file_bytes(&DependenciesImpl, EVERY_KIND_SDFJ.as_bytes()).unwrap();

        let names = from_sdfj_file(&file).unwrap().into_state().names;

        assert_eq!(
            names.materials,
            [
                ("rune".to_owned(), U32Id::from_u32(0)),
                ("glass".to_owned(), U32Id::from_u32(1)),
                ("stone".to_owned(), U32Id::from_u32(0)),
            ]
        );

        assert_eq!(names.parts, [("part".to_owned(), U32Id::from_u32(0))]);
    }

    #[test]
    fn a_broken_reference_errors_with_the_sdfcore_rule() {
        let mut file = chair();

        file.objects[0].steps.push(5);

        let error = from_sdfj_file(&file).unwrap_err();

        assert!(matches!(
            error,
            Error::Sdf(SdfError::Reference {
                entry_id: SdfEntryId::Object(_),
                referenced_entry_id: SdfEntryId::Step(_),
            })
        ));

        assert_eq!(
            error.to_string(),
            "objects[0] references steps[5], past the end of steps"
        );
    }
}
