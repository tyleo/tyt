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

    let check = |file: &str, flag: &str| -> Result<()> {
        if written(file) {
            return Ok(());
        }

        Err(Error::usage(format!(
            "{flag} references `{file}`, which no --write-file-png-value writes"
        )))
    };

    let check_extras = |extras: &[ExtraWrite], flag: &str| -> Result<()> {
        extras
            .iter()
            .filter(|extra| extra.form == ExtraForm::Image)
            .try_for_each(|extra| match &extra.source {
                ExtraSource::File(file) => check(file, flag),
                ExtraSource::Value(_) => Ok(()),
            })
    };

    for material in record.materials.iter() {
        for slot in &material.slots {
            if let SlotSource::File(file) = &slot.source {
                check(file, "--write-material-slot-file")?;
            }
        }

        check_extras(&material.extras, "--write-material-extra-image-file")?;
    }

    check_extras(&record.mesh_extras, "--write-mesh-extra-image-file")
}
