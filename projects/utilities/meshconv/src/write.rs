use crate::{
    Dependencies, InstalledFormat, MeshDocumentFile, Result, WriteFormat, WriteFormatVisitor,
};
use meshdoc::MeshMain;

/// Encodes a bare [`MeshMain`] as a document's files, synthesizing the
/// format's ext. The `ext` feature's `ext::write_with_ext` writes from a
/// loaded ext instead.
pub fn write<D: Dependencies>(
    dependencies: &D,
    format: &WriteFormat,
    main: MeshMain<()>,
) -> Result<Vec<MeshDocumentFile>> {
    format.with(Write { dependencies, main })
}

/// The bare write of one format.
struct Write<'a, D> {
    dependencies: &'a D,
    main: MeshMain<()>,
}

impl<D: Dependencies> WriteFormatVisitor for Write<'_, D> {
    type Output = Result<Vec<MeshDocumentFile>>;

    fn visit<F: InstalledFormat>(self, options: &F::WriteOptions) -> Self::Output {
        F::write(self.dependencies, options, self.main)
    }
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        DependenciesImpl, ReadFormat, WriteFormat,
        gltf::{GltfContainer, GltfImageStorage, GltfWriteFormat, GltfWriteOptions},
        read, test_main, write,
    };

    #[test]
    fn each_container_round_trips_a_bare_state() {
        for container in [GltfContainer::Glb, GltfContainer::Gltf] {
            let format = WriteFormat::Gltf(GltfWriteFormat {
                container,
                options: GltfWriteOptions::default(),
            });

            let files = write(&DependenciesImpl, &format, test_main(())).unwrap();

            assert_eq!(files.len(), 1);

            assert!(files[0].path.is_empty());

            let loaded = read(&DependenciesImpl, format.read_format(), &files).unwrap();

            assert_eq!(loaded.object_count(), 1, "{container:?}");
        }
    }

    #[test]
    fn loose_images_take_their_own_files() {
        let format = WriteFormat::Gltf(GltfWriteFormat {
            container: GltfContainer::Gltf,
            options: GltfWriteOptions {
                images: GltfImageStorage::Loose,
            },
        });

        let files = write(&DependenciesImpl, &format, test_main(())).unwrap();

        let paths: Vec<&str> = files.iter().map(|file| file.path.as_str()).collect();

        assert_eq!(paths, ["", "atlas.png"]);

        let loaded = read(&DependenciesImpl, ReadFormat::Gltf, &files).unwrap();

        assert_eq!(loaded.image_count(), 1);

        // Without the loose file the primary cannot resolve its image.
        assert!(read(&DependenciesImpl, ReadFormat::Gltf, &files[..1]).is_err());
    }

    #[test]
    fn a_document_without_a_primary_errors() {
        let format = WriteFormat::from(ReadFormat::Gltf);

        let mut files = write(&DependenciesImpl, &format, test_main(())).unwrap();

        files[0].path = "moved.glb".to_owned();

        assert!(read(&DependenciesImpl, ReadFormat::Gltf, &files).is_err());
    }
}
