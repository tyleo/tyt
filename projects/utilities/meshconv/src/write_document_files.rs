use crate::{MeshDocumentFile, Result, WriteFile};
use std::path::Path;

/// Writes a document's files. The empty-path entry lands at `output`. Every
/// other entry lands beside `output`, or under it when `package` is set.
pub fn write_document_files<D: WriteFile>(
    dependencies: &D,
    output: &Path,
    package: bool,
    files: &[MeshDocumentFile],
) -> Result<()> {
    let directory = output.parent().unwrap_or_else(|| Path::new(""));

    for file in files {
        let path = if file.path.is_empty() {
            output.to_path_buf()
        } else if package {
            output.join(&file.path)
        } else {
            directory.join(&file.path)
        };

        dependencies.write_file(&path, &file.bytes)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{MeshDocumentFile, WriteFile, write_document_files};
    use std::{
        cell::RefCell,
        io::Result as IOResult,
        path::{Path, PathBuf},
    };

    /// Records each write's path and bytes.
    #[derive(Default)]
    struct Writes(RefCell<Vec<(PathBuf, Vec<u8>)>>);

    impl WriteFile for Writes {
        fn write_file(&self, path: &Path, bytes: &[u8]) -> IOResult<()> {
            self.0
                .borrow_mut()
                .push((path.to_path_buf(), bytes.to_vec()));

            Ok(())
        }
    }

    #[test]
    fn an_empty_path_writes_the_output_itself() {
        let writes = Writes::default();

        write_document_files(
            &writes,
            Path::new("out/m.glb"),
            false,
            &[MeshDocumentFile::primary(b"glTF".to_vec())],
        )
        .unwrap();

        assert_eq!(
            writes.0.into_inner(),
            vec![(PathBuf::from("out/m.glb"), b"glTF".to_vec())]
        );
    }

    #[test]
    fn loose_files_land_beside_the_output_and_package_files_under_it() {
        let files = [
            MeshDocumentFile::primary(b"{}".to_vec()),
            MeshDocumentFile::new("skin.png", b"png".to_vec()),
        ];

        let writes = Writes::default();

        write_document_files(&writes, Path::new("out/m.gltf"), false, &files).unwrap();

        assert_eq!(
            writes.0.into_inner(),
            vec![
                (PathBuf::from("out/m.gltf"), b"{}".to_vec()),
                (PathBuf::from("out/skin.png"), b"png".to_vec()),
            ]
        );

        let writes = Writes::default();

        write_document_files(&writes, Path::new("out/p.pkg"), true, &files).unwrap();

        assert_eq!(
            writes.0.into_inner(),
            vec![
                (PathBuf::from("out/p.pkg"), b"{}".to_vec()),
                (PathBuf::from("out/p.pkg/skin.png"), b"png".to_vec()),
            ]
        );
    }
}
