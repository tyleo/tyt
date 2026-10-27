use crate::{
    CliValue, Dependencies, Error, MeshInput, NoneOr, Result, Rgba, VoxjEncodingOptions,
    cli_value_parser,
    commands::{GridResolutionOptions, MeshDocVoxelizeProfile, load_mesh_doc_voxelize_profile_set},
    warn_slashed_names,
};
use clap::Parser;
use meshconv::load;
use std::path::PathBuf;
use voxconv::{
    WriteFormat, save,
    voxj::{EditStateMode, VoxjWriteFormat, VoxjWriteOptions},
};
use voxsmith::{
    dependencies::DependenciesImpl as VoxsmithDependenciesImpl,
    operations::mesh_doc::{
        MaterialMode, OutOfRangeProperty, SurfaceMode, VoxelScale, VoxelizeOptions, voxelize,
    },
    utilities::{FillMode, FlattenMode, GridResolution, VoxelFrame},
};

/// Rasterizes a mesh into voxel objects, the inverse of `object mesh`.
#[derive(Clone, Debug, Parser)]
#[command(name = "voxelize")]
pub struct MeshDocVoxelize {
    #[command(flatten)]
    input: MeshInput,

    /// The output `.voxj` or `.voxjz` document to write. Defaults to the input
    /// path with a `.voxj` extension, or `.voxjz` when `--format zip`.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    #[command(flatten)]
    resolution_options: GridResolutionOptions,

    /// The frame each object's grid is built in. Defaults to `world`.
    #[arg(value_name = "frame", long, value_parser = cli_value_parser::<VoxelFrame>())]
    frame: Option<VoxelFrame>,

    /// What happens to a placing node's scale. Defaults to `bake`.
    #[arg(value_name = "scale", long, value_parser = cli_value_parser::<VoxelScale>())]
    scale: Option<VoxelScale>,

    /// How much of the hierarchy the document flattens. Flattening requires
    /// `--frame world` and `--scale bake`. Defaults to `none`.
    #[arg(
        value_name = "flatten",
        long,
        value_parser = cli_value_parser::<FlattenMode>()
    )]
    flatten: Option<FlattenMode>,

    /// How the mesh fills the grid, independent of `--material-mode`. `solid`
    /// needs a watertight mesh. Defaults to `solid`.
    #[arg(value_name = "fill-mode", long, value_parser = cli_value_parser::<FillMode>())]
    fill_mode: Option<FillMode>,

    /// How the surface is found, independent of `--fill-mode`. Defaults to
    /// `center-inside`.
    #[arg(
        value_name = "surface-mode",
        long,
        value_parser = cli_value_parser::<SurfaceMode>()
    )]
    surface_mode: Option<SurfaceMode>,

    /// Where each voxel's color comes from, independent of `--fill-mode`.
    /// Defaults to `auto`.
    #[arg(
        value_name = "material-mode",
        long,
        value_parser = cli_value_parser::<MaterialMode>()
    )]
    material_mode: Option<MaterialMode>,

    /// Fill color as a `#RRGGBBAA` hex color, or `none`. Under
    /// `--material-mode flat` it paints every voxel, white when `none`. Under
    /// `--fill-mode solid` it paints a body's interior, the nearest surface
    /// color when `none`. Rejected on a sampling-mode surface, which samples
    /// every voxel. Defaults to `none`.
    #[arg(value_name = "fill-color", long)]
    fill_color: Option<NoneOr<Rgba>>,

    /// What a source material value outside its property's range does, such
    /// as a `metallic` above `1`. Defaults to `error`.
    #[arg(
        value_name = "out-of-range-property",
        long,
        value_parser = cli_value_parser::<OutOfRangeProperty>()
    )]
    out_of_range_property: Option<OutOfRangeProperty>,

    /// Applies saved voxelize flags. A flag given here overrides the element
    /// it mirrors. Either voxel-size flag overrides both `resolution` and
    /// `voxelSize`. The profiles come from every `.vxlconfig`'s
    /// `meshDoc.voxelize.profiles`, the user's `~/.vxlconfig` first and then
    /// each directory from the git root down to the working directory. A name
    /// reads from the last file supplying it.
    #[arg(value_name = "profile", long)]
    profile: Option<String>,

    #[command(flatten)]
    encoding_options: VoxjEncodingOptions,
}

impl MeshDocVoxelize {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let profile = match &self.profile {
            Some(name) => load_mesh_doc_voxelize_profile_set(&dependencies)?
                .get("--profile", name)?
                .clone(),

            None => MeshDocVoxelizeProfile::default(),
        };
        let options = self.resolve(&profile)?;

        let (serialization, write_options, output) = self
            .encoding_options
            .resolve_output(&self.input.path, self.output);

        // A voxelized mesh has neither a source ext to carry nor an editor
        // build volume to record.
        let write_options = VoxjWriteOptions {
            ext: false,
            edit_state: EditStateMode::Never,
            ..write_options
        };

        let from = self.input.resolve_format()?;

        let document = load(&dependencies, from, &self.input.path)?;

        let main = voxelize(&VoxsmithDependenciesImpl, &document, &options)?;

        warn_slashed_names(&dependencies, main.state())?;

        Ok(save(
            &dependencies,
            &WriteFormat::Voxj(VoxjWriteFormat {
                serialization,
                options: write_options,
            }),
            main,
            &output,
        )?)
    }

    /// The [`VoxelizeOptions`] these flags set over `profile`, each flag
    /// overriding the element it mirrors. Errors on a combination the
    /// voxelizer would reject or ignore.
    fn resolve(&self, profile: &MeshDocVoxelizeProfile) -> Result<VoxelizeOptions> {
        let profile_resolution = profile.grid_resolution()?;

        let resolution = self
            .resolution_options
            .resolve()?
            .or(profile_resolution)
            .unwrap_or(GridResolution::VoxelSize(1.0));

        let stem = self
            .input
            .path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("voxelized");

        let options = VoxelizeOptions {
            resolution,
            frame: self
                .frame
                .or(profile.frame.map(|named| named.0))
                .unwrap_or(VoxelFrame::World),
            scale: self
                .scale
                .or(profile.scale.map(|named| named.0))
                .unwrap_or(VoxelScale::Bake),
            flatten: self
                .flatten
                .or(profile.flatten.map(|named| named.0))
                .unwrap_or(FlattenMode::None),
            surface_mode: self
                .surface_mode
                .or(profile.surface_mode.map(|named| named.0))
                .unwrap_or(SurfaceMode::CenterInside),
            fill_mode: self
                .fill_mode
                .or(profile.fill_mode.map(|named| named.0))
                .unwrap_or(FillMode::Solid),
            material_mode: self
                .material_mode
                .or(profile.material_mode.map(|named| named.0))
                .unwrap_or(MaterialMode::Auto),
            fill_color: self
                .fill_color
                .or(profile.fill_color)
                .and_then(NoneOr::value)
                .map(|color| color.0),
            fallback_name: Some(stem.to_owned()),
            out_of_range_property: self
                .out_of_range_property
                .or(profile.out_of_range_property.map(|named| named.0))
                .unwrap_or(OutOfRangeProperty::Error),
        };

        validate_fill_color(&options)?;
        validate_reference(&options)?;
        validate_flatten(&options)?;

        Ok(options)
    }
}

/// Rejects a `--fill-color` that a sampling-mode surface shell would drop.
fn validate_fill_color(options: &VoxelizeOptions) -> Result<()> {
    if options.fill_color.is_some()
        && options.fill_mode == FillMode::Surface
        && options.material_mode != MaterialMode::Flat
    {
        return Err(Error::usage(
            "--fill-color has no effect with --fill-mode surface and a sampling \
             --material-mode; it applies under --material-mode flat or --fill-mode solid",
        ));
    }

    Ok(())
}

/// Rejects a world `--resolution` reference under `--scale keep`, which leaves
/// no world voxel size to divide.
fn validate_reference(options: &VoxelizeOptions) -> Result<()> {
    if let GridResolution::ReferenceCount { reference, .. } = options.resolution
        && reference.is_world()
        && options.scale == VoxelScale::Keep
    {
        return Err(Error::usage(format!(
            "--resolution {} measures world space, which --scale keep does not voxelize in; \
             use an object reference or --voxel-size",
            reference.name()
        )));
    }

    Ok(())
}

/// Rejects a `--flatten` outside `--frame world` and `--scale bake`. The
/// objects share no one grid there.
fn validate_flatten(options: &VoxelizeOptions) -> Result<()> {
    if options.flatten != FlattenMode::None
        && (options.frame != VoxelFrame::World || options.scale != VoxelScale::Bake)
    {
        return Err(Error::usage(format!(
            "--flatten {} needs --frame world and --scale bake",
            options.flatten.name()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        Result,
        commands::{MeshDocVoxelize, MeshDocVoxelizeProfile},
    };
    use clap::Parser;
    use voxsmith::{
        operations::mesh_doc::{VoxelScale, VoxelizeOptions},
        utilities::{FillMode, FlattenMode, GridResolution, ResolutionReference, VoxelFrame},
    };

    /// The options a `mesh-doc voxelize` invocation of `args` resolves to over
    /// `profile`.
    fn resolve_over(args: &[&str], profile: &MeshDocVoxelizeProfile) -> Result<VoxelizeOptions> {
        let mut argv = vec!["voxelize", "model.glb"];
        argv.extend_from_slice(args);
        MeshDocVoxelize::try_parse_from(argv)
            .unwrap()
            .resolve(profile)
    }

    /// The options `args` resolve to with a valid resolution already set and
    /// no profile, so a test only supplies the flags it exercises.
    fn resolve(args: &[&str]) -> Result<VoxelizeOptions> {
        let mut argv = vec!["--resolution", "longest-world", "32"];
        argv.extend_from_slice(args);
        resolve_over(&argv, &MeshDocVoxelizeProfile::default())
    }

    /// The profile `json` defines.
    fn profile(json: &str) -> MeshDocVoxelizeProfile {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn a_fill_color_is_rejected_on_a_sampled_surface() {
        assert!(
            resolve(&[
                "--fill-mode",
                "surface",
                "--material-mode",
                "per-texel",
                "--fill-color",
                "#ff0000",
            ])
            .is_err()
        );
    }

    #[test]
    fn a_flat_surface_accepts_a_fill_color() {
        assert!(
            resolve(&[
                "--fill-mode",
                "surface",
                "--material-mode",
                "flat",
                "--fill-color",
                "#ff0000",
            ])
            .is_ok()
        );
    }

    #[test]
    fn a_solid_body_accepts_a_fill_color() {
        assert!(
            resolve(&[
                "--fill-mode",
                "solid",
                "--material-mode",
                "per-texel",
                "--fill-color",
                "#ff0000",
            ])
            .is_ok()
        );
    }

    #[test]
    fn a_sampled_surface_without_a_fill_color_is_fine() {
        assert!(resolve(&["--fill-mode", "surface", "--material-mode", "per-texel"]).is_ok());
    }

    #[test]
    fn an_omitted_fill_color_is_none() {
        assert_eq!(resolve(&[]).unwrap().fill_color, None);
    }

    #[test]
    fn a_world_reference_is_rejected_under_a_kept_scale() {
        assert!(resolve(&["--scale", "keep"]).is_err());
        assert!(resolve(&[]).is_ok());

        let object = resolve_over(
            &["--resolution", "longest-object", "32", "--scale", "keep"],
            &MeshDocVoxelizeProfile::default(),
        );
        assert!(object.is_ok());
    }

    #[test]
    fn flattening_needs_the_world_frame_and_a_baked_scale() {
        assert_eq!(
            resolve(&["--flatten", "objects"]).unwrap().flatten,
            FlattenMode::Objects
        );
        assert_eq!(resolve(&[]).unwrap().flatten, FlattenMode::None);

        for args in [
            &["--flatten", "nodes", "--frame", "local"][..],
            &["--flatten", "nodes", "--scale", "keep"][..],
        ] {
            let error = resolve_over(
                &[&["--voxel-size", "0.5"][..], args].concat(),
                &MeshDocVoxelizeProfile::default(),
            )
            .unwrap_err()
            .to_string();
            assert!(
                error.contains("--flatten nodes needs --frame world and --scale bake"),
                "{error}"
            );
        }
    }

    #[test]
    fn neither_voxel_size_flag_defaults_to_one_meter_per_voxel() {
        let options = resolve_over(&[], &MeshDocVoxelizeProfile::default()).unwrap();
        assert_eq!(options.resolution, GridResolution::VoxelSize(1.0));
    }

    #[test]
    fn a_profile_fills_what_the_flags_leave_and_the_flags_win() {
        let local = profile(
            r##"{
                "resolution": { "reference": "longest-object", "count": 16 },
                "frame": "local",
                "fillColor": "#ff0000"
            }"##,
        );

        let options = resolve_over(&[], &local).unwrap();
        assert_eq!(
            options.resolution,
            GridResolution::ReferenceCount {
                reference: ResolutionReference::LongestObject,
                count: 16,
            }
        );
        assert_eq!(options.frame, VoxelFrame::Local);
        assert_eq!(options.fill_color, Some([255, 0, 0, 255]));
        assert_eq!(options.fill_mode, FillMode::Solid);

        let options = resolve_over(
            &[
                "--voxel-size",
                "0.5",
                "--frame",
                "world",
                "--fill-color",
                "none",
            ],
            &local,
        )
        .unwrap();
        assert_eq!(options.resolution, GridResolution::VoxelSize(0.5));
        assert_eq!(options.frame, VoxelFrame::World);
        assert_eq!(options.fill_color, None);
    }

    #[test]
    fn a_profile_combination_is_checked_like_the_flags() {
        let kept = profile(r#"{ "scale": "keep" }"#);
        assert!(resolve_over(&["--resolution", "world-x", "8"], &kept).is_err());

        let kept_world =
            profile(r#"{ "scale": "keep", "resolution": { "reference": "world-x", "count": 8 } }"#);
        assert!(resolve_over(&[], &kept_world).is_err());
        assert_eq!(
            resolve_over(&["--scale", "bake"], &kept_world)
                .unwrap()
                .scale,
            VoxelScale::Bake
        );
    }
}
