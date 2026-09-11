use crate::{
    Dependencies, InstalledFormat, Result, VoxDocumentFile, WriteFormat, WriteFormatVisitor,
    ext::VoxconvVoxMain,
};

/// Encodes a state as a document's files. A box holding the format's ext
/// writes the loaded file back exactly. Any other box encodes to its `ext`
/// block, and the format takes its slot from that block. A block with no
/// slot for the format writes a file synthesized from the scene. The state
/// is consumed because the format's writer needs it in the format's ext
/// type.
pub fn write_with_ext<D: Dependencies>(
    dependencies: &D,
    format: &WriteFormat,
    state: VoxconvVoxMain,
) -> Result<Vec<VoxDocumentFile>> {
    format.with(WriteWithExt {
        dependencies,
        state,
    })
}

/// The typed write of one format.
struct WriteWithExt<'a, D> {
    dependencies: &'a D,
    state: VoxconvVoxMain,
}

impl<D: Dependencies> WriteFormatVisitor for WriteWithExt<'_, D> {
    type Output = Result<Vec<VoxDocumentFile>>;

    fn visit<F: InstalledFormat>(self, options: &F::WriteOptions) -> Self::Output {
        F::write_with_ext(self.dependencies, options, self.state)
    }
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        DependenciesImpl, ReadFormat, VoxDocumentFile, WriteFormat,
        ext::{VoxconvExt, read_with_ext, write_with_ext},
        test_state,
        vmax::VMaxWriteOptions,
        voxj::ext::{CompositeVoxExt, InertVoxExt, voxj_vox_ext_from_ext},
        write,
    };
    use vmax_voxcore::ext::VMaxExt;
    use voxcore::{VoxMapEntry, VoxValue};

    /// A Voxel Max package written from a bare state.
    fn vmax_package() -> Vec<VoxDocumentFile> {
        write(
            &DependenciesImpl,
            &WriteFormat::VMax(VMaxWriteOptions::default()),
            &test_state(()),
        )
        .unwrap()
    }

    /// A read boxes the format's ext, and a same-format write takes it back
    /// through the downcast.
    #[test]
    fn a_format_writes_its_own_ext_back() {
        let files = vmax_package();

        let loaded = read_with_ext(&DependenciesImpl, ReadFormat::VMax, &files).unwrap();

        assert!(loaded.ext().is::<VMaxExt>());

        let again = write_with_ext(
            &DependenciesImpl,
            &WriteFormat::VMax(VMaxWriteOptions::default()),
            loaded,
        )
        .unwrap();

        assert_eq!(again, files);
    }

    /// An ext rides through a Voxel Json document: the block decodes into the
    /// composite, and the format's write takes its entry back.
    #[test]
    fn an_ext_rides_through_voxel_json() {
        let files = vmax_package();

        let loaded = read_with_ext(&DependenciesImpl, ReadFormat::VMax, &files).unwrap();

        let voxj = write_with_ext(
            &DependenciesImpl,
            &WriteFormat::from(ReadFormat::Voxj),
            loaded,
        )
        .unwrap();

        let loaded = read_with_ext(&DependenciesImpl, ReadFormat::Voxj, &voxj).unwrap();

        assert!(loaded.ext().is::<CompositeVoxExt>());

        let block = voxj_vox_ext_from_ext(loaded.ext().as_ref()).unwrap();

        assert!(block.slot("vmax").is_some());

        let again = write_with_ext(
            &DependenciesImpl,
            &WriteFormat::VMax(VMaxWriteOptions::default()),
            loaded,
        )
        .unwrap();

        assert_eq!(again, files);
    }

    /// A box holding another format's ext writes a synthesized file.
    #[test]
    fn a_foreign_ext_writes_a_synthesized_file() {
        let files = vmax_package();

        let loaded = read_with_ext(&DependenciesImpl, ReadFormat::VMax, &files).unwrap();

        let mvox = write_with_ext(&DependenciesImpl, &WriteFormat::MVox, loaded).unwrap();

        let loaded = read_with_ext(&DependenciesImpl, ReadFormat::MVox, &mvox).unwrap();

        assert_eq!(loaded.object_count(), 1);
    }

    /// An entry no enabled format owns rides through a Voxel Json document
    /// as it was, inert in the composite.
    #[test]
    fn an_inert_entry_rides_through_voxel_json() {
        let inert = InertVoxExt {
            key: "other".to_owned(),
            value: VoxValue::Bool(true),
        };

        let state = test_state(Box::new(inert.clone()) as Box<dyn VoxconvExt>);

        let files = write_with_ext(
            &DependenciesImpl,
            &WriteFormat::from(ReadFormat::Voxj),
            state,
        )
        .unwrap();

        let loaded = read_with_ext(&DependenciesImpl, ReadFormat::Voxj, &files).unwrap();

        let composite = loaded
            .ext()
            .downcast_ref::<CompositeVoxExt>()
            .expect("a Voxel Json read carries the composite");

        assert_eq!(
            composite.exts[0].downcast_ref::<InertVoxExt>(),
            Some(&inert)
        );

        assert_eq!(
            voxj_vox_ext_from_ext(loaded.ext().as_ref())
                .unwrap()
                .into_slots(),
            vec![VoxMapEntry {
                key: "other".to_owned(),
                value: VoxValue::Bool(true),
            }]
        );
    }
}
