use crate::{Result, VoxDocumentFile, WriteFile};
use std::path::Path;

/// Writes a document's files under `output`: an empty-path entry to `output`
/// itself, a package entry to `output` joined with its path.
pub fn write_document_files<D: WriteFile>(
    dependencies: &D,
    output: &Path,
    files: &[VoxDocumentFile],
) -> Result<()> {
    for file in files {
        let path = if file.path.is_empty() {
            output.to_path_buf()
        } else {
            output.join(&file.path)
        };

        dependencies.write_file(&path, &file.bytes)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{VoxDocumentFile, WriteFile, write_document_files};
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
            Path::new("out/m.vox"),
            &[VoxDocumentFile::single(b"VOX ".to_vec())],
        )
        .unwrap();

        assert_eq!(
            writes.0.into_inner(),
            vec![(PathBuf::from("out/m.vox"), b"VOX ".to_vec())]
        );
    }

    #[test]
    fn package_paths_join_the_output() {
        let writes = Writes::default();

        write_document_files(
            &writes,
            Path::new("out/p.vmax"),
            &[
                VoxDocumentFile::new("scene.json", b"{}".to_vec()),
                VoxDocumentFile::new("QuickLook/Thumbnail.png", b"png".to_vec()),
            ],
        )
        .unwrap();

        assert_eq!(
            writes.0.into_inner(),
            vec![
                (PathBuf::from("out/p.vmax/scene.json"), b"{}".to_vec()),
                (
                    PathBuf::from("out/p.vmax/QuickLook/Thumbnail.png"),
                    b"png".to_vec()
                ),
            ]
        );
    }
}
