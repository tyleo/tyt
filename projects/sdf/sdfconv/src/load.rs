use crate::{
    Dependencies, Format, LoadOptions, ReadFile, ReadFormat, ReadFormatVisitor, Result, read,
};
use sdfcore::SdfMain;
use std::path::Path;

/// Reads the document at `input` into an [`SdfMain`]: the file's bytes, then
/// [`read()`], or the format's read with fonts when `options.fonts` is set.
pub fn load<D: Dependencies + ReadFile>(
    dependencies: &D,
    format: ReadFormat,
    input: &Path,
    options: LoadOptions,
) -> Result<SdfMain> {
    let bytes = dependencies.read_file(input)?;
    if !options.fonts {
        return read(dependencies, format, &bytes);
    }

    format.with(ReadWithFonts {
        dependencies,
        bytes: &bytes,
        directory: input.parent().expect("an input path names a file"),
    })
}

/// The read of one format with its fonts.
struct ReadWithFonts<'a, D> {
    dependencies: &'a D,

    bytes: &'a [u8],

    directory: &'a Path,
}

impl<D: Dependencies> ReadFormatVisitor for ReadWithFonts<'_, D> {
    type Output = Result<SdfMain>;

    fn visit<F: Format>(self) -> Self::Output {
        F::read_with_fonts(self.dependencies, self.bytes, self.directory)
    }
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{LoadOptions, MemoryFiles, ReadFormat, WriteFormat, load, save, test_main};
    use std::path::Path;

    #[test]
    fn the_fonts_load_only_when_asked() {
        let memory = MemoryFiles::default();
        let input = Path::new("models/m.sdfj");
        save(
            &memory,
            &WriteFormat::from(ReadFormat::Sdfj),
            &test_main(),
            input,
        )
        .unwrap();

        let bare = load(&memory, ReadFormat::Sdfj, input, LoadOptions::default()).unwrap();
        assert!(bare.font_data().is_none());

        let loaded = load(
            &memory,
            ReadFormat::Sdfj,
            input,
            LoadOptions { fonts: true },
        )
        .unwrap();
        assert!(loaded.font_data().unwrap().is_empty());
    }
}
