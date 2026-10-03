use crate::{
    Error, Result,
    operations::object::{
        AttributeWrite, CheckedDestination, ExtraForm, ExtraSource, ExtraWrite, FileForm, Landing,
        MeshElement, MeshRecord, SlotProperty, SlotSource,
    },
};
use branded_id::ext::IteratorExt;
use vox_value_language::{CheckedProgram, Expression, check_expression, parse_expression};

/// A record element holding an expression the run writes somewhere.
#[derive(Clone, Debug, PartialEq)]
pub struct Destination {
    pub element: MeshElement,

    pub landing: Landing,

    pub text: String,

    pub expression: Expression,
}

impl Destination {
    /// Every destination `record` holds, in table order, with each expression
    /// parsed. Errors if a slot names a property the material model does not
    /// hold, or on an expression that fails to parse.
    pub(crate) fn of_record(record: &MeshRecord) -> Result<Vec<Self>> {
        let mut destinations = Vec::new();

        let extras = |destinations: &mut Vec<Self>,
                      extras: &[ExtraWrite],
                      element: &dyn Fn(&str) -> MeshElement|
         -> Result<()> {
            for extra in extras {
                let ExtraSource::Value(value) = &extra.source else {
                    continue;
                };

                destinations.push(parsed(
                    element(&extra.name),
                    match extra.form {
                        ExtraForm::Image => Landing::Texture,
                        ExtraForm::Json => Landing::Json,
                    },
                    &value.expression,
                )?);
            }

            Ok(())
        };

        for (material_id, material) in record.materials.iter().enumerate_ids() {
            for slot in &material.slots {
                let element = MeshElement::Slot {
                    material_id,
                    property: slot.property.clone(),
                };

                let Some(property) = SlotProperty::parse(&slot.property) else {
                    return Err(Error::mesh_record(
                        element,
                        "is not a material property the document models",
                    ));
                };

                let SlotSource::Value(text) = &slot.source else {
                    continue;
                };

                let landing = if property.is_texture() {
                    Landing::Texture
                } else {
                    Landing::Factor
                };

                destinations.push(parsed(element, landing, text)?);
            }

            extras(&mut destinations, &material.extras, &|name| {
                MeshElement::MaterialExtra {
                    material_id,
                    name: name.to_owned(),
                }
            })?;
        }

        for (primitive_id, primitive) in record.primitives.iter().enumerate_ids() {
            destinations.push(parsed(
                MeshElement::PrimitiveSelect { primitive_id },
                Landing::Select,
                &primitive.select,
            )?);

            for attribute in &primitive.attributes {
                let text = match attribute {
                    AttributeWrite::Builtin { expression, .. } => expression,
                    AttributeWrite::Custom { value, .. } => &value.expression,
                };

                destinations.push(parsed(
                    MeshElement::PrimitiveAttribute {
                        primitive_id,
                        name: attribute.name().to_owned(),
                    },
                    Landing::Attribute,
                    text,
                )?);
            }
        }

        for file in &record.files {
            destinations.push(parsed(
                MeshElement::File {
                    file: file.file.clone(),
                },
                match file.form {
                    FileForm::Json { .. } => Landing::Json,
                    FileForm::Png => Landing::Texture,
                },
                &file.value.expression,
            )?);
        }

        extras(&mut destinations, &record.mesh_extras, &|name| {
            MeshElement::MeshExtra {
                name: name.to_owned(),
            }
        })?;

        Ok(destinations)
    }

    /// Checks the expression in the scope at `checked`'s end. Every error
    /// rises from the element.
    pub(crate) fn check(self, checked: &CheckedProgram) -> Result<CheckedDestination> {
        let expression = check_expression(&self.expression, checked)
            .map_err(|error| Error::mesh_record(self.element.clone(), error))?;

        Ok(CheckedDestination {
            destination: self,
            expression,
        })
    }
}

/// Parses `text` into the destination at `element`. A parse error rises from
/// the element.
fn parsed(element: MeshElement, landing: Landing, text: &str) -> Result<Destination> {
    let expression =
        parse_expression(text).map_err(|error| Error::mesh_record(element.clone(), error))?;

    Ok(Destination {
        element,
        landing,
        text: text.to_owned(),
        expression,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        Error,
        operations::object::{
            AttributeWrite, Destination, ExtraForm, ExtraSource, ExtraWrite, FileForm, FileWrite,
            Landing, MaterialRecord, MeshElement, MeshRecord, Method, PrimitiveRecord, SlotSource,
            SlotWrite, TextureShape, Transfer, WrittenValue,
        },
    };
    use branded_id::{IdVec, U32Id};

    fn written(expression: &str) -> WrittenValue {
        WrittenValue {
            expression: expression.to_owned(),
            transfer: Transfer::Linear,
        }
    }

    /// A record of one material with `slots` and the implicit primitive.
    fn with_slots(slots: Vec<SlotWrite>) -> MeshRecord {
        MeshRecord {
            method: Method::Greedy,
            texture_shape: TextureShape::Pot,
            voxel_size: 1.0,
            computed_bindings: Vec::new(),
            program: String::new(),
            materials: IdVec::from_vec(vec![MaterialRecord {
                slots,
                ..Default::default()
            }]),
            primitives: IdVec::from_vec(vec![PrimitiveRecord {
                material_id: Some(U32Id::from_u32(0)),
                select: "true".to_owned(),
                name: None,
                normal: true,
                uv_streams: None,
                attributes: Vec::new(),
            }]),
            files: Vec::new(),
            mesh_extras: Vec::new(),
        }
    }

    #[test]
    fn a_slot_outside_the_material_model_errors_and_names_itself() {
        let record = with_slots(vec![SlotWrite {
            property: "subsurface".to_owned(),
            source: SlotSource::File("skin.png".to_owned()),
        }]);

        let error = Destination::of_record(&record).unwrap_err();

        assert!(
            matches!(
                &error,
                Error::MeshRecord {
                    element: MeshElement::Slot { property, .. },
                    ..
                } if property == "subsurface"
            ),
            "{error}"
        );
    }

    #[test]
    fn an_expression_that_fails_to_parse_errors_from_its_element() {
        let record = with_slots(vec![SlotWrite {
            property: "emissiveStrength".to_owned(),
            source: SlotSource::Value("0.5 +".to_owned()),
        }]);

        let error = Destination::of_record(&record).unwrap_err();

        assert!(
            matches!(
                &error,
                Error::MeshRecord {
                    element: MeshElement::Slot { property, .. },
                    ..
                } if property == "emissiveStrength"
            ),
            "{error}"
        );
    }

    #[test]
    fn every_expression_of_the_record_is_a_destination_and_files_are_not() {
        let record = MeshRecord {
            method: Method::Greedy,
            texture_shape: TextureShape::Pot,
            voxel_size: 1.0,
            computed_bindings: Vec::new(),
            program: String::new(),
            materials: IdVec::from_vec(vec![MaterialRecord {
                name: None,
                uv_streams: None,
                slots: vec![
                    SlotWrite {
                        property: "baseColorTexture".to_owned(),
                        source: SlotSource::Value("albedo".to_owned()),
                    },
                    SlotWrite {
                        property: "occlusionTexture".to_owned(),
                        source: SlotSource::File("ao.png".to_owned()),
                    },
                    SlotWrite {
                        property: "emissiveStrength".to_owned(),
                        source: SlotSource::Value("max(strength)".to_owned()),
                    },
                ],
                extras: vec![ExtraWrite {
                    name: "glow".to_owned(),
                    form: ExtraForm::Image,
                    source: ExtraSource::Value(written("emissive")),
                }],
            }]),
            primitives: IdVec::from_vec(vec![PrimitiveRecord {
                material_id: Some(U32Id::from_u32(0)),
                select: "solid".to_owned(),
                name: None,
                normal: true,
                uv_streams: None,
                attributes: vec![
                    AttributeWrite::Builtin {
                        attribute: "COLOR_0".to_owned(),
                        expression: "tint".to_owned(),
                    },
                    AttributeWrite::Custom {
                        name: "_PALETTE".to_owned(),
                        value: written("u8(swatchIndex)"),
                    },
                ],
            }]),
            files: vec![
                FileWrite {
                    file: "ao.png".to_owned(),
                    value: written("ao"),
                    form: FileForm::Png,
                },
                FileWrite {
                    file: "meta.json".to_owned(),
                    value: written("count"),
                    form: FileForm::Json {
                        name: "count".to_owned(),
                    },
                },
            ],
            mesh_extras: vec![
                ExtraWrite {
                    name: "height".to_owned(),
                    form: ExtraForm::Json,
                    source: ExtraSource::Value(written("max(voxelPosition.z)")),
                },
                ExtraWrite {
                    name: "thumb".to_owned(),
                    form: ExtraForm::Image,
                    source: ExtraSource::File("thumb.png".to_owned()),
                },
            ],
        };

        let destinations = Destination::of_record(&record).unwrap();

        let material_id = U32Id::from_u32(0);
        let primitive_id = U32Id::from_u32(0);
        let summary: Vec<(MeshElement, Landing, &str)> = destinations
            .iter()
            .map(|destination| {
                (
                    destination.element.clone(),
                    destination.landing,
                    destination.text.as_str(),
                )
            })
            .collect();
        assert_eq!(
            summary,
            [
                (
                    MeshElement::Slot {
                        material_id,
                        property: "baseColorTexture".to_owned()
                    },
                    Landing::Texture,
                    "albedo"
                ),
                (
                    MeshElement::Slot {
                        material_id,
                        property: "emissiveStrength".to_owned()
                    },
                    Landing::Factor,
                    "max(strength)"
                ),
                (
                    MeshElement::MaterialExtra {
                        material_id,
                        name: "glow".to_owned()
                    },
                    Landing::Texture,
                    "emissive"
                ),
                (
                    MeshElement::PrimitiveSelect { primitive_id },
                    Landing::Select,
                    "solid"
                ),
                (
                    MeshElement::PrimitiveAttribute {
                        primitive_id,
                        name: "COLOR_0".to_owned()
                    },
                    Landing::Attribute,
                    "tint"
                ),
                (
                    MeshElement::PrimitiveAttribute {
                        primitive_id,
                        name: "_PALETTE".to_owned()
                    },
                    Landing::Attribute,
                    "u8(swatchIndex)"
                ),
                (
                    MeshElement::File {
                        file: "ao.png".to_owned()
                    },
                    Landing::Texture,
                    "ao"
                ),
                (
                    MeshElement::File {
                        file: "meta.json".to_owned()
                    },
                    Landing::Json,
                    "count"
                ),
                (
                    MeshElement::MeshExtra {
                        name: "height".to_owned()
                    },
                    Landing::Json,
                    "max(voxelPosition.z)"
                ),
            ]
        );
    }
}
