use crate::{
    Error, Result,
    operations::mesh::{
        ArrayDomain, Atlases, AttributeWrite, MeshElement, ProgramRun, Transfer, encode_components,
    },
};
use branded_id::U32Id;
use meshdoc::{BMeshPrimitive, MeshAttributeComponents, MeshPrimitive, MeshVertexAttribute};
use ty_math::TyLinSrgbaF64;
use vox_value_language::{Components, Domain, Scalar, Value, eval_expression};

/// Lands `attributes` on `primitive`, each read at the corners of `faces`
/// with lower domains climbing in.
pub(crate) fn write_attributes(
    primitive: &mut MeshPrimitive,
    primitive_id: U32Id<BMeshPrimitive>,
    attributes: &[AttributeWrite],
    faces: &[usize],
    run: &ProgramRun,
    atlases: &Atlases<'_>,
) -> Result<()> {
    for attribute in attributes {
        let element = MeshElement::PrimitiveAttribute {
            primitive_id,
            name: attribute.name().to_owned(),
        };

        let checked = run
            .destinations
            .iter()
            .find(|checked| checked.destination.element == element)
            .expect("every attribute is a destination");

        let value = eval_expression(&checked.expression, &run.evaluated)
            .map_err(|error| Error::mesh_record(element.clone(), error))?;

        let entries: Vec<usize> = faces
            .iter()
            .flat_map(|&face| corner_entries(atlases, &value, face))
            .collect();

        match attribute {
            AttributeWrite::Builtin { attribute, .. } => {
                if attribute != "COLOR_0" {
                    return Err(Error::mesh_record(
                        element,
                        format!(
                            "is `{attribute}`, and the modeled vocabulary holds `COLOR_0` alone"
                        ),
                    ));
                }

                if primitive.colors().is_some() {
                    return Err(Error::mesh_record(element, "is written twice"));
                }

                primitive.set_colors(Some(vertex_colors(&element, &value, &entries)?))?;
            }

            AttributeWrite::Custom {
                name,
                value: written,
            } => {
                let components = custom_components(&element, &value, written.transfer, &entries)?;

                primitive.push_vertex_attribute(MeshVertexAttribute {
                    name: name.clone(),
                    width: value.dimension().width(),
                    components,
                })?;
            }
        }
    }

    Ok(())
}

/// The entry each corner of `face` reads from `value`, lower domains climbing
/// in.
fn corner_entries(atlases: &Atlases<'_>, value: &Value, face: usize) -> [usize; 4] {
    if value.domain() == Domain::Corner {
        return [0, 1, 2, 3].map(|corner| face * 4 + corner);
    }

    let mut pieces = atlases
        .cell_entries(ArrayDomain::Corner, value.domain(), face)
        .into_iter();

    let entry = pieces.next().expect("a face covers a voxel");

    assert!(
        pieces.all(|other| entries_agree(value, entry, other)),
        "a face's voxels agree on an attribute"
    );

    [entry; 4]
}

/// Whether entries `left` and `right` of `value` hold the same components.
fn entries_agree(value: &Value, left: usize, right: usize) -> bool {
    let width = value.dimension().width();
    let range = |entry: usize| entry * width..(entry + 1) * width;

    match value.components() {
        Components::Bool(components) => components[range(left)] == components[range(right)],

        Components::F32(components) => components[range(left)]
            .iter()
            .map(|component| component.to_bits())
            .eq(components[range(right)]
                .iter()
                .map(|component| component.to_bits())),

        Components::String(components) => components[range(left)] == components[range(right)],
        Components::U8(components) => components[range(left)] == components[range(right)],
        Components::U16(components) => components[range(left)] == components[range(right)],
        Components::U32(components) => components[range(left)] == components[range(right)],
    }
}

/// One linear color per entry of `entries` from an f32 vec3 or vec4 `value`
/// in `[0, 1]`. A vec3 takes an alpha of one.
fn vertex_colors(
    element: &MeshElement,
    value: &Value,
    entries: &[usize],
) -> Result<Vec<TyLinSrgbaF64>> {
    let width = value.dimension().width();

    let components = match value.components() {
        Components::F32(components) if width == 3 || width == 4 => components,
        _ => {
            return Err(Error::mesh_record(
                element.clone(),
                format!(
                    "is a {}, and `COLOR_0` takes an f32 vec3 or vec4",
                    value.to_type()
                ),
            ));
        }
    };

    entries
        .iter()
        .map(|&entry| {
            let encoded = encode_components(
                element,
                &components[entry * width..(entry + 1) * width],
                Transfer::Linear,
                true,
            )?;

            let alpha = if width == 4 { encoded[3] } else { 1.0 };

            Ok(TyLinSrgbaF64::new(
                encoded[0], encoded[1], encoded[2], alpha,
            ))
        })
        .collect()
}

/// The components of `entries` of `value` under `transfer`, in the value's
/// scalar.
fn custom_components(
    element: &MeshElement,
    value: &Value,
    transfer: Transfer,
    entries: &[usize],
) -> Result<MeshAttributeComponents> {
    let kind = value.to_type();
    let width = value.dimension().width();
    let range = |entry: usize| entry * width..(entry + 1) * width;

    if transfer == Transfer::Srgb && value.scalar() != Scalar::F32 {
        return Err(Error::mesh_record(
            element.clone(),
            format!("is a {kind}, and `srgb` transfers f32 alone"),
        ));
    }

    Ok(match value.components() {
        Components::Bool(_) | Components::String(_) => {
            return Err(Error::mesh_record(
                element.clone(),
                format!("is a {kind}, and an attribute takes f32, u8, or u16"),
            ));
        }

        Components::F32(components) => MeshAttributeComponents::F64(
            entries
                .iter()
                .map(|&entry| {
                    encode_components(element, &components[range(entry)], transfer, false)
                })
                .collect::<Result<Vec<_>>>()?
                .concat(),
        ),

        Components::U8(components) => MeshAttributeComponents::U8(
            entries
                .iter()
                .flat_map(|&entry| components[range(entry)].iter().copied())
                .collect(),
        ),

        Components::U16(components) => MeshAttributeComponents::U16(
            entries
                .iter()
                .flat_map(|&entry| components[range(entry)].iter().copied())
                .collect(),
        ),

        Components::U32(_) => {
            return Err(Error::mesh_record(
                element.clone(),
                format!(
                    "is a {kind}, and glTF forbids the width on an attribute; narrow with `u16()`"
                ),
            ));
        }
    })
}
