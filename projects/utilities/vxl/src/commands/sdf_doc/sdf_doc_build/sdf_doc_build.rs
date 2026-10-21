use crate::{
    Dependencies, Result, cli_value_parser,
    commands::{
        SdfDocBuildProfile, load_sdf_doc_build_profile_set, run_sdfj_builder,
        sdfj_builder_libraries,
    },
};
use clap::{ArgAction, Parser};
use sdfj_builder::JavaScriptRuntime;
use std::path::PathBuf;

/// Records a TypeScript model as an `.sdfj` document.
#[derive(Clone, Debug, Parser)]
#[command(name = "build")]
pub struct SdfDocBuild {
    /// The input TypeScript model file.
    #[arg(value_name = "model")]
    model: PathBuf,

    /// The output `.sdfj` or `.sdfjz` document to write. Defaults to the
    /// model's path with an `.sdfj` extension. An `.sdfjz` path writes the
    /// document zipped.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    /// A library the model reads through `lib` and `mat`. The flag replaces the
    /// profile's `libraries`. vxl defines `materials`, and each `.vxlconfig`
    /// can define more at `sdfDoc.build.libraries`. Repeatable; a later library
    /// wins a name.
    #[arg(value_name = "library", long)]
    library: Vec<String>,

    /// The runtime that runs the model, which has to be installed. Defaults to
    /// `node`.
    #[arg(
        value_name = "runtime",
        long,
        value_parser = cli_value_parser::<JavaScriptRuntime>()
    )]
    runtime: Option<JavaScriptRuntime>,

    /// Writes one entry per distinct entry text, so a value the model builds
    /// twice reads once. `--intern false` writes an entry per value instead.
    /// Defaults to `true`.
    #[arg(
        value_name = "intern",
        long,
        num_args = 0..=1,
        default_missing_value = "true",
        action = ArgAction::Set
    )]
    intern: Option<bool>,

    /// Applies saved build flags. A flag given here overrides the element it
    /// mirrors. The profiles come from every `.vxlconfig`'s
    /// `sdfDoc.build.profiles`, the user's `~/.vxlconfig` first and then each
    /// directory from the git root down to the working directory. A name reads
    /// from the last file supplying it.
    #[arg(value_name = "profile", long)]
    profile: Option<String>,
}

impl SdfDocBuild {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let profile = match &self.profile {
            Some(name) => load_sdf_doc_build_profile_set(&dependencies)?
                .get("--profile", name)?
                .clone(),

            None => SdfDocBuildProfile::default(),
        };

        let (origin, libraries) = self.libraries(&profile);
        let libraries = sdfj_builder_libraries(&dependencies, origin, libraries)?;

        run_sdfj_builder(
            &dependencies,
            self.runtime(&profile),
            &self.model,
            &self.output(),
            &libraries,
            self.intern(&profile),
        )
    }

    /// The setting that lists the build's libraries, and the libraries it lists.
    fn libraries<'a>(&'a self, profile: &'a SdfDocBuildProfile) -> (&'static str, &'a [String]) {
        if self.library.is_empty() {
            ("the profile's `libraries`", &profile.libraries)
        } else {
            ("--library", &self.library)
        }
    }

    fn runtime(&self, profile: &SdfDocBuildProfile) -> JavaScriptRuntime {
        self.runtime
            .or(profile.runtime.map(|named| named.0))
            .unwrap_or(JavaScriptRuntime::Node)
    }

    fn intern(&self, profile: &SdfDocBuildProfile) -> bool {
        self.intern.or(profile.intern).unwrap_or(true)
    }

    fn output(&self) -> PathBuf {
        self.output
            .clone()
            .unwrap_or_else(|| self.model.with_extension("sdfj"))
    }
}
