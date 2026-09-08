use crate::{Dependencies, Result, VoxDocumentFile, WriteFormat, internal};
use voxcore::{VoxMain, ext::VoxExt};

/// Encodes a state as a document's files. A box holding the format's ext
/// writes the loaded file back exactly. Any other box encodes to its block.
/// The format takes its entry from that block. A block with no entry for the
/// format writes a file synthesized from the scene. The
/// state is consumed because the format's writer needs it in the format's
/// ext type.
pub fn write_with_ext<D: Dependencies>(
    dependencies: &D,
    format: &WriteFormat,
    state: VoxMain<Box<dyn VoxExt>>,
) -> Result<Vec<VoxDocumentFile>> {
    match format {
        #[cfg(feature = "goxl")]
        WriteFormat::Goxl => internal::write_goxl_with_ext(dependencies.goxl(), state),
        #[cfg(feature = "mvox")]
        WriteFormat::MVox => internal::write_mvox_with_ext(dependencies, state),
        #[cfg(feature = "qbcl")]
        WriteFormat::Qb => internal::write_qb_with_ext(state),
        #[cfg(feature = "qbcl")]
        WriteFormat::Qbt => internal::write_qbt_with_ext(dependencies.qbcl(), state),
        #[cfg(feature = "qbcl")]
        WriteFormat::Qbcl => internal::write_qbcl_with_ext(dependencies.qbcl(), state),
        #[cfg(feature = "vmax")]
        WriteFormat::VMax(options) => {
            internal::write_vmax_with_ext(dependencies.vmax(), state, options)
        }
        #[cfg(feature = "voxj")]
        WriteFormat::Voxj {
            serialization,
            options,
        } => internal::write_voxj(dependencies.voxj(), &state, *serialization, options),
    }
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        DependenciesImpl, ReadFormat, VoxDocumentFile, WriteFormat,
        ext::{read_with_ext, write_with_ext},
        test_state,
        vmax::VMaxWriteOptions,
        write,
    };
    use vmax_voxcore::ext::VMaxExt;
    use voxcore::{
        VoxMap, VoxValue,
        ext::{CompositeVoxExt, VoxExt},
    };

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

        assert!(loaded.ext().as_any().is::<VMaxExt>());

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

        assert!(loaded.ext().as_any().is::<CompositeVoxExt>());

        let block = loaded.ext().to_vox_ext().unwrap();

        assert!(block.0.iter().any(|(key, _)| key == "vmax"));

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

    /// A block no enabled format owns rides through a Voxel Json document as
    /// it was.
    #[test]
    fn a_verbatim_block_rides_through_voxel_json() {
        let block = VoxMap(vec![("other".to_owned(), VoxValue::Bool(true))]);

        let state = test_state(Box::new(block.clone()) as Box<dyn VoxExt>);

        let files = write_with_ext(
            &DependenciesImpl,
            &WriteFormat::from(ReadFormat::Voxj),
            state,
        )
        .unwrap();

        let loaded = read_with_ext(&DependenciesImpl, ReadFormat::Voxj, &files).unwrap();

        assert_eq!(loaded.ext().to_vox_ext().unwrap(), block);
    }
}
