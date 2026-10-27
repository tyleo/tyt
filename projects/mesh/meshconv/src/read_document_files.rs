use crate::{DirectoryEntry, Error, ListDir, MeshDocumentFile, ReadFile, ReadFormat, Result};
use std::path::Path;

/// The files of the document at `input`: the primary, then the loose files
/// it references in reference order. A package lists every file of the
/// directory in path order, descending one level into subdirectories. A
/// deeper directory or a non-UTF-8 name is an error.
pub fn read_document_files<D: ReadFile + ListDir>(
    dependencies: &D,
    format: ReadFormat,
    input: &Path,
) -> Result<Vec<MeshDocumentFile>> {
    if !format.is_package() {
        let primary = dependencies.read_file(input)?;

        let directory = input.parent().unwrap_or_else(|| Path::new(""));

        let mut files = Vec::new();

        for path in format.loose_paths(&primary)? {
            files.push(MeshDocumentFile::new(
                path.clone(),
                dependencies.read_file(&directory.join(&path))?,
            ));
        }

        files.insert(0, MeshDocumentFile::primary(primary));

        return Ok(files);
    }

    let mut files = Vec::new();

    for entry in sorted_entries(dependencies, input)? {
        let name = entry_name(&entry.path)?;

        if !entry.is_dir {
            files.push(MeshDocumentFile::new(
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

            files.push(MeshDocumentFile::new(
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

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        MemoryFiles, MeshDocumentFile, ReadFormat, WriteFormat,
        gltf::{GltfContainer, GltfImageStorage, GltfWriteFormat, GltfWriteOptions},
        read_document_files, test_main, write,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn a_single_file_document_is_one_empty_path() {
        let memory = MemoryFiles::default();

        let files = write(
            &DependenciesImplForTests,
            &WriteFormat::from(ReadFormat::Gltf),
            test_main(()),
        )
        .unwrap();

        memory
            .0
            .borrow_mut()
            .insert(PathBuf::from("m.glb"), files[0].bytes.clone());

        assert_eq!(
            read_document_files(&memory, ReadFormat::Gltf, Path::new("m.glb")).unwrap(),
            vec![MeshDocumentFile::primary(files[0].bytes.clone())]
        );
    }

    #[test]
    fn a_primary_with_loose_files_reads_them_beside_it() {
        let memory = MemoryFiles::default();

        let files = write(
            &DependenciesImplForTests,
            &WriteFormat::Gltf(GltfWriteFormat {
                container: GltfContainer::Gltf,
                options: GltfWriteOptions {
                    images: GltfImageStorage::Loose,
                },
            }),
            test_main(()),
        )
        .unwrap();

        {
            let mut memory = memory.0.borrow_mut();
            memory.insert(PathBuf::from("out/m.gltf"), files[0].bytes.clone());
            memory.insert(PathBuf::from("out/atlas.png"), files[1].bytes.clone());
        }

        assert_eq!(
            read_document_files(&memory, ReadFormat::Gltf, Path::new("out/m.gltf")).unwrap(),
            files
        );

        // A missing loose file is an error.
        memory.0.borrow_mut().remove(Path::new("out/atlas.png"));

        assert!(read_document_files(&memory, ReadFormat::Gltf, Path::new("out/m.gltf")).is_err());
    }

    use crate::DependenciesImpl as DependenciesImplForTests;
}
