use crate::{CreateTempDir, ReadFile, Result, RunProgram, WriteFile};
use sdfconv::sdfj::SdfjSerialization;
use sdfj_builder::{JavaScriptRuntime, SDFJ_BUILDER_FILES, SDFJ_BUILDER_LIBRARIES_PATH};
use sdfj_codec::{DependenciesImpl as SdfjCodecDependencies, wrap_sdfjz};
use std::{
    io::{Error as IOError, ErrorKind},
    path::Path,
};

/// Builds `model` under `runtime` into the document at `output`, zipped when
/// `output` takes the `.sdfjz` extension. The builder reads its libraries from
/// the JSON `libraries`. A failed run leaves `output` untouched.
pub fn run_sdfj_builder(
    dependencies: &(impl CreateTempDir + ReadFile + RunProgram + WriteFile),
    runtime: JavaScriptRuntime,
    model: &Path,
    output: &Path,
    libraries: &[u8],
) -> Result<()> {
    let directory = dependencies.create_temp_dir()?;
    let directory = directory.path();
    for file in SDFJ_BUILDER_FILES {
        dependencies.write_file(&directory.join(file.path), file.text.as_bytes())?;
    }
    dependencies.write_file(&directory.join(SDFJ_BUILDER_LIBRARIES_PATH), libraries)?;

    let program = runtime.program();
    let document = directory.join("document.sdfj");
    let code = dependencies
        .run_program(program, &runtime.args(directory, model, &document))
        .map_err(|error| match error.kind() {
            ErrorKind::NotFound => IOError::new(
                ErrorKind::NotFound,
                format!("the runtime `{program}` is not on the PATH"),
            ),

            _ => error,
        })?;

    let model = model.display();
    match code {
        Some(0) => {}

        Some(code) => {
            return Err(IOError::other(format!(
                "building `{model}` under `{program}` failed with exit code {code}"
            ))
            .into());
        }

        None => {
            return Err(IOError::other(format!(
                "building `{model}` under `{program}` ended on a signal"
            ))
            .into());
        }
    }

    let mut bytes = dependencies.read_file(&document)?;
    let extension = output.extension().and_then(|extension| extension.to_str());
    if extension.and_then(SdfjSerialization::from_extension) == Some(SdfjSerialization::Zip) {
        bytes = wrap_sdfjz(&SdfjCodecDependencies, &bytes);
    }

    Ok(dependencies.write_file(output, &bytes)?)
}

#[cfg(test)]
mod tests {
    use crate::{
        CreateTempDir, DependenciesImpl, ReadFile, RunProgram, WriteFile,
        commands::run_sdfj_builder,
    };
    use sdfj_builder::{JavaScriptRuntime, SDFJ_BUILDER_FILES, SDFJ_BUILDER_LIBRARIES_PATH};
    use sdfj_codec::{DependenciesImpl as SdfjCodecDependencies, unwrap_sdfjz};
    use std::{
        cell::RefCell,
        ffi::OsString,
        fs,
        io::{Error as IOError, ErrorKind, Result as IOResult},
        path::{Path, PathBuf},
    };
    use tempfile::TempDir;

    const DOCUMENT: &str = "{\"version\":1}\n";

    const LIBRARIES: &str = "[{\"name\":\"materials\",\"document\":{}}]";

    /// Real files and a stand-in for `node`.
    struct StandIn {
        exit: fn() -> IOResult<Option<i32>>,

        /// The builder's directory during the last run.
        directory: RefCell<Option<PathBuf>>,
    }

    impl StandIn {
        fn new(exit: fn() -> IOResult<Option<i32>>) -> Self {
            StandIn {
                exit,
                directory: RefCell::new(None),
            }
        }
    }

    impl CreateTempDir for StandIn {
        fn create_temp_dir(&self) -> IOResult<TempDir> {
            DependenciesImpl.create_temp_dir()
        }
    }

    impl ReadFile for StandIn {
        fn read_file(&self, path: &Path) -> IOResult<Vec<u8>> {
            ReadFile::read_file(&DependenciesImpl, path)
        }
    }

    impl WriteFile for StandIn {
        fn write_file(&self, path: &Path, bytes: &[u8]) -> IOResult<()> {
            WriteFile::write_file(&DependenciesImpl, path, bytes)
        }
    }

    impl RunProgram for StandIn {
        fn run_program(&self, program: &str, args: &[OsString]) -> IOResult<Option<i32>> {
            assert_eq!(program, "node");
            let [main, model, document] = args else {
                panic!("{args:?}");
            };
            assert_eq!(model, "chair.ts");

            let directory = Path::new(main).parent().unwrap().parent().unwrap();
            for file in SDFJ_BUILDER_FILES {
                let text = fs::read_to_string(directory.join(file.path)).unwrap();
                assert_eq!(text, file.text, "{}", file.path);
            }
            let libraries = fs::read_to_string(directory.join(SDFJ_BUILDER_LIBRARIES_PATH));
            assert_eq!(libraries.unwrap(), LIBRARIES);

            fs::write(document, DOCUMENT).unwrap();
            *self.directory.borrow_mut() = Some(directory.to_owned());
            (self.exit)()
        }
    }

    fn build_chair(stand_in: &StandIn, out: &TempDir) -> Option<String> {
        build_chair_to(stand_in, &out.path().join("models/chair.sdfj"))
    }

    fn build_chair_to(stand_in: &StandIn, output: &Path) -> Option<String> {
        run_sdfj_builder(
            stand_in,
            JavaScriptRuntime::Node,
            Path::new("chair.ts"),
            output,
            LIBRARIES.as_bytes(),
        )
        .err()
        .map(|error| error.to_string())
    }

    #[test]
    fn a_build_writes_the_document_and_removes_the_builder() {
        let stand_in = StandIn::new(|| Ok(Some(0)));
        let out = TempDir::new().unwrap();

        assert_eq!(build_chair(&stand_in, &out), None);

        let output = fs::read_to_string(out.path().join("models/chair.sdfj"));
        assert_eq!(output.unwrap(), DOCUMENT);
        assert!(!stand_in.directory.borrow().as_ref().unwrap().exists());
    }

    #[test]
    fn an_sdfjz_output_holds_the_document_zipped() {
        let stand_in = StandIn::new(|| Ok(Some(0)));
        let out = TempDir::new().unwrap();
        let output = out.path().join("models/chair.SDFJZ");

        assert_eq!(build_chair_to(&stand_in, &output), None);

        let zip = fs::read(output).unwrap();
        assert!(zip.starts_with(b"PK"));
        assert_eq!(
            unwrap_sdfjz(&SdfjCodecDependencies, &zip).unwrap(),
            DOCUMENT.as_bytes()
        );
    }

    #[test]
    fn a_failed_run_errors_and_writes_no_document() {
        for (exit, message) in [
            (
                (|| Ok(Some(1))) as fn() -> IOResult<Option<i32>>,
                "building `chair.ts` under `node` failed with exit code 1",
            ),
            (
                || Ok(None),
                "building `chair.ts` under `node` ended on a signal",
            ),
            (
                || Err(IOError::from(ErrorKind::NotFound)),
                "the runtime `node` is not on the PATH",
            ),
        ] {
            let out = TempDir::new().unwrap();

            assert_eq!(
                build_chair(&StandIn::new(exit), &out).as_deref(),
                Some(message)
            );
            assert!(!out.path().join("models").exists());
        }
    }
}
