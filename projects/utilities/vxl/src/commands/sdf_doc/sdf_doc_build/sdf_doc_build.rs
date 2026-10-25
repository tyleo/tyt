use crate::{
    Dependencies, Error, ProfileOrigin, ProfileSet, Result, cli_value_parser,
    commands::{
        BuildVar, FontFlagParser, SdfDocBuildFont, SdfDocBuildProfile, document_font_path,
        land_profile_vars, load_sdf_doc_build_profile_set, run_sdfj_builder,
        sdfj_builder_libraries,
    },
};
use clap::{ArgAction, Parser};
use sdfj::SdfjFont;
use sdfj_builder::JavaScriptRuntime;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
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

    /// Sets the font a model's `text` reads as `name`, as in `--font installed
    /// sign=Futura`. A `file` font takes a font file's path, and an
    /// `installed` font takes an installed face's family or full name. The
    /// flag replaces any profile's font of that name. Repeatable, once per
    /// name.
    #[arg(
        value_names = ["kind", "name=font"],
        long,
        num_args = 2,
        value_parser = FontFlagParser,
        hide_possible_values = true
    )]
    font: Vec<String>,

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

        let (profile_dir, profile) = match &self.profile {
            Some(name) => {
                let (origin, profile) = profiles.get_with_origin("--profile", name)?;
                let ProfileOrigin::File(config) = origin else {
                    unreachable!("vxl ships no build profiles");
                };
                let dir = config.parent().expect("a `.vxlconfig` sits in a directory");
                (Some(dir.to_owned()), profile.clone())
            }

            None => (None, SdfDocBuildProfile::default()),
        };

        let cwd = dependencies.resolve_prefs_paths()?.cwd;
        let (origin, libraries) = self.libraries(&profile);
        let libraries =
            sdfj_builder_libraries(&dependencies, origin, libraries, &self.document_dir(&cwd))?;
        let fonts = self.fonts(&profile, profile_dir.as_deref().unwrap_or(&cwd), &cwd)?;
        let fonts = serde_json::to_vec(&fonts).expect("fonts serialize");
        let vars = serde_json::to_vec(&self.vars(&profiles)?).expect("JSON values serialize");

        run_sdfj_builder(
            &dependencies,
            self.runtime(&profile),
            &self.model,
            &self.output(),
            &libraries,
            &fonts,
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

    /// The document's `fonts` entries in name order, with each file path
    /// relative to the output's directory. A profile's path resolves against
    /// `profile_dir` and a flag's against `cwd`.
    fn fonts(
        &self,
        profile: &SdfDocBuildProfile,
        profile_dir: &Path,
        cwd: &Path,
    ) -> Result<Vec<SdfjFont>> {
        let document_dir = self.document_dir(cwd);

        let mut fonts = BTreeMap::new();
        for (name, font) in &profile.fonts {
            let font = sdfj_font(name, font, profile_dir, &document_dir)?;
            fonts.insert(name.as_str(), font);
        }

        let mut flagged = BTreeSet::new();
        for values in self.font.chunks_exact(2) {
            let [kind, entry] = values else {
                unreachable!("--font takes two values");
            };
            let Some((name, font)) = entry.split_once('=') else {
                return Err(Error::usage(format!(
                    "--font takes `<name>=<font>`, not `{entry}`"
                )));
            };
            if name.is_empty() {
                return Err(Error::usage(format!(
                    "--font `{entry}` has no name before the `=`"
                )));
            }

            let font = match kind.as_str() {
                "file" => SdfDocBuildFont::File { path: font.into() },

                "installed" => SdfDocBuildFont::Installed {
                    font: font.to_owned(),
                },

                _ => {
                    return Err(Error::usage(format!(
                        "--font takes the kind `file` or `installed`, not `{kind}`"
                    )));
                }
            };

            if !flagged.insert(name) {
                return Err(Error::usage(format!("--font sets `{name}` twice")));
            }
            fonts.insert(name, sdfj_font(name, &font, cwd, &document_dir)?);
        }

        Ok(fonts.into_values().collect())
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

    /// The absolute directory of the output document.
    fn document_dir(&self, cwd: &Path) -> PathBuf {
        let output = cwd.join(self.output());
        let dir = output.parent().expect("the output path holds a file name");
        dir.to_owned()
    }

    fn output(&self) -> PathBuf {
        self.output
            .clone()
            .unwrap_or_else(|| self.model.with_extension("sdfj"))
    }
}

/// The `fonts` entry `name` takes for `font`, whose relative file path resolves
/// against `base` and lands relative to `document_dir`.
fn sdfj_font(
    name: &str,
    font: &SdfDocBuildFont,
    base: &Path,
    document_dir: &Path,
) -> Result<SdfjFont> {
    Ok(match font {
        SdfDocBuildFont::File { path } => SdfjFont::File {
            name: name.to_owned(),
            path: document_font_path(name, path, base, document_dir)?,
        },

        SdfDocBuildFont::Installed { font } => SdfjFont::Installed {
            name: name.to_owned(),
            font: font.clone(),
        },
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        ProfileSet,
        commands::{SdfDocBuild, SdfDocBuildProfile},
        profile_set_from_json,
    };
    use clap::Parser;
    use sdfj::SdfjFont;
    use serde_json::json;
    use std::{collections::BTreeMap, path::Path};

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

    /// A profile holding a font file and an installed font.
    fn font_profile() -> SdfDocBuildProfile {
        serde_json::from_str(
            r#"{ "fonts": {
                "label": { "kind": "file", "path": "fonts/plex.otf" },
                "sign": { "kind": "installed", "font": "Futura" }
            } }"#,
        )
        .unwrap()
    }

    fn file(name: &str, path: &str) -> SdfjFont {
        SdfjFont::File {
            name: name.to_owned(),
            path: path.to_owned(),
        }
    }

    fn installed(name: &str, font: &str) -> SdfjFont {
        SdfjFont::Installed {
            name: name.to_owned(),
            font: font.to_owned(),
        }
    }

    #[test]
    fn a_font_flag_replaces_the_profile_s_font_by_name() {
        let fonts = build(&["--font", "installed", "label=Menlo"])
            .fonts(&font_profile(), Path::new("/repo"), Path::new("/repo"))
            .unwrap();

        assert_eq!(
            fonts,
            [installed("label", "Menlo"), installed("sign", "Futura")]
        );
    }

    #[test]
    fn a_relative_font_file_lands_relative_to_the_output_s_directory() {
        let fonts = SdfDocBuild::try_parse_from([
            "build",
            "sign.ts",
            "out/sign.sdfj",
            "--font",
            "file",
            "small=../shared/./stix.otf",
            "--font",
            "file",
            "system=/fonts/plex.otf",
        ])
        .unwrap()
        .fonts(
            &font_profile(),
            Path::new("/repo"),
            Path::new("/repo/models"),
        )
        .unwrap();

        assert_eq!(
            fonts,
            [
                file("label", "../../fonts/plex.otf"),
                installed("sign", "Futura"),
                file("small", "../../shared/stix.otf"),
                file("system", "/fonts/plex.otf"),
            ]
        );
    }

    #[test]
    fn a_font_flag_without_a_kind_or_a_name_errors() {
        for (flags, message) in [
            (
                ["--font", "system", "sign=Futura"].as_slice(),
                "--font takes the kind `file` or `installed`, not `system`",
            ),
            (
                &["--font", "installed", "Futura"],
                "--font takes `<name>=<font>`, not `Futura`",
            ),
            (
                &["--font", "installed", "=Futura"],
                "--font `=Futura` has no name before the `=`",
            ),
            (
                &[
                    "--font",
                    "installed",
                    "sign=Futura",
                    "--font",
                    "file",
                    "sign=futura.otf",
                ],
                "--font sets `sign` twice",
            ),
        ] {
            let error = build(flags)
                .fonts(&font_profile(), Path::new("/repo"), Path::new("/repo"))
                .unwrap_err();
            assert_eq!(error.to_string(), format!("error: {message}\n"));
        }
    }
}
