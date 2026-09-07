use crate::{Dependencies, Result, VoxDocumentFile, WriteFormat, internal};
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Encodes a state as a document's files. The ext `T` moves into the
/// format's ext through its block form. A state carrying another format's
/// ext, or none, writes the format's default ext. The state is consumed
/// because the format's writer needs it in the format's ext type.
pub fn write<D: Dependencies, T: VoxExtBlockCodec>(
    dependencies: &D,
    format: &WriteFormat,
    state: VoxMain<T>,
) -> Result<Vec<VoxDocumentFile>> {
    match format {
        #[cfg(feature = "goxl")]
        WriteFormat::Goxl => internal::write_goxl(dependencies.goxl(), state),
        #[cfg(feature = "mvox")]
        WriteFormat::MVox => internal::write_mvox(dependencies, state),
        #[cfg(feature = "qbcl")]
        WriteFormat::Qb => internal::write_qb(state),
        #[cfg(feature = "qbcl")]
        WriteFormat::Qbt => internal::write_qbt(dependencies.qbcl(), state),
        #[cfg(feature = "qbcl")]
        WriteFormat::Qbcl => internal::write_qbcl(dependencies.qbcl(), state),
        #[cfg(feature = "vmax")]
        WriteFormat::VMax(options) => internal::write_vmax(dependencies.vmax(), state, options),
        #[cfg(feature = "voxj")]
        WriteFormat::Voxj(options) => internal::write_voxj(dependencies.voxj(), &state, *options),
    }
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        DependenciesImpl, ReadFormat, WriteFormat, read, test_state,
        vmax::VMaxWriteOptions,
        voxj::{VoxjSerialization, VoxjWriteOptions},
        write,
    };
    use voxcore::{VoxMain, VoxMap};

    /// Every format a bare state writes, read back through the raw slot.
    #[test]
    fn each_format_round_trips_a_bare_state() {
        let formats = [
            WriteFormat::Goxl,
            WriteFormat::MVox,
            WriteFormat::Qbcl,
            WriteFormat::VMax(VMaxWriteOptions::default()),
            WriteFormat::Voxj(VoxjWriteOptions::default()),
        ];

        for format in formats {
            let files = write(&DependenciesImpl, &format, test_state(())).unwrap();

            let loaded: VoxMain<Option<VoxMap>> =
                read(&DependenciesImpl, format.read_format(), &files).unwrap();

            assert_eq!(loaded.object_count(), 1, "{format:?}");
        }
    }

    /// A single-file format writes one empty-path entry. The package writes
    /// its scene file among others.
    #[test]
    fn documents_take_their_file_shape() {
        let single = write(&DependenciesImpl, &WriteFormat::MVox, test_state(())).unwrap();

        assert_eq!(single.len(), 1);

        assert!(single[0].path.is_empty());

        let package = write(
            &DependenciesImpl,
            &WriteFormat::VMax(VMaxWriteOptions::default()),
            test_state(()),
        )
        .unwrap();

        assert!(package.iter().any(|file| file.path == "scene.json"));
    }

    /// A format's ext survives its own round trip as a keyed block, and a
    /// foreign block writes another format's default ext.
    #[test]
    fn exts_ride_the_raw_ext_between_formats() {
        let vmax = WriteFormat::VMax(VMaxWriteOptions::default());

        let files = write(&DependenciesImpl, &vmax, test_state(())).unwrap();

        let loaded: VoxMain<Option<VoxMap>> =
            read(&DependenciesImpl, ReadFormat::VMax, &files).unwrap();

        let block = loaded.ext().clone().expect("a vmax read stashes its ext");

        assert!(block.0.iter().any(|(key, _)| key == "vmax"));

        let again = write(&DependenciesImpl, &vmax, loaded).unwrap();

        assert_eq!(again.len(), files.len());

        let reloaded: VoxMain<Option<VoxMap>> =
            read(&DependenciesImpl, ReadFormat::VMax, &files).unwrap();

        let mvox = write(&DependenciesImpl, &WriteFormat::MVox, reloaded).unwrap();

        let loaded: VoxMain<Option<VoxMap>> =
            read(&DependenciesImpl, ReadFormat::MVox, &mvox).unwrap();

        assert_eq!(loaded.object_count(), 1);
    }

    /// The `()` ext drops every block on the way in.
    #[test]
    fn the_unit_ext_drops_the_block() {
        let files = write(&DependenciesImpl, &WriteFormat::Goxl, test_state(())).unwrap();

        let loaded: VoxMain<()> = read(&DependenciesImpl, ReadFormat::Goxl, &files).unwrap();

        assert_eq!(loaded.object_count(), 1);
    }

    /// Each Voxel Json serialization writes its container form.
    #[test]
    fn voxj_serializations_take_their_form() {
        let bytes = |serialization| {
            let options = VoxjWriteOptions {
                serialization,
                ..Default::default()
            };

            write(
                &DependenciesImpl,
                &WriteFormat::Voxj(options),
                test_state(()),
            )
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
        let mut files = write(&DependenciesImpl, &WriteFormat::MVox, test_state(())).unwrap();

        files.push(files[0].clone());

        assert!(read::<_, ()>(&DependenciesImpl, ReadFormat::MVox, &files).is_err());
    }
}
