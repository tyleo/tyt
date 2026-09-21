use crate::{
    Error, Result,
    dependencies::mesh::EncodePng,
    operations::mesh::{
        ArrayDomain, CheckedDestination, ExtraForm, ExtraSource, ExtraWrite, FileForm, Images,
        MeshElement, WriteContext, extra_property_value,
    },
};
use branded_id::U32Id;
use meshdoc::{BMeshUvStream, MeshMain, MeshProperty, MeshPropertyValue, MeshTextureRef};
use std::collections::HashSet;
use vox_value_language::eval_expression;

/// The properties `extras` land as, in write order. `element` names each
/// entry's record element, and `stream_id` gives the stream an image samples
/// through at its bake domain.
pub(crate) fn write_extras<D: EncodePng>(
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

                    let texture_id = images.reference(document, file, file_id)?;

                    MeshPropertyValue::Texture(MeshTextureRef {
                        texture_id,
                        uv_stream_id: stream_id(context.streams.bake(&element)),
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
