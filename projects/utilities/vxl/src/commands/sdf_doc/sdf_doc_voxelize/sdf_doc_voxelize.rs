use crate::{
    CliValue, Dependencies, Error, Result, VoxjEncodingOptions, WriteFile, cli_value_parser,
    commands::{GridResolutionOptions, SdfDocVoxelizeProfile, load_sdf_doc_voxelize_profile_set},
};
use clap::{ArgAction, Parser};
use sdfconv::{ReadFormat, load};
use std::path::PathBuf;
use voxconv::{
    WriteFormat, save,
    voxj::{EditStateMode, VoxjWriteFormat, VoxjWriteOptions},
};
use voxsmith::{
    dependencies::DependenciesImpl as VoxsmithDependenciesImpl,
    operations::sdf_doc::{SdfSampleOptions, SdfVoxMainOptions, report, sample, to_vox_main},
    utilities::{FillMode, FlattenMode, GridResolution, VoxelFrame},
};

/// Samples an `.sdfj` document on a voxel grid into a Voxel Json document.
#[derive(Clone, Debug, Parser)]
#[command(name = "voxelize")]
pub struct SdfDocVoxelize {
    /// The input `.sdfj` document.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// The output `.voxj` or `.voxjz` document to write. Defaults to the input
    /// path with a `.voxj` extension, or `.voxjz` when `--format zip`.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    #[command(flatten)]
    resolution_options: GridResolutionOptions,

    /// The frame each part's object grid is built in. Defaults to `world`.
    #[arg(value_name = "frame", long, value_parser = cli_value_parser::<VoxelFrame>())]
    frame: Option<VoxelFrame>,

    /// How much of each part the part's object keeps. Defaults to `solid`.
    #[arg(value_name = "fill-mode", long, value_parser = cli_value_parser::<FillMode>())]
    fill_mode: Option<FillMode>,

    /// How much of the part hierarchy the document flattens. Flattening
    /// requires `--frame world`. Defaults to `none`.
    #[arg(
        value_name = "flatten",
        long,
        value_parser = cli_value_parser::<FlattenMode>()
    )]
    flatten: Option<FlattenMode>,

    /// Prints a line for the model, each part, each step, and each piece, and
    /// writes the lines to the output's stem with `-report.txt` beside it.
    /// `--report false` skips the report. Defaults to `true`.
    #[arg(
        value_name = "report",
        long,
        num_args = 0..=1,
        default_missing_value = "true",
        action = ArgAction::Set
    )]
    report: Option<bool>,

    /// Applies saved voxelize flags. A flag given here overrides the element
    /// it mirrors. Either voxel-size flag overrides both `resolution` and
    /// `voxelSize`. The profiles come from every `.vxlconfig`'s
    /// `sdfDoc.voxelize.profiles`, the user's `~/.vxlconfig` first and then
    /// each directory from the git root down to the working directory. A name
    /// reads from the last file supplying it.
    #[arg(value_name = "profile", long)]
    profile: Option<String>,

    #[command(flatten)]
    encoding_options: VoxjEncodingOptions,
}

/// The settings a run takes from its flags over its profile.
#[derive(Debug, PartialEq)]
struct Settings {
    sample: SdfSampleOptions,

    vox_main: SdfVoxMainOptions,

    report: bool,
}

impl SdfDocVoxelize {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let profile = match &self.profile {
            Some(name) => load_sdf_doc_voxelize_profile_set(&dependencies)?
                .get("--profile", name)?
                .clone(),

            None => SdfDocVoxelizeProfile::default(),
        };
        let settings = self.resolve(&profile)?;

        let (serialization, write_options, output) = self
            .encoding_options
            .resolve_output(&self.input, self.output);

        // A sampled model has neither a source ext to carry nor an editor
        // build volume to record.
        let write_options = VoxjWriteOptions {
            ext: false,
            edit_state: EditStateMode::Never,
            ..write_options
        };

        let main = load(&dependencies, ReadFormat::Sdfj, &self.input)?;
        let sampling = sample(&VoxsmithDependenciesImpl, &main, &settings.sample)?;
        let document = to_vox_main(&sampling, &settings.vox_main)?;

        save(
            &dependencies,
            &WriteFormat::Voxj(VoxjWriteFormat {
                serialization,
                options: write_options,
            }),
            document,
            &output,
        )?;

        if settings.report {
            // The report reads the grids of the world frame under either
            // frame.
            let world = match sampling.frame {
                VoxelFrame::World => sampling,

                VoxelFrame::Local => sample(
                    &VoxsmithDependenciesImpl,
                    &main,
                    &SdfSampleOptions {
                        frame: VoxelFrame::World,
                        record_writes: true,
                        ..settings.sample
                    },
                )?,
            };

            let name = output
                .file_name()
                .expect("an output path names a file")
                .to_string_lossy();
            let stem = output
                .file_stem()
                .expect("an output path names a file")
                .to_string_lossy();

            let report = report(&main, &world, settings.vox_main.fill_mode, &name)?;

            WriteFile::write_file(
                &dependencies,
                &output.with_file_name(format!("{stem}-report.txt")),
                report.as_bytes(),
            )?;
            dependencies.write_stdout(report.as_bytes())?;
        }

        Ok(())
    }

    /// The settings these flags set over `profile`. A flag overrides the
    /// element it mirrors. Errors on flattening beside `--frame local`.
    fn resolve(&self, profile: &SdfDocVoxelizeProfile) -> Result<Settings> {
        let profile_resolution = profile.grid_resolution()?;

        let resolution = self
            .resolution_options
            .resolve()?
            .or(profile_resolution)
            .unwrap_or(GridResolution::VoxelSize(1.0));

        let frame = self
            .frame
            .or(profile.frame.map(|named| named.0))
            .unwrap_or(VoxelFrame::World);

        let flatten = self
            .flatten
            .or(profile.flatten.map(|named| named.0))
            .unwrap_or(FlattenMode::None);

        if flatten != FlattenMode::None && frame == VoxelFrame::Local {
            return Err(Error::usage(format!(
                "--flatten {} needs --frame world",
                flatten.name()
            )));
        }

        let report = self.report.or(profile.report).unwrap_or(false);

        Ok(Settings {
            // Under the local frame the report samples the world frame again.
            sample: SdfSampleOptions {
                resolution,
                frame,
                record_writes: report && frame == VoxelFrame::World,
            },
            vox_main: SdfVoxMainOptions {
                fill_mode: self
                    .fill_mode
                    .or(profile.fill_mode.map(|named| named.0))
                    .unwrap_or(FillMode::Solid),
                flatten,
            },
            report,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Result,
        commands::{SdfDocVoxelize, SdfDocVoxelizeProfile},
    };
    use clap::Parser;
    use voxsmith::utilities::{
        FillMode, FlattenMode, GridResolution, ResolutionReference, VoxelFrame,
    };

    /// The frame, fill mode, flattening, and report setting that an
    /// `sdf-doc voxelize` invocation of `args` resolves to over the profile
    /// `json` defines.
    fn resolve(args: &[&str], json: &str) -> Result<(VoxelFrame, FillMode, FlattenMode, bool)> {
        let profile: SdfDocVoxelizeProfile = serde_json::from_str(json).unwrap();
        let mut argv = vec!["voxelize", "chair.sdfj"];
        argv.extend_from_slice(args);

        let settings = SdfDocVoxelize::try_parse_from(argv)
            .unwrap()
            .resolve(&profile)?;

        Ok((
            settings.sample.frame,
            settings.vox_main.fill_mode,
            settings.vox_main.flatten,
            settings.report,
        ))
    }

    #[test]
    fn the_settings_default_to_one_meter_solid_world_cells() {
        let settings = SdfDocVoxelize::try_parse_from(["voxelize", "chair.sdfj"])
            .unwrap()
            .resolve(&SdfDocVoxelizeProfile::default())
            .unwrap();

        assert_eq!(settings.sample.resolution, GridResolution::VoxelSize(1.0));
        assert_eq!(
            resolve(&[], "{}").unwrap(),
            (VoxelFrame::World, FillMode::Solid, FlattenMode::None, false)
        );
    }

    #[test]
    fn a_flag_overrides_the_element_it_mirrors() {
        let profile = r#"{ "frame": "local", "fillMode": "surface", "report": true }"#;

        assert_eq!(
            resolve(&[], profile).unwrap(),
            (
                VoxelFrame::Local,
                FillMode::Surface,
                FlattenMode::None,
                true
            )
        );
        assert_eq!(
            resolve(
                &[
                    "--frame",
                    "world",
                    "--fill-mode",
                    "solid",
                    "--report",
                    "false"
                ],
                profile
            )
            .unwrap(),
            (VoxelFrame::World, FillMode::Solid, FlattenMode::None, false)
        );
        assert_eq!(
            resolve(&["--flatten", "nodes", "--report"], "{}").unwrap(),
            (VoxelFrame::World, FillMode::Solid, FlattenMode::Nodes, true)
        );
        assert_eq!(
            resolve(&["--flatten", "none"], r#"{ "flatten": "objects" }"#).unwrap(),
            (VoxelFrame::World, FillMode::Solid, FlattenMode::None, false)
        );
    }

    #[test]
    fn either_size_flag_replaces_the_profile_resolution() {
        let settings = SdfDocVoxelize::try_parse_from([
            "voxelize",
            "chair.sdfj",
            "--resolution",
            "longest-world",
            "32",
        ])
        .unwrap()
        .resolve(&serde_json::from_str(r#"{ "voxelSize": 0.025 }"#).unwrap())
        .unwrap();

        assert_eq!(
            settings.sample.resolution,
            GridResolution::ReferenceCount {
                reference: ResolutionReference::LongestWorld,
                count: 32,
            }
        );
    }

    #[test]
    fn flattening_beside_the_local_frame_errors() {
        for (args, profile, message) in [
            (
                &["--flatten", "nodes", "--frame", "local"][..],
                "{}",
                "--flatten nodes needs --frame world",
            ),
            (
                &["--frame", "local"][..],
                r#"{ "flatten": "objects" }"#,
                "--flatten objects needs --frame world",
            ),
        ] {
            let error = resolve(args, profile).unwrap_err().to_string();
            assert!(error.contains(message), "{error}");
        }
    }

    #[test]
    fn the_size_flags_exclude_each_other() {
        assert!(
            SdfDocVoxelize::try_parse_from([
                "voxelize",
                "chair.sdfj",
                "--voxel-size",
                "0.025",
                "--resolution",
                "longest-world",
                "32",
            ])
            .is_err()
        );
    }
}
