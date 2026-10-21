use crate::{
    Dependencies, Result, cli_value_parser,
    commands::{
        SdfDocBuildProfile, load_sdf_doc_build_profile_set, run_sdfj_builder,
        sdfj_builder_libraries,
    },
};
use clap::Parser;
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

    fn output(&self) -> PathBuf {
        self.output
            .clone()
            .unwrap_or_else(|| self.model.with_extension("sdfj"))
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::{SdfDocBuild, SdfDocBuildProfile};
    use clap::Parser;
    use sdfj_builder::JavaScriptRuntime;
    use std::path::Path;

    fn parse(args: &[&str]) -> SdfDocBuild {
        let mut argv = vec!["build"];
        argv.extend_from_slice(args);
        SdfDocBuild::try_parse_from(argv).unwrap()
    }

    #[test]
    fn the_flag_overrides_the_profile_and_node_is_the_default() {
        let bun: SdfDocBuildProfile = serde_json::from_str(r#"{ "runtime": "bun" }"#).unwrap();

        let none = parse(&["chair.ts"]);
        assert_eq!(
            none.runtime(&SdfDocBuildProfile::default()),
            JavaScriptRuntime::Node
        );
        assert_eq!(none.runtime(&bun), JavaScriptRuntime::Bun);

        let deno = parse(&["chair.ts", "--runtime", "deno"]);
        assert_eq!(deno.runtime(&bun), JavaScriptRuntime::Deno);
    }

    #[test]
    fn the_flag_replaces_the_profile_libraries() {
        let studio: SdfDocBuildProfile =
            serde_json::from_str(r#"{ "libraries": ["materials", "props"] }"#).unwrap();

        let none = parse(&["chair.ts"]);
        assert_eq!(
            none.libraries(&SdfDocBuildProfile::default()).1,
            [] as [String; 0]
        );
        assert_eq!(
            none.libraries(&studio),
            (
                "the profile's `libraries`",
                &["materials", "props"].map(String::from)[..]
            )
        );

        let fabrics = parse(&["chair.ts", "--library", "fabrics", "--library", "props"]);
        assert_eq!(
            fabrics.libraries(&studio),
            ("--library", &["fabrics", "props"].map(String::from)[..])
        );
    }

    #[test]
    fn the_output_defaults_to_the_model_with_an_sdfj_extension() {
        assert_eq!(
            parse(&["models/chair.ts"]).output(),
            Path::new("models/chair.sdfj")
        );
        assert_eq!(
            parse(&["models/chair.ts", "out.sdfj"]).output(),
            Path::new("out.sdfj")
        );
    }
}
