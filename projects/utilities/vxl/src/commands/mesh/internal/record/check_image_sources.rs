use crate::{Error, Result};
use voxsmith::operations::mesh::{
    ExtraForm, ExtraSource, ExtraWrite, FileForm, MeshRecord, SlotSource,
};

/// Errors unless every image reference in `record`, a slot or an image
/// extra sourced from a file, points at a PNG the run writes.
pub(crate) fn check_image_sources(record: &MeshRecord) -> Result<()> {
    let written = |file: &str| {
        record
            .files
            .iter()
            .any(|write| write.form == FileForm::Png && write.file == file)
    };

    let check = |file: &str, reference: String| -> Result<()> {
        if written(file) {
            return Ok(());
        }

        Err(Error::usage(format!(
            "{reference} references `{file}`, which nothing writes as a PNG"
        )))
    };

    let check_extras = |extras: &[ExtraWrite], owner: &str| -> Result<()> {
        extras
            .iter()
            .filter(|extra| extra.form == ExtraForm::Image)
            .try_for_each(|extra| match &extra.source {
                ExtraSource::File(file) => check(file, format!("{owner} extra `{}`", extra.name)),
                ExtraSource::Value(_) => Ok(()),
            })
    };

    for (index, material) in (0..).zip(record.materials.iter()) {
        for slot in &material.slots {
            if let SlotSource::File(file) = &slot.source {
                check(file, format!("material {index}'s slot `{}`", slot.property))?;
            }
        }

        check_extras(&material.extras, &format!("material {index}'s"))?;
    }

    check_extras(&record.mesh_extras, "the mesh")
}
