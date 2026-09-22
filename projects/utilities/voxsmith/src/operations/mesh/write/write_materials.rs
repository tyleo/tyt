use crate::{
    Error, Result,
    dependencies::mesh::EncodePng,
    operations::mesh::{
        FileForm, Images, MeshElement, SlotProperty, SlotSource, WriteContext, table_index,
        write_extras,
    },
};
use branded_id::U32Id;
use meshdoc::{
    MeshAlphaMode, MeshMain, MeshMaterial, MeshTextureRef,
    material::{
        ALPHA_CUTOFF, COLOR_RANGE, EMISSIVE_STRENGTH, IOR, METALLIC, NORMAL_SCALE,
        OCCLUSION_STRENGTH, ROUGHNESS, TRANSMISSION, scalar_range,
    },
};
use ty_math::{TyLinSrgbF64, TyLinSrgbaF64};
use vox_value_language::{Components, Dimension, Domain, Value, eval_expression};

/// Retains the record's materials into `document` in table order, each slot
/// filling its modeled field and each extra a named property.
pub(crate) fn write_materials<D: EncodePng>(
    context: &WriteContext<'_, D>,
    document: &mut MeshMain<()>,
    images: &mut Images,
) -> Result<()> {
    let WriteContext {
        dependencies,
        record,
        run,
        streams,
        atlases,
        file_ids,
    } = *context;

    for (index, material_record) in record.materials.iter().enumerate() {
        let material_id = U32Id::from_u32(table_index(index));

        let mut material = MeshMaterial::new(material_record.name.clone().unwrap_or_default());

        for slot in &material_record.slots {
            let element = MeshElement::Slot {
                material_id,
                property: slot.property.clone(),
            };

            let property = SlotProperty::parse(&slot.property)
                .expect("the destinations checked every slot property");

            let texture_ref = |texture_id| MeshTextureRef {
                texture_id,
                uv_stream_id: streams.stream_id(material_id, streams.bake(&element)),
            };

            match &slot.source {
                SlotSource::File(file) => {
                    if !property.is_texture() {
                        return Err(Error::mesh_record(
                            element,
                            format!("references a file, and `{}` takes a value", property.name()),
                        ));
                    }

                    let written = record
                        .files
                        .iter()
                        .find(|write| write.file == *file && write.form == FileForm::Png)
                        .expect("the streams checked every referenced file");

                    if written.value.transfer != property.transfer() {
                        return Err(Error::mesh_record(
                            element,
                            format!(
                                "references `{file}`, written as {}, and `{}` takes a {} image",
                                written.value.transfer,
                                property.name(),
                                property.transfer()
                            ),
                        ));
                    }

                    let file_id = file_ids[file];

                    let texture_id =
                        images.reference(document, file, file_id, streams.bake(&element))?;

                    set_texture(&mut material, property, texture_ref(texture_id));
                }

                SlotSource::Value(_) => {
                    let checked = run
                        .destinations
                        .iter()
                        .find(|checked| checked.destination.element == element)
                        .expect("every slot value is a destination");

                    if property.is_texture() {
                        let texture_id = images.embed(
                            dependencies,
                            document,
                            &element,
                            checked,
                            &run.evaluated,
                            property.transfer(),
                            streams.bake(&element),
                            atlases,
                        )?;

                        set_texture(&mut material, property, texture_ref(texture_id));
                    } else {
                        let value = eval_expression(&checked.expression, &run.evaluated)
                            .map_err(|error| Error::mesh_record(element.clone(), error))?;

                        set_factor(&mut material, property, &element, &value)?;
                    }
                }
            }
        }

        material.properties = write_extras(
            context,
            document,
            images,
            &material_record.extras,
            |name| MeshElement::MaterialExtra {
                material_id,
                name: name.to_owned(),
            },
            |bake| streams.stream_id(material_id, bake),
        )?;

        let retained_id = document.retain_material(material)?;

        assert_eq!(
            retained_id, material_id,
            "the materials retain in table order"
        );
    }

    Ok(())
}

/// Points the texture slot `property` of `material` at `texture_ref`.
fn set_texture(material: &mut MeshMaterial, property: SlotProperty, texture_ref: MeshTextureRef) {
    let slot = match property {
        SlotProperty::BaseColorTexture => &mut material.base_color_texture,
        SlotProperty::EmissiveTexture => &mut material.emissive_texture,
        SlotProperty::MetallicRoughnessTexture => &mut material.metallic_roughness_texture,
        SlotProperty::NormalTexture => &mut material.normal_texture,
        SlotProperty::OcclusionTexture => &mut material.occlusion_texture,
        SlotProperty::TransmissionTexture => &mut material.transmission_texture,
        _ => unreachable!("a factor takes no texture"),
    };

    *slot = Some(texture_ref);
}

/// Sets the factor `property` of `material` to the plain `value`. Errors from
/// `element` on a value of the wrong shape or outside the field's range.
fn set_factor(
    material: &mut MeshMaterial,
    property: SlotProperty,
    element: &MeshElement,
    value: &Value,
) -> Result<()> {
    let name = property.name();
    let kind = value.to_type();

    let wrong_shape = |takes: &str| {
        Error::mesh_record(
            element.clone(),
            format!("is a {kind}, and `{name}` takes a plain {takes}"),
        )
    };

    if value.domain() != Domain::Plain {
        return Err(wrong_shape(match property {
            SlotProperty::AlphaMode => "string",
            SlotProperty::BaseColorFactor => "f32 vec4",
            SlotProperty::DoubleSided => "bool",
            SlotProperty::EmissiveFactor => "f32 vec3",
            _ => "f32 vec1",
        }));
    }

    let floats = |width: Dimension| match value.components() {
        Components::F32(components) if value.dimension() == width => Some(components.as_slice()),
        _ => None,
    };

    let in_range = |key: &str, component: f64| -> Result<f64> {
        let range = match key {
            "color" => COLOR_RANGE,
            _ => scalar_range(key).expect("every scalar factor has a range"),
        };

        if range.contains(component) {
            Ok(component)
        } else {
            Err(Error::mesh_record(
                element.clone(),
                format!("is {component}, outside the range {range}"),
            ))
        }
    };

    match property {
        SlotProperty::AlphaMode => {
            let Components::String(components) = value.components() else {
                return Err(wrong_shape("string"));
            };

            material.alpha_mode = match components[0].as_str() {
                "BLEND" => MeshAlphaMode::Blend,
                "MASK" => MeshAlphaMode::Mask,
                "OPAQUE" => MeshAlphaMode::Opaque,
                token => {
                    return Err(Error::mesh_record(
                        element.clone(),
                        format!("is \"{token}\", and `{name}` takes OPAQUE, MASK, or BLEND"),
                    ));
                }
            };
        }

        SlotProperty::BaseColorFactor => {
            let components = floats(Dimension::Vec4).ok_or_else(|| wrong_shape("f32 vec4"))?;

            let [red, green, blue, alpha] = [0, 1, 2, 3]
                .map(|index| in_range("color", f64::from(components[index])))
                .into_iter()
                .collect::<Result<Vec<_>>>()?
                .try_into()
                .expect("four components");

            material.base_color_factor = TyLinSrgbaF64::new(red, green, blue, alpha);
        }

        SlotProperty::DoubleSided => {
            let Components::Bool(components) = value.components() else {
                return Err(wrong_shape("bool"));
            };

            material.double_sided = components[0];
        }

        SlotProperty::EmissiveFactor => {
            let components = floats(Dimension::Vec3).ok_or_else(|| wrong_shape("f32 vec3"))?;

            let [red, green, blue] = [0, 1, 2]
                .map(|index| in_range("color", f64::from(components[index])))
                .into_iter()
                .collect::<Result<Vec<_>>>()?
                .try_into()
                .expect("three components");

            material.emissive_factor = TyLinSrgbF64::new(red, green, blue);
        }

        SlotProperty::AlphaCutoff
        | SlotProperty::EmissiveStrength
        | SlotProperty::Ior
        | SlotProperty::MetallicFactor
        | SlotProperty::NormalScale
        | SlotProperty::OcclusionStrength
        | SlotProperty::RoughnessFactor
        | SlotProperty::TransmissionFactor => {
            let components = floats(Dimension::Vec1).ok_or_else(|| wrong_shape("f32 vec1"))?;

            let key = match property {
                SlotProperty::AlphaCutoff => ALPHA_CUTOFF,
                SlotProperty::EmissiveStrength => EMISSIVE_STRENGTH,
                SlotProperty::Ior => IOR,
                SlotProperty::MetallicFactor => METALLIC,
                SlotProperty::NormalScale => NORMAL_SCALE,
                SlotProperty::OcclusionStrength => OCCLUSION_STRENGTH,
                SlotProperty::RoughnessFactor => ROUGHNESS,
                _ => TRANSMISSION,
            };

            let scalar = in_range(key, f64::from(components[0]))?;

            assert!(
                material.set_scalar(key, scalar),
                "every scalar factor has a modeled field"
            );
        }

        SlotProperty::BaseColorTexture
        | SlotProperty::EmissiveTexture
        | SlotProperty::MetallicRoughnessTexture
        | SlotProperty::NormalTexture
        | SlotProperty::OcclusionTexture
        | SlotProperty::TransmissionTexture => {
            unreachable!("a texture slot embeds or references an image")
        }
    }

    Ok(())
}
