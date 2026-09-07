use crate::{DirectoryEntry, Error, ListDir, ReadFile, ReadFormat, Result, VoxDocumentFile};
use std::path::Path;

/// The files of the document at `input`: one entry with an empty path for a
/// single-file format, or every file of a package, in path order. A package
/// descends one level into its subdirectories so their files keep their
/// prefix. A deeper directory or a non-UTF-8 name is an error.
pub fn read_document_files<D: ReadFile + ListDir>(
    dependencies: &D,
    format: ReadFormat,
    input: &Path,
) -> Result<Vec<VoxDocumentFile>> {
    if !format.is_package() {
        return Ok(vec![VoxDocumentFile::single(
            dependencies.read_file(input)?,
        )]);
    }

    let mut files = Vec::new();

    for entry in sorted_entries(dependencies, input)? {
        let name = entry_name(&entry.path)?;

        if !entry.is_dir {
            files.push(VoxDocumentFile::new(
                name,
                dependencies.read_file(&entry.path)?,
            ));

            continue;
        }

        for child in sorted_entries(dependencies, &entry.path)? {
            if child.is_dir {
                return Err(Error::Files(format!(
                    "`{}` nests a directory deeper than a package holds",
                    child.path.display()
                )));
            }

            files.push(VoxDocumentFile::new(
                format!("{name}/{}", entry_name(&child.path)?),
                dependencies.read_file(&child.path)?,
            ));
        }
    }

    Ok(files)
}

/// The entries of `path`, sorted by path.
fn sorted_entries<D: ListDir>(dependencies: &D, path: &Path) -> Result<Vec<DirectoryEntry>> {
    let mut entries = dependencies.list_dir(path)?;

    entries.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(entries)
}

/// The file name of `path` as UTF-8.
fn entry_name(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .ok_or_else(|| Error::Files(format!("`{}` is not a UTF-8 file name", path.display())))
}

#[cfg(test)]
mod tests {
    use crate::{
        DirectoryEntry, ListDir, ReadFile, ReadFormat, VoxDocumentFile, read_document_files,
    };
    use std::{
        collections::BTreeMap,
        io::{Error as IOError, ErrorKind, Result as IOResult},
        path::{Path, PathBuf},
    };

    /// A filesystem of fixed files, listing a directory as its immediate
    /// children.
    struct Files(BTreeMap<PathBuf, Vec<u8>>);

    impl ReadFile for Files {
        fn read_file(&self, path: &Path) -> IOResult<Vec<u8>> {
            self.0
                .get(path)
                .cloned()
                .ok_or_else(|| IOError::from(ErrorKind::NotFound))
        }
    }

    impl ListDir for Files {
        fn list_dir(&self, path: &Path) -> IOResult<Vec<DirectoryEntry>> {
            let mut entries: Vec<DirectoryEntry> = self
                .0
                .keys()
                .filter_map(|file| {
                    let relative = file.strip_prefix(path).ok()?;

                    let first = relative.components().next()?;

                    Some(DirectoryEntry {
                        path: path.join(first),
                        is_dir: relative.components().count() > 1,
                    })
                })
                .collect();

            entries.dedup();

            Ok(entries)
        }
    }

    #[test]
    fn a_single_file_document_is_one_empty_path() {
        let files = Files(BTreeMap::from([(PathBuf::from("m.vox"), b"VOX ".to_vec())]));

        assert_eq!(
            read_document_files(&files, ReadFormat::MVox, Path::new("m.vox")).unwrap(),
            vec![VoxDocumentFile::single(b"VOX ".to_vec())]
        );
    }

    #[test]
    fn a_package_lists_its_files_one_level_deep_in_path_order() {
        let files = Files(BTreeMap::from([
            (PathBuf::from("p.vmax/scene.json"), b"{}".to_vec()),
            (
                PathBuf::from("p.vmax/QuickLook/Thumbnail.png"),
                b"png".to_vec(),
            ),
            (PathBuf::from("p.vmax/contents0.vmaxb"), b"bplist".to_vec()),
        ]));

        assert_eq!(
            read_document_files(&files, ReadFormat::VMax, Path::new("p.vmax")).unwrap(),
            vec![
                VoxDocumentFile::new("QuickLook/Thumbnail.png", b"png".to_vec()),
                VoxDocumentFile::new("contents0.vmaxb", b"bplist".to_vec()),
                VoxDocumentFile::new("scene.json", b"{}".to_vec()),
            ]
        );
    }

    #[test]
    fn a_deeper_directory_errors() {
        let files = Files(BTreeMap::from([(
            PathBuf::from("p.vmax/QuickLook/deep/file.png"),
            b"png".to_vec(),
        )]));

        assert!(read_document_files(&files, ReadFormat::VMax, Path::new("p.vmax")).is_err());
    }
}
