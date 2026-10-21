use crate::{
    Dependencies, Error, Result, cli_value_parser,
    commands::{
        BuildVar, SdfDocBuildProfile, load_sdf_doc_build_profile_set, run_sdfj_builder,
        sdfj_builder_libraries,
    },
};
use clap::{ArgAction, Parser};
use sdfj_builder::JavaScriptRuntime;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

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

    /// Sets the build variable the model reads as `vars.<name>` to a JSON
    /// value. A string takes quotes, as in `wood='"oak"'`. The flag replaces
    /// the profile's variable of that name. Repeatable, once per name.
    #[arg(value_name = "name=value", long)]
    var: Vec<BuildVar>,

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
        let vars = serde_json::to_vec(&self.vars(&profile)?).expect("JSON values serialize");

        run_sdfj_builder(
            &dependencies,
            self.runtime(&profile),
            &self.model,
            &self.output(),
            &libraries,
            &vars,
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

    /// The profile's `vars` with each `--var` replacing the entry of its name.
    fn vars(&self, profile: &SdfDocBuildProfile) -> Result<BTreeMap<String, Value>> {
        let mut vars = profile.vars.clone();
        let mut flagged = BTreeSet::new();
        for var in &self.var {
            if !flagged.insert(&var.name) {
                return Err(Error::usage(format!("--var sets `{}` twice", var.name)));
            }
            vars.insert(var.name.clone(), var.value.clone());
        }
        Ok(vars)
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

#[cfg(test)]
mod tests {
    use crate::commands::{SdfDocBuild, SdfDocBuildProfile};
    use clap::Parser;
    use serde_json::json;
    use std::collections::BTreeMap;

    fn build(vars: &[&str]) -> SdfDocBuild {
        let vars = vars.iter().flat_map(|var| ["--var", var]);
        SdfDocBuild::try_parse_from(["build", "chair.ts"].into_iter().chain(vars)).unwrap()
    }

    #[test]
    fn a_var_flag_replaces_only_the_profile_var_of_its_name() {
        let profile = SdfDocBuildProfile {
            vars: BTreeMap::from([
                ("height".to_owned(), json!(1)),
                ("legs".to_owned(), json!(4)),
            ]),
            ..SdfDocBuildProfile::default()
        };

        let vars = build(&["legs=3", "wood=\"oak\""]).vars(&profile).unwrap();

        assert_eq!(
            vars,
            BTreeMap::from([
                ("height".to_owned(), json!(1)),
                ("legs".to_owned(), json!(3)),
                ("wood".to_owned(), json!("oak")),
            ])
        );
    }

    #[test]
    fn a_var_flagged_twice_errors() {
        let error = build(&["legs=3", "legs=4"])
            .vars(&SdfDocBuildProfile::default())
            .unwrap_err();

        assert_eq!(error.to_string(), "error: --var sets `legs` twice\n");
    }
}
