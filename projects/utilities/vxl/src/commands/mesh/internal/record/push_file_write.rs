use crate::{Error, Result};
use voxsmith::operations::mesh::{FileForm, FileWrite};

/// Pushes `write`, which `origin` holds, onto `files`. A PNG written twice
/// errors, as does a JSON entry named twice in one file or a path holding
/// both forms. JSON entries under different names merge into one file.
pub(crate) fn push_file_write(
    files: &mut Vec<FileWrite>,
    write: FileWrite,
    origin: &str,
) -> Result<()> {
    for existing in files.iter().filter(|existing| existing.file == write.file) {
        match (&existing.form, &write.form) {
            (FileForm::Json { name: existing }, FileForm::Json { name }) if existing == name => {
                return Err(Error::usage(format!(
                    "{origin} writes `{name}` into `{}` twice",
                    write.file
                )));
            }

            (FileForm::Json { .. }, FileForm::Json { .. }) => {}

            (FileForm::Png, FileForm::Png) => {
                return Err(Error::usage(format!(
                    "{origin} writes `{}` twice",
                    write.file
                )));
            }

            (FileForm::Json { .. }, FileForm::Png) | (FileForm::Png, FileForm::Json { .. }) => {
                return Err(Error::usage(format!(
                    "`{}` is written as both a PNG and a JSON file",
                    write.file
                )));
            }
        }
    }

    files.push(write);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::commands::push_file_write;
    use voxsmith::operations::mesh::{FileForm, FileWrite, Transfer, WrittenValue};

    /// A write of `x` to `file` in `form`.
    fn write(file: &str, form: FileForm) -> FileWrite {
        FileWrite {
            file: file.to_owned(),
            value: WrittenValue {
                expression: "x".to_owned(),
                transfer: Transfer::Linear,
            },
            form,
        }
    }

    /// A JSON entry named `name`.
    fn json(name: &str) -> FileForm {
        FileForm::Json {
            name: name.to_owned(),
        }
    }

    #[test]
    fn json_entries_merge_by_name_and_pngs_never_repeat() {
        let mut files = Vec::new();
        let origin = "--write-file-json-value";

        assert!(push_file_write(&mut files, write("v.json", json("a")), origin).is_ok());
        assert!(push_file_write(&mut files, write("v.json", json("b")), origin).is_ok());
        assert!(push_file_write(&mut files, write("v.json", json("a")), origin).is_err());

        let origin = "--write-file-png-value";

        assert!(push_file_write(&mut files, write("m.png", FileForm::Png), origin).is_ok());
        assert!(push_file_write(&mut files, write("m.png", FileForm::Png), origin).is_err());
        assert!(push_file_write(&mut files, write("m.png", json("a")), origin).is_err());
        assert!(push_file_write(&mut files, write("v.json", FileForm::Png), origin).is_err());

        assert_eq!(files.len(), 3);
    }
}
