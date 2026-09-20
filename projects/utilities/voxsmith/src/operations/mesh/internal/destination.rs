use crate::{
    Error, Result,
    operations::mesh::{
        AttributeWrite, CheckedDestination, ExtraForm, ExtraSource, ExtraWrite, FileForm, Landing,
        MeshElement, MeshRecord, SlotSource,
    },
};
use branded_id::U32Id;
use vox_value_language::{CheckedProgram, check_expression, parse_expression};

/// A record element holding an expression the run writes somewhere.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Destination {
    pub(crate) element: MeshElement,
    pub(crate) landing: Landing,
    pub(crate) text: String,
}

impl Destination {
    /// Every destination `record` holds, in table order.
    pub(crate) fn of_record(record: &MeshRecord) -> Vec<Self> {
        let mut destinations = Vec::new();

        let extras = |destinations: &mut Vec<Self>,
                      extras: &[ExtraWrite],
                      element: &dyn Fn(&str) -> MeshElement| {
            for extra in extras {
                let ExtraSource::Value(value) = &extra.source else {
                    continue;
                };

                destinations.push(Destination {
                    element: element(&extra.name),
                    landing: match extra.form {
                        ExtraForm::Image => Landing::Texture,
                        ExtraForm::Json => Landing::Json,
                    },
                    text: value.expression.clone(),
                });
            }
        };

        for (index, material) in record.materials.iter().enumerate() {
            let material_id = U32Id::from_u32(table_index(index));

            for slot in &material.slots {
                let SlotSource::Value(text) = &slot.source else {
                    continue;
                };

                destinations.push(Destination {
                    element: MeshElement::Slot {
                        material_id,
                        property: slot.property.clone(),
                    },
                    landing: Landing::Texture,
                    text: text.clone(),
                });
            }

            extras(&mut destinations, &material.extras, &|name| {
                MeshElement::MaterialExtra {
                    material_id,
                    name: name.to_owned(),
                }
            });
        }

        for (index, primitive) in record.primitives.iter().enumerate() {
            let primitive_id = U32Id::from_u32(table_index(index));

            destinations.push(Destination {
                element: MeshElement::PrimitiveSelect { primitive_id },
                landing: Landing::Select,
                text: primitive.select.clone(),
            });

            for attribute in &primitive.attributes {
                let text = match attribute {
                    AttributeWrite::Builtin { expression, .. } => expression,
                    AttributeWrite::Custom { value, .. } => &value.expression,
                };

                destinations.push(Destination {
                    element: MeshElement::PrimitiveAttribute {
                        primitive_id,
                        name: attribute.name().to_owned(),
                    },
                    landing: Landing::Attribute,
                    text: text.clone(),
                });
            }
        }

        for file in &record.files {
            destinations.push(Destination {
                element: MeshElement::File {
                    file: file.file.clone(),
                },
                landing: match file.form {
                    FileForm::Json { .. } => Landing::Json,
                    FileForm::Png => Landing::Texture,
                },
                text: file.value.expression.clone(),
            });
        }

        extras(&mut destinations, &record.mesh_extras, &|name| {
            MeshElement::MeshExtra {
                name: name.to_owned(),
            }
        });

        destinations
    }

    /// Parses and checks the expression in the scope at `checked`'s end.
    /// Every error rises from the element.
    pub(crate) fn check(self, checked: &CheckedProgram) -> Result<CheckedDestination> {
        let parsed = parse_expression(&self.text)
            .map_err(|error| Error::mesh_record(self.element.clone(), error))?;

        let expression = check_expression(&parsed, checked)
            .map_err(|error| Error::mesh_record(self.element.clone(), error))?;

        Ok(CheckedDestination {
            destination: self,
            expression,
        })
    }
}

fn table_index(index: usize) -> u32 {
    u32::try_from(index).expect("a record table is shorter than u32::MAX")
}

#[cfg(test)]
mod tests {
    use crate::operations::mesh::{
        AttributeWrite, Destination, ExtraForm, ExtraSource, ExtraWrite, FileForm, FileWrite,
        Landing, MaterialRecord, MeshElement, MeshRecord, Method, PrimitiveRecord, SlotSource,
        SlotWrite, TextureShape, Transfer, WrittenValue,
    };
    use branded_id::{IdVec, U32Id};

    fn written(expression: &str) -> WrittenValue {
        WrittenValue {
            expression: expression.to_owned(),
            transfer: Transfer::Linear,
        }
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

        let destinations = Destination::of_record(&record);

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
