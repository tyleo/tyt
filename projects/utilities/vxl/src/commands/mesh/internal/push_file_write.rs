use crate::{Error, Result};
use voxsmith::operations::mesh::{FileForm, FileWrite};

/// Pushes `write` onto `files`. A PNG written twice errors, as does a JSON
/// entry named twice in one file or a path holding both forms. JSON entries
/// under different names merge into one file.
pub(crate) fn push_file_write(files: &mut Vec<FileWrite>, write: FileWrite) -> Result<()> {
    for existing in files.iter().filter(|existing| existing.file == write.file) {
        match (&existing.form, &write.form) {
            (FileForm::Json { name: existing }, FileForm::Json { name }) if existing == name => {
                return Err(Error::usage(format!(
                    "--write-file-json-value writes `{name}` into `{}` twice",
                    write.file
                )));
            }

            (FileForm::Json { .. }, FileForm::Json { .. }) => {}

            (FileForm::Png, FileForm::Png) => {
                return Err(Error::usage(format!(
                    "--write-file-png-value writes `{}` twice",
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

        assert!(push_file_write(&mut files, write("v.json", json("a"))).is_ok());
        assert!(push_file_write(&mut files, write("v.json", json("b"))).is_ok());
        assert!(push_file_write(&mut files, write("v.json", json("a"))).is_err());

        assert!(push_file_write(&mut files, write("m.png", FileForm::Png)).is_ok());
        assert!(push_file_write(&mut files, write("m.png", FileForm::Png)).is_err());
        assert!(push_file_write(&mut files, write("m.png", json("a"))).is_err());
        assert!(push_file_write(&mut files, write("v.json", FileForm::Png)).is_err());

        assert_eq!(files.len(), 3);
    }
}
