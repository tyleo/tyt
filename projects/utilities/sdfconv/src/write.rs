use crate::{Dependencies, Format, Result, WriteFormat, WriteFormatVisitor};
use sdfcore::SdfMain;

/// Encodes an [`SdfMain`] as a document's bytes.
pub fn write<D: Dependencies>(
    dependencies: &D,
    format: &WriteFormat,
    main: &SdfMain,
) -> Result<Vec<u8>> {
    format.with(Write { dependencies, main })
}

/// The write of one format.
struct Write<'a, D> {
    dependencies: &'a D,

    main: &'a SdfMain,
}

impl<D: Dependencies> WriteFormatVisitor for Write<'_, D> {
    type Output = Result<Vec<u8>>;

    fn visit<F: Format>(self, options: &F::WriteOptions) -> Self::Output {
        F::write(self.dependencies, options, self.main)
    }
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{DependenciesImpl, WriteFormat, read, sdfj::SdfjSerialization, test_main, write};

    #[test]
    fn each_serialization_round_trips() {
        for serialization in [
            SdfjSerialization::Compact,
            SdfjSerialization::Pretty,
            SdfjSerialization::Zip,
        ] {
            let format = WriteFormat::Sdfj(serialization);

            let bytes = write(&DependenciesImpl, &format, &test_main()).unwrap();

            let loaded = read(&DependenciesImpl, format.read_format(), &bytes).unwrap();

            assert_eq!(loaded, test_main(), "{serialization:?}");
        }
    }

    #[test]
    fn undecodable_bytes_error() {
        let format = WriteFormat::Sdfj(SdfjSerialization::Compact);

        assert!(read(&DependenciesImpl, format.read_format(), b"not a document").is_err());
    }
}
