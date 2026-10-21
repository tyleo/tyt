use crate::{
    Dependencies, Error, ProfileSet, Result, cli_value_parser,
    commands::{
        BuildVar, SdfDocBuildProfile, land_profile_vars, load_sdf_doc_build_profile_set,
        run_sdfj_builder, sdfj_builder_libraries,
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
    /// any profile's variable of that name. Repeatable, once per name.
    #[arg(value_name = "name=value", long)]
    var: Vec<BuildVar>,

    /// Imports a profile's build variables after its `varsFrom` imports,
    /// replacing the variables of `--profile` and earlier flags by name. Every
    /// other element the profile holds stays behind. Repeatable.
    #[arg(value_name = "profile", long)]
    vars_from: Vec<String>,

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
        let profiles = if self.profile.is_some() || !self.vars_from.is_empty() {
            load_sdf_doc_build_profile_set(&dependencies)?
        } else {
            ProfileSet::layered(BTreeMap::new(), [])
        };

        let profile = match &self.profile {
            Some(name) => profiles.get("--profile", name)?.clone(),
            None => SdfDocBuildProfile::default(),
        };

        let (origin, libraries) = self.libraries(&profile);
        let libraries = sdfj_builder_libraries(&dependencies, origin, libraries)?;
        let vars = serde_json::to_vec(&self.vars(&profiles)?).expect("JSON values serialize");

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

    fn vars(&self, profiles: &ProfileSet<SdfDocBuildProfile>) -> Result<BTreeMap<String, Value>> {
        let mut vars = BTreeMap::new();
        if let Some(name) = &self.profile {
            land_profile_vars(profiles, "--profile", name, &mut vars)?;
        }

        for (position, name) in (0..).zip(&self.vars_from) {
            if self.vars_from[..position].contains(name) {
                return Err(Error::usage(format!("--vars-from lists `{name}` twice")));
            }
            land_profile_vars(profiles, "--vars-from", name, &mut vars)?;
        }

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
    use crate::{
        ProfileSet,
        commands::{SdfDocBuild, SdfDocBuildProfile},
        profile_set_from_json,
    };
    use clap::Parser;
    use serde_json::json;
    use std::collections::BTreeMap;

    fn build(flags: &[&str]) -> SdfDocBuild {
        SdfDocBuild::try_parse_from(["build", "chair.ts"].iter().chain(flags)).unwrap()
    }

    fn profiles() -> ProfileSet<SdfDocBuildProfile> {
        profile_set_from_json(&[
            ("table", r#"{ "vars": { "height": 0.75, "legs": 4 } }"#),
            (
                "tall",
                r#"{ "varsFrom": ["table"], "vars": { "height": 1.1 } }"#,
            ),
            (
                "short-oak",
                r#"{ "vars": { "height": 0.5, "legs": 3, "wood": "oak" } }"#,
            ),
        ])
    }

    #[test]
    fn the_command_line_replaces_the_profile_s_vars_by_name() {
        let vars = build(&[
            "--profile",
            "tall",
            "--vars-from",
            "short-oak",
            "--var",
            "wood=\"pine\"",
        ])
        .vars(&profiles())
        .unwrap();

        assert_eq!(
            vars,
            BTreeMap::from([
                ("height".to_owned(), json!(0.5)),
                ("legs".to_owned(), json!(3)),
                ("wood".to_owned(), json!("pine")),
            ])
        );
    }

    #[test]
    fn vars_from_lands_a_profile_the_profile_imported_again() {
        let vars = build(&["--profile", "tall", "--vars-from", "table"])
            .vars(&profiles())
            .unwrap();

        assert_eq!(vars["height"], json!(0.75));
    }

    #[test]
    fn a_name_flagged_twice_errors() {
        for (flags, message) in [
            (
                ["--var", "legs=3", "--var", "legs=4"],
                "--var sets `legs` twice",
            ),
            (
                ["--vars-from", "table", "--vars-from", "table"],
                "--vars-from lists `table` twice",
            ),
        ] {
            let error = build(&flags).vars(&profiles()).unwrap_err();
            assert_eq!(error.to_string(), format!("error: {message}\n"));
        }
    }
}
