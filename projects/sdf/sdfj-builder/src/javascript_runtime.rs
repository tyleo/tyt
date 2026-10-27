use std::{ffi::OsString, path::Path};

/// A runtime that runs the builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JavaScriptRuntime {
    /// Bun.
    Bun,

    /// Deno.
    Deno,

    /// Node 24 or later.
    Node,
}

impl JavaScriptRuntime {
    /// The program that starts the runtime.
    pub fn program(self) -> &'static str {
        match self {
            JavaScriptRuntime::Bun => "bun",
            JavaScriptRuntime::Deno => "deno",
            JavaScriptRuntime::Node => "node",
        }
    }

    /// The arguments that start the builder written under `directory` to record
    /// `model` as the document at `output`. Without `intern` the document keeps
    /// an entry per value.
    pub fn args(
        self,
        directory: &Path,
        model: &Path,
        output: &Path,
        intern: bool,
    ) -> Vec<OsString> {
        let mut args: Vec<OsString> = match self {
            JavaScriptRuntime::Bun => vec!["run".into()],

            // The builder's config keeps a config beside the model out of the
            // run.
            JavaScriptRuntime::Deno => vec![
                "run".into(),
                "--allow-read".into(),
                "--allow-write".into(),
                "--config".into(),
                directory.join("deno.json").into(),
            ],

            JavaScriptRuntime::Node => Vec::new(),
        };
        args.extend([
            directory.join("ts/main.ts").into(),
            model.into(),
            output.into(),
        ]);
        if !intern {
            args.extend(["--intern".into(), "false".into()]);
        }
        args
    }
}

#[cfg(test)]
mod tests {
    use crate::{JavaScriptRuntime, SDFJ_BUILDER_FILES};
    use std::{ffi::OsString, path::Path};

    fn args(runtime: JavaScriptRuntime) -> Vec<OsString> {
        runtime.args(
            Path::new("/run"),
            Path::new("chair.ts"),
            Path::new("/out/chair.sdfj"),
            true,
        )
    }

    #[test]
    fn each_runtime_starts_the_entry_point_on_the_model() {
        assert_eq!(
            args(JavaScriptRuntime::Bun),
            ["run", "/run/ts/main.ts", "chair.ts", "/out/chair.sdfj"]
        );
        assert_eq!(
            args(JavaScriptRuntime::Deno),
            [
                "run",
                "--allow-read",
                "--allow-write",
                "--config",
                "/run/deno.json",
                "/run/ts/main.ts",
                "chair.ts",
                "/out/chair.sdfj",
            ]
        );
        assert_eq!(
            args(JavaScriptRuntime::Node),
            ["/run/ts/main.ts", "chair.ts", "/out/chair.sdfj"]
        );
    }

    #[test]
    fn a_build_without_interning_ends_on_the_flag() {
        let args = JavaScriptRuntime::Node.args(
            Path::new("/run"),
            Path::new("chair.ts"),
            Path::new("/out/chair.sdfj"),
            false,
        );
        assert_eq!(
            args,
            [
                "/run/ts/main.ts",
                "chair.ts",
                "/out/chair.sdfj",
                "--intern",
                "false",
            ]
        );
    }

    #[test]
    fn the_args_reference_only_files_the_builder_holds() {
        for runtime in [
            JavaScriptRuntime::Bun,
            JavaScriptRuntime::Deno,
            JavaScriptRuntime::Node,
        ] {
            let args = args(runtime);
            let paths = args
                .iter()
                .filter_map(|arg| arg.to_str().unwrap().strip_prefix("/run/"));
            for path in paths {
                assert!(
                    SDFJ_BUILDER_FILES.iter().any(|file| file.path == path),
                    "{path}"
                );
            }
        }
    }
}
