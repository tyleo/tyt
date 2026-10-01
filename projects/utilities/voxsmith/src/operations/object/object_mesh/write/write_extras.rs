use crate::{
    Error, Result,
    dependencies::object::EncodePng,
    operations::object::{
        ArrayDomain, CheckedDestination, ExtraForm, ExtraSource, ExtraWrite, FileForm, Images,
        MeshElement, Transfer, WriteContext, encode_components,
    },
};
use branded_id::U32Id;
use meshdoc::{BMeshUvStream, MeshMain, MeshProperty, MeshPropertyValue, MeshTextureRef};
use std::collections::HashSet;
use vox_value_language::{Components, Scalar, Value, eval_expression};

/// The properties `extras` land as, in write order. `element` names each
/// entry's record element, and `stream_id` gives the stream an image samples
/// through at its bake domain.
pub fn write_extras<D: EncodePng>(
    context: &WriteContext<'_, D>,
    document: &mut MeshMain<()>,
    images: &mut Images,
    extras: &[ExtraWrite],
    element: impl Fn(&str) -> MeshElement,
    stream_id: impl Fn(ArrayDomain) -> U32Id<BMeshUvStream>,
) -> Result<Vec<MeshProperty>> {
    let mut names = HashSet::new();

    extras
        .iter()
        .map(|extra| {
            let element = element(&extra.name);

            if !names.insert(extra.name.as_str()) {
                return Err(Error::mesh_record(element, "is written twice"));
            }

            let value = match (extra.form, &extra.source) {
                (ExtraForm::Image, ExtraSource::File(file)) => {
                    let file_id = *context
                        .file_ids
                        .get(file)
                        .expect("the streams checked every referenced png");

                    let bake = context.streams.bake(&element);

                    let texture_id = images.reference(document, file, file_id, bake)?;

                    MeshPropertyValue::Texture(MeshTextureRef {
                        texture_id,
                        uv_stream_id: stream_id(bake),
                    })
                }

                (ExtraForm::Image, ExtraSource::Value(written)) => {
                    let bake = context.streams.bake(&element);

                    let texture_id = images.embed(
                        context.dependencies,
                        document,
                        &element,
                        checked(context, &element),
                        &context.run.evaluated,
                        written.transfer,
                        bake,
                        context.atlases,
                    )?;

                    MeshPropertyValue::Texture(MeshTextureRef {
                        texture_id,
                        uv_stream_id: stream_id(bake),
                    })
                }

                (ExtraForm::Json, ExtraSource::File(file)) => {
                    let written = context
                        .record
                        .files
                        .iter()
                        .find(|write| write.file == *file);

                    match written.map(|write| &write.form) {
                        Some(FileForm::Json { .. }) => MeshPropertyValue::File(
                            *context
                                .file_ids
                                .get(file)
                                .expect("the document retained every written file"),
                        ),

                        Some(FileForm::Png) => {
                            return Err(Error::mesh_record(
                                element,
                                format!("references `{file}`, which the run writes as a png"),
                            ));
                        }

                        None => {
                            return Err(Error::mesh_record(
                                element,
                                format!(
                                    "references `{file}`, which the run writes as no JSON object"
                                ),
                            ));
                        }
                    }
                }

                (ExtraForm::Json, ExtraSource::Value(written)) => {
                    let value = eval_expression(
                        &checked(context, &element).expression,
                        &context.run.evaluated,
                    )
                    .map_err(|error| Error::mesh_record(element.clone(), error))?;

                    extra_property_value(&element, &value, written.transfer)?
                }
            };

            Ok(MeshProperty {
                name: extra.name.clone(),
                value,
            })
        })
        .collect()
}

/// The checked destination at `element`.
fn checked<'a, D: EncodePng>(
    context: &'a WriteContext<'_, D>,
    element: &MeshElement,
) -> &'a CheckedDestination {
    context
        .run
        .destinations
        .iter()
        .find(|checked| checked.destination.element == *element)
        .expect("every extras value is a destination")
}

/// The property `value` lands as under `transfer`.
fn extra_property_value(
    element: &MeshElement,
    value: &Value,
    transfer: Transfer,
) -> Result<MeshPropertyValue> {
    if transfer == Transfer::Srgb && value.scalar() != Scalar::F64 {
        return Err(Error::mesh_record(
            element.clone(),
            format!("is a {}, and `srgb` transfers f64 alone", value.to_type()),
        ));
    }

    let shape = Shape {
        width: value.dimension().width(),
        array: value.domain().is_array(),
    };

    let ints = |components: Vec<i64>| {
        shape.land(
            &components,
            MeshPropertyValue::Int,
            MeshPropertyValue::Ints,
            MeshPropertyValue::IntRows,
        )
    };

    Ok(match value.components() {
        Components::Bool(components) => shape.land(
            components,
            MeshPropertyValue::Bool,
            MeshPropertyValue::Bools,
            MeshPropertyValue::BoolRows,
        ),

        Components::F64(components) => {
            let encoded: Vec<f64> = components
                .chunks_exact(shape.width)
                .map(|entry| encode_components(element, entry, transfer, false))
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .flatten()
                .collect();

            shape.land(
                &encoded,
                MeshPropertyValue::Float,
                MeshPropertyValue::Floats,
                MeshPropertyValue::FloatRows,
            )
        }

        Components::String(components) => shape.land(
            components,
            MeshPropertyValue::Text,
            MeshPropertyValue::Texts,
            MeshPropertyValue::TextRows,
        ),

        Components::U8(components) => {
            ints(components.iter().map(|&number| i64::from(number)).collect())
        }

        Components::U16(components) => {
            ints(components.iter().map(|&number| i64::from(number)).collect())
        }

        Components::U32(components) => {
            ints(components.iter().map(|&number| i64::from(number)).collect())
        }
    })
}

/// The shape a value's components land in.
#[derive(Clone, Copy)]
struct Shape {
    width: usize,

    array: bool,
}

impl Shape {
    /// `components` as a leaf for a plain vec1, a list for a plain vecN or a
    /// vec1 array, or rows of `width` for a vecN array.
    fn land<T: Clone>(
        self,
        components: &[T],
        leaf: fn(T) -> MeshPropertyValue,
        list: fn(Vec<T>) -> MeshPropertyValue,
        rows: fn(Vec<Vec<T>>) -> MeshPropertyValue,
    ) -> MeshPropertyValue {
        if !self.array && self.width == 1 {
            leaf(components[0].clone())
        } else if !self.array || self.width == 1 {
            list(components.to_vec())
        } else {
            rows(
                components
                    .chunks_exact(self.width)
                    .map(<[T]>::to_vec)
                    .collect(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::object::{
        MeshElement, Transfer, object_mesh::write::write_extras::extra_property_value,
    };
    use meshdoc::MeshPropertyValue;
    use vox_value_language::{Components, Dimension, Domain, Value};

    fn element() -> MeshElement {
        MeshElement::MeshExtra {
            name: "bar".to_owned(),
        }
    }

    fn landed(domain: Domain, dimension: Dimension, components: Components) -> MeshPropertyValue {
        let value = Value::new(domain, dimension, components).unwrap();
        extra_property_value(&element(), &value, Transfer::Linear).unwrap()
    }

    #[test]
    fn a_plain_vec1_lands_as_a_leaf_and_wider_shapes_as_lists_and_rows() {
        assert_eq!(
            landed(Domain::Plain, Dimension::Vec1, Components::F64(vec![0.4])),
            MeshPropertyValue::Float(0.4)
        );
        assert_eq!(
            landed(
                Domain::Plain,
                Dimension::Vec2,
                Components::F64(vec![1.0 / 3.0, 1e39])
            ),
            MeshPropertyValue::Floats(vec![1.0 / 3.0, 1e39])
        );
        assert_eq!(
            landed(
                Domain::Plain,
                Dimension::Vec3,
                Components::F64(vec![1.0, 0.5, 0.0])
            ),
            MeshPropertyValue::Floats(vec![1.0, 0.5, 0.0])
        );
        assert_eq!(
            landed(
                Domain::Swatch,
                Dimension::Vec1,
                Components::F64(vec![1.0, 0.0])
            ),
            MeshPropertyValue::Floats(vec![1.0, 0.0])
        );
        assert_eq!(
            landed(
                Domain::Swatch,
                Dimension::Vec2,
                Components::F64(vec![1.0, 0.0, 0.5, 0.5])
            ),
            MeshPropertyValue::FloatRows(vec![vec![1.0, 0.0], vec![0.5, 0.5]])
        );
        assert_eq!(
            landed(Domain::Plain, Dimension::Vec1, Components::Bool(vec![true])),
            MeshPropertyValue::Bool(true)
        );
        assert_eq!(
            landed(
                Domain::Face,
                Dimension::Vec1,
                Components::Bool(vec![true, false])
            ),
            MeshPropertyValue::Bools(vec![true, false])
        );
        assert_eq!(
            landed(
                Domain::Face,
                Dimension::Vec2,
                Components::Bool(vec![true, false, false, true])
            ),
            MeshPropertyValue::BoolRows(vec![vec![true, false], vec![false, true]])
        );
        assert_eq!(
            landed(Domain::Plain, Dimension::Vec1, Components::U32(vec![7])),
            MeshPropertyValue::Int(7)
        );
        assert_eq!(
            landed(Domain::Swatch, Dimension::Vec1, Components::U8(vec![1, 2])),
            MeshPropertyValue::Ints(vec![1, 2])
        );
        assert_eq!(
            landed(
                Domain::Swatch,
                Dimension::Vec2,
                Components::U8(vec![1, 2, 3, 4])
            ),
            MeshPropertyValue::IntRows(vec![vec![1, 2], vec![3, 4]])
        );
        assert_eq!(
            landed(
                Domain::Plain,
                Dimension::Vec1,
                Components::String(vec!["steel".to_owned()])
            ),
            MeshPropertyValue::Text("steel".to_owned())
        );
        assert_eq!(
            landed(
                Domain::Swatch,
                Dimension::Vec1,
                Components::String(vec!["glass".to_owned(), "steel".to_owned()])
            ),
            MeshPropertyValue::Texts(vec!["glass".to_owned(), "steel".to_owned()])
        );
    }

    #[test]
    fn srgb_curves_the_floats_and_errors_on_a_non_float() {
        let grey = Value::new(Domain::Plain, Dimension::Vec1, Components::F64(vec![0.5])).unwrap();
        let MeshPropertyValue::Float(curved) =
            extra_property_value(&element(), &grey, Transfer::Srgb).unwrap()
        else {
            panic!("a plain vec1 lands as a float");
        };
        assert!((curved - 0.7354).abs() < 1e-4);

        let flag =
            Value::new(Domain::Plain, Dimension::Vec1, Components::Bool(vec![true])).unwrap();
        assert!(extra_property_value(&element(), &flag, Transfer::Srgb).is_err());
    }
}
