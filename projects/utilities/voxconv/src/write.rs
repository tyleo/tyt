use crate::{
    Dependencies, InstalledFormat, Result, VoxDocumentFile, WriteFormat, WriteFormatVisitor,
};
use voxcore::VoxMain;

/// Encodes a bare [`VoxMain`] as a document's files synthesized from its
/// scene. The `ext` feature's `ext::write_with_ext` writes from the format's
/// ext instead when the state carries it.
pub fn write<D: Dependencies>(
    dependencies: &D,
    format: &WriteFormat,
    main: VoxMain<()>,
) -> Result<Vec<VoxDocumentFile>> {
    format.with(Write { dependencies, main })
}

/// The bare write of one format.
struct Write<'a, D> {
    dependencies: &'a D,
    main: VoxMain<()>,
}

impl<D: Dependencies> WriteFormatVisitor for Write<'_, D> {
    type Output = Result<Vec<VoxDocumentFile>>;

    fn visit<F: InstalledFormat>(self, options: &F::WriteOptions) -> Self::Output {
        F::write(self.dependencies, options, self.main)
    }
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        DependenciesImpl, ReadFormat, WriteFormat, read, test_main,
        vmax::VMaxWriteOptions,
        voxj::{VoxjSerialization, VoxjWriteFormat, VoxjWriteOptions},
        write,
    };

    /// Every format a bare state writes reads back bare.
    #[test]
    fn each_format_round_trips_a_bare_state() {
        let formats = [
            WriteFormat::Goxl,
            WriteFormat::MVox,
            WriteFormat::Qb,
            WriteFormat::Qbt,
            WriteFormat::Qbcl,
            WriteFormat::VMax(VMaxWriteOptions::default()),
            WriteFormat::from(ReadFormat::Voxj),
        ];

        for format in formats {
            let files = write(&DependenciesImpl, &format, test_main(())).unwrap();

            let loaded = read(&DependenciesImpl, format.read_format(), &files).unwrap();

            assert_eq!(loaded.object_count(), 1, "{format:?}");
        }
    }

    /// A single-file format writes one empty-path entry. The package writes
    /// its scene file among others.
    #[test]
    fn documents_take_their_file_shape() {
        let single = write(&DependenciesImpl, &WriteFormat::MVox, test_main(())).unwrap();

        assert_eq!(single.len(), 1);

        assert!(single[0].path.is_empty());

        let package = write(
            &DependenciesImpl,
            &WriteFormat::VMax(VMaxWriteOptions::default()),
            test_main(()),
        )
        .unwrap();

        assert!(package.iter().any(|file| file.path == "scene.json"));
    }

    /// Each Voxel Json serialization writes its container form.
    #[test]
    fn voxj_serializations_take_their_form() {
        let bytes = |serialization| {
            let format = WriteFormat::Voxj(VoxjWriteFormat {
                serialization,
                options: VoxjWriteOptions::default(),
            });

            write(&DependenciesImpl, &format, test_main(()))
                .unwrap()
                .remove(0)
                .bytes
        };

        assert!(bytes(VoxjSerialization::Compact).starts_with(b"{\""));

        assert!(bytes(VoxjSerialization::Pretty).starts_with(b"{\n"));

        assert!(bytes(VoxjSerialization::Zip).starts_with(b"PK"));
    }

    /// A single-file format given two files is an error.
    #[test]
    fn two_files_for_one_format_error() {
        let mut files = write(&DependenciesImpl, &WriteFormat::MVox, test_main(())).unwrap();

        files.push(files[0].clone());

        assert!(read(&DependenciesImpl, ReadFormat::MVox, &files).is_err());
    }
}
