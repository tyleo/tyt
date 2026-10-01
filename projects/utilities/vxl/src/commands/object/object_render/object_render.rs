use crate::{
    Dependencies, Error, ObjectSelection, PositiveF64, ProfileSet, Result, VoxelInput, WriteFile,
    cli_value_parser,
    commands::{
        Background, LightKind, LightTable, NonNegativeF64, OutputKind, ProjectionKind,
        RenderProfile, RenderProfileStack, SrgbColor, TransformFrame, ViewTable,
        load_render_profile_set,
    },
    flag_occurrences, parse_flag_index, parse_flag_value,
};
use branded_id::{IdVec, U32Id};
use clap::{ArgAction, Parser};
use std::{
    fmt::Display,
    num::NonZeroU32,
    path::{Path, PathBuf},
    str::FromStr,
};
use ty_math::{TyAngleUnit, TyQuaternionF64, TyVector3F64};
use voxconv::load;
use voxcore::{BVoxObject, VoxExt, VoxMain};
use voxsmith::{
    dependencies::DependenciesImpl as VoxsmithDependenciesImpl,
    operations::object::{
        BRenderView, FitOrFixed, PoseTransform, PositionTransform, RenderOcclusion, RenderRecord,
        RenderShadow, RenderView, Rotation, ViewRecord, encode_render_png, render,
    },
};

/// Renders the selected objects, placed by the hierarchy reaching them, under
/// the views and lights the profiles and flags set, to the terminal or to a
/// PNG per view. A run that sets no view renders the built-in `hero` view,
/// and one that sets no light uses the built-in `studio` rig.
#[derive(Clone, Debug, Parser)]
#[command(name = "render")]
pub struct ObjectRender {
    #[command(flatten)]
    input: VoxelInput,

    /// Where the views go, `terminal` or `png`, defaulting to `terminal`. In
    /// the terminal the views show one below another, each scaled to fit,
    /// through the Kitty or iTerm2 graphics protocol or as ANSI half blocks
    /// where neither is supported. As PNG, one view writes the stem with
    /// `.png` beside the input, and several write the stem, a hyphen, and
    /// the view's name.
    #[arg(value_name = "to", long, value_parser = cli_value_parser::<OutputKind>())]
    to: Option<OutputKind>,

    /// The stem the PNGs are named by under `--to png`, defaulting to the
    /// input's.
    #[arg(value_name = "file-stem", long)]
    file_stem: Option<String>,

    /// The image width in pixels, defaulting to `1024`.
    #[arg(value_name = "width", long)]
    width: Option<NonZeroU32>,

    /// The image height in pixels, defaulting to `1024`.
    #[arg(value_name = "height", long)]
    height: Option<NonZeroU32>,

    /// What fills the pixels no ray hits, `transparent` or a `#RRGGBB` color,
    /// defaulting to `transparent`.
    #[arg(value_name = "background", long)]
    background: Option<Background>,

    /// The occlusion the render shades with, defaulting to `corner`.
    #[arg(value_name = "occlusion", long, value_parser = cli_value_parser::<RenderOcclusion>())]
    occlusion: Option<RenderOcclusion>,

    /// The real-world edge length of one voxel in meters, defaulting to `1.0`
    /// and applied as a uniform scale to the whole scene, so a point light's
    /// falloff runs in meters after it applies.
    #[arg(value_name = "voxel-size", long)]
    voxel_size: Option<PositiveF64>,

    /// Applies a profile whole, expanding it into its flags with its
    /// `viewsFrom` and `lightsFrom` imports first. An explicit flag replaces
    /// the profile element it collides with. Repeatable: the profiles stack,
    /// views merging by name and the lights as one rig. An element two of
    /// them set errors. The profiles are the built-ins under every
    /// `.vxlconfig`'s `object.render.profiles`, the user's `~/.vxlconfig`
    /// first and then each directory from the git root down to the working
    /// directory, a name reading from the last file supplying it.
    #[arg(value_name = "profile", long, action = ArgAction::Append)]
    profile: Vec<String>,

    /// Applies a profile's views alone, its `viewsFrom` imports first.
    /// Repeatable.
    #[arg(value_name = "profile", long, action = ArgAction::Append)]
    views_from: Vec<String>,

    /// Applies a profile's light rig alone, its `lightsFrom` imports first.
    /// Repeatable.
    #[arg(value_name = "profile", long, action = ArgAction::Append)]
    lights_from: Vec<String>,

    /// The frame the named view's `--view-position` and rotation are read
    /// in, `world`, `subject`, or `node`. Repeatable.
    #[arg(
        value_names = ["view", "frame"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    view_frame: Vec<String>,

    /// The node path the named view's `node` frame reads its position and
    /// rotation in: a glob over hierarchy node paths that matches exactly
    /// one. Repeatable.
    #[arg(
        value_names = ["view", "path"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    view_node: Vec<String>,

    /// The named view's position in its frame, in meters. Repeatable.
    #[arg(
        value_names = ["view", "x", "y", "z"],
        long,
        num_args = 4,
        allow_negative_numbers = true,
        action = ArgAction::Append,
    )]
    view_position: Vec<String>,

    /// The named view's rotation as a unit quaternion. Repeatable.
    #[arg(
        value_names = ["view", "x", "y", "z", "w"],
        long,
        num_args = 5,
        allow_negative_numbers = true,
        action = ArgAction::Append,
    )]
    view_quaternion: Vec<String>,

    /// The named view's rotation as Euler angles in degrees about the fixed
    /// x, y, then z axes. Repeatable.
    #[arg(
        value_names = ["view", "x", "y", "z"],
        long,
        num_args = 4,
        allow_negative_numbers = true,
        action = ArgAction::Append,
    )]
    view_euler: Vec<String>,

    /// Aims the named view's -Z at a point in its frame, with its frame's +Y
    /// up. Repeatable.
    #[arg(
        value_names = ["view", "x", "y", "z"],
        long,
        num_args = 4,
        allow_negative_numbers = true,
        action = ArgAction::Append,
    )]
    view_look_at: Vec<String>,

    /// The named view's rotation as the one facing its frame's origin from
    /// the direction at an azimuth from +Z toward +X and an elevation toward
    /// +Y, in degrees. Repeatable.
    #[arg(
        value_names = ["view", "azimuth", "elevation"],
        long,
        num_args = 3,
        allow_negative_numbers = true,
        action = ArgAction::Append,
    )]
    view_angles: Vec<String>,

    /// Places the named view on a sphere about the subject's center, facing
    /// it, at an azimuth and elevation in degrees and a distance in meters or
    /// `fit`. Replaces the frame, position, and rotation. Repeatable.
    #[arg(
        value_names = ["view", "azimuth", "elevation", "distance"],
        long,
        num_args = 4,
        allow_negative_numbers = true,
        action = ArgAction::Append,
    )]
    view_orbit: Vec<String>,

    /// The named view's projection, `perspective` or `orthographic`,
    /// defaulting to `perspective`. Repeatable.
    #[arg(
        value_names = ["view", "projection"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    view_projection: Vec<String>,

    /// The named view's vertical field of view in degrees under
    /// `perspective`, defaulting to `35`. Repeatable.
    #[arg(
        value_names = ["view", "degrees"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    view_fov: Vec<String>,

    /// The world units across the shorter image axis under `orthographic`,
    /// or `fit`, defaulting to `fit`. Repeatable.
    #[arg(
        value_names = ["view", "scale"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    view_scale: Vec<String>,

    /// Narrows the named view's subject, which its `subject` frame and orbit
    /// are about, to the rendered objects a hierarchy-path glob matches.
    /// Repeatable, unioning the globs.
    #[arg(
        value_names = ["view", "select"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    view_select: Vec<String>,

    /// Declares the indexed light's kind, `directional`, `point`, or
    /// `hemisphere`. Lights number from `0` with no gaps. Any `--light`
    /// replaces a profile's rig whole. Repeatable.
    #[arg(
        value_names = ["light-index", "kind"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    light: Vec<String>,

    /// The frame the indexed light's transform is read in: `world`,
    /// `camera`, or `node` for a directional light, and `world`, `subject`,
    /// `camera`, or `node` for a point light. Repeatable.
    #[arg(
        value_names = ["light-index", "frame"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    light_frame: Vec<String>,

    /// The node path the indexed light's `node` frame reads its transform
    /// in: a glob over hierarchy node paths that matches exactly one.
    /// Repeatable.
    #[arg(
        value_names = ["light-index", "path"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    light_node: Vec<String>,

    /// The indexed point light's position in its frame, in meters.
    /// Repeatable.
    #[arg(
        value_names = ["light-index", "x", "y", "z"],
        long,
        num_args = 4,
        allow_negative_numbers = true,
        action = ArgAction::Append,
    )]
    light_position: Vec<String>,

    /// The indexed directional light's rotation as a unit quaternion.
    /// Repeatable.
    #[arg(
        value_names = ["light-index", "x", "y", "z", "w"],
        long,
        num_args = 5,
        allow_negative_numbers = true,
        action = ArgAction::Append,
    )]
    light_quaternion: Vec<String>,

    /// The indexed directional light's rotation as Euler angles in degrees
    /// about the fixed x, y, then z axes. Repeatable.
    #[arg(
        value_names = ["light-index", "x", "y", "z"],
        long,
        num_args = 4,
        allow_negative_numbers = true,
        action = ArgAction::Append,
    )]
    light_euler: Vec<String>,

    /// Aims the indexed directional light's -Z from its frame's origin at a
    /// point in its frame. Repeatable.
    #[arg(
        value_names = ["light-index", "x", "y", "z"],
        long,
        num_args = 4,
        allow_negative_numbers = true,
        action = ArgAction::Append,
    )]
    light_look_at: Vec<String>,

    /// The direction the indexed directional light shines from, as an
    /// azimuth from +Z toward +X and an elevation toward +Y, in degrees.
    /// Repeatable.
    #[arg(
        value_names = ["light-index", "azimuth", "elevation"],
        long,
        num_args = 3,
        allow_negative_numbers = true,
        action = ArgAction::Append,
    )]
    light_angles: Vec<String>,

    /// Places the indexed point light on a sphere about the subject's center
    /// at an azimuth and elevation in degrees and a distance in meters.
    /// Replaces the frame and position. Repeatable.
    #[arg(
        value_names = ["light-index", "azimuth", "elevation", "distance"],
        long,
        num_args = 4,
        allow_negative_numbers = true,
        action = ArgAction::Append,
    )]
    light_orbit: Vec<String>,

    /// The indexed light's shadow granularity, `none`, `per-pixel`,
    /// `per-face`, or `per-corner`, defaulting to `per-corner`. Repeatable.
    #[arg(
        value_names = ["light-index", "shadow"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    light_shadow: Vec<String>,

    /// The indexed directional or point light's color as a `#RRGGBB` hex,
    /// defaulting to white. Repeatable.
    #[arg(
        value_names = ["light-index", "color"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    light_color: Vec<String>,

    /// The strength scaling the indexed light's color, defaulting to `1`.
    /// Repeatable.
    #[arg(
        value_names = ["light-index", "strength"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    light_strength: Vec<String>,

    /// The distance the indexed point light reaches, in meters. Without it,
    /// the light has no cutoff. Repeatable.
    #[arg(
        value_names = ["light-index", "meters"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    light_range: Vec<String>,

    /// The indexed hemisphere light's color from above as a `#RRGGBB` hex,
    /// defaulting to white. Repeatable.
    #[arg(
        value_names = ["light-index", "color"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    light_sky: Vec<String>,

    /// The indexed hemisphere light's color from below as a `#RRGGBB` hex,
    /// defaulting to white. Repeatable.
    #[arg(
        value_names = ["light-index", "color"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    light_ground: Vec<String>,

    #[command(flatten)]
    selection: ObjectSelection,

    /// Prints each rendered view's resolved pose as its `--view-frame <view>
    /// world`, `--view-position`, and `--view-quaternion` flags on one line,
    /// ready to paste back.
    #[arg(value_name = "print-camera", long = "print-camera")]
    print_camera: bool,
}

impl ObjectRender {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let profiles = load_render_profile_set(&dependencies)?;

        let record = self.record(&profiles)?;

        let outputs = self
            .png_output()?
            .map(|output| view_outputs(&output, &record.views));

        let input = &self.input;

        let from = input.resolve_format()?;

        let main: VoxMain = load(&dependencies, from, &input.path)?;

        let object_ids = select_render_objects(&main, &self.selection)?;

        let rendered = render(&main, &object_ids, &record)?;

        if let Some(outputs) = &outputs {
            for (path, view) in outputs.iter().zip(rendered.iter()) {
                let png = encode_render_png(&VoxsmithDependenciesImpl, &view.image)?;

                WriteFile::write_file(&dependencies, path, &png)?;
            }
        }

        for (index, (view, rendered)) in record.views.iter().zip(rendered.iter()).enumerate() {
            if outputs.is_none() {
                if index > 0 {
                    dependencies.write_stdout(b"\n")?;
                }

                let image = &rendered.image;

                dependencies.display_image(image.width(), image.height(), &image.to_bytes())?;
            }

            if self.print_camera {
                dependencies.write_stdout(camera_line(&view.name, &rendered.view).as_bytes())?;
            }
        }

        Ok(())
    }

    /// The path one view writes under `--to png`: `--file-stem` or the
    /// input's stem, under `.png`, beside the input. `None` under the
    /// terminal, where a `--file-stem` errors.
    fn png_output(&self) -> Result<Option<PathBuf>> {
        match self.to.unwrap_or(OutputKind::Terminal) {
            OutputKind::Terminal => {
                if self.file_stem.is_some() {
                    return Err(Error::usage(
                        "--file-stem names the PNGs, and the output is the terminal; give --to png",
                    ));
                }

                Ok(None)
            }

            OutputKind::Png => {
                let mut output = self.input.output_path(None, "png");

                if let Some(stem) = &self.file_stem {
                    output.set_file_name(format!("{stem}.png"));
                }

                Ok(Some(output))
            }
        }
    }

    /// The record the flags and the profile stack lower into. A flag's
    /// element stands, and the stack fills the rest. A run with no view takes
    /// `hero`'s, and one with no rig takes `studio`'s.
    fn record(&self, profiles: &ProfileSet<RenderProfile>) -> Result<RenderRecord> {
        let mut stack = RenderProfileStack::new(profiles);

        stack.land_profiles("--profile", &self.profile)?;
        stack.land_view_sets("--views-from", &self.views_from)?;
        stack.land_rigs("--lights-from", &self.lights_from)?;

        let mut views = ViewTable::default();

        self.lower_views(&mut views)?;

        if views.is_empty() && !stack.has_views() {
            stack.land_view_sets("the default view", &["hero".to_owned()])?;
        }

        let declared = self.declared_lights()?;

        if declared.is_empty() && !stack.has_rig() {
            stack.land_rigs("the default rig", &["studio".to_owned()])?;
        }

        let stack = stack.finish();

        let mut lights = if declared.is_empty() {
            LightTable::from_rig(&stack.lights)
        } else {
            LightTable::declared(&declared)?
        };

        self.lower_lights(&mut lights)?;

        Ok(RenderRecord {
            width: self.width.or(stack.width).map_or(1024, u32::from),
            height: self.height.or(stack.height).map_or(1024, u32::from),
            background: self
                .background
                .or(stack.background)
                .and_then(Background::color),
            occlusion: self
                .occlusion
                .or(stack.occlusion.map(|occlusion| occlusion.0))
                .unwrap_or(RenderOcclusion::Corner),
            voxel_size: self
                .voxel_size
                .or(stack.voxel_size)
                .map_or(1.0, |size| size.0),
            views: views.finish(&stack.views)?,
            lights: lights.finish()?,
        })
    }

    /// Fills `views` from every flag that takes a view name.
    fn lower_views(&self, views: &mut ViewTable) -> Result<()> {
        for [name, frame] in flag_occurrences::<2>(&self.view_frame) {
            let flag = "--view-frame";
            let frame = parse_flag_value::<TransformFrame>(flag, frame)?;

            views.view(flag, name)?.set_frame(flag, frame)?;
        }

        for [name, path] in flag_occurrences::<2>(&self.view_node) {
            let flag = "--view-node";

            views.view(flag, name)?.set_node(flag, path.clone())?;
        }

        for [name, x, y, z] in flag_occurrences::<4>(&self.view_position) {
            let flag = "--view-position";
            let position = parse_flag_vector(flag, [x, y, z])?;

            views.view(flag, name)?.set_position(flag, position)?;
        }

        for [name, x, y, z, w] in flag_occurrences::<5>(&self.view_quaternion) {
            let flag = "--view-quaternion";
            let rotation = Rotation::Quaternion {
                value: parse_flag_quaternion(flag, [x, y, z, w])?,
            };

            views.view(flag, name)?.set_rotation(flag, rotation)?;
        }

        for [name, x, y, z] in flag_occurrences::<4>(&self.view_euler) {
            let flag = "--view-euler";
            let rotation = Rotation::Euler {
                value: parse_flag_vector(flag, [x, y, z])?,
                unit: TyAngleUnit::Degrees,
            };

            views.view(flag, name)?.set_rotation(flag, rotation)?;
        }

        for [name, x, y, z] in flag_occurrences::<4>(&self.view_look_at) {
            let flag = "--view-look-at";
            let rotation = Rotation::LookAt {
                target: Some(parse_flag_vector(flag, [x, y, z])?),
            };

            views.view(flag, name)?.set_rotation(flag, rotation)?;
        }

        for [name, azimuth, elevation] in flag_occurrences::<3>(&self.view_angles) {
            let flag = "--view-angles";
            let rotation = Rotation::Angles {
                azimuth: parse_flag_f64(flag, azimuth)?,
                elevation: parse_flag_f64(flag, elevation)?,
            };

            views.view(flag, name)?.set_rotation(flag, rotation)?;
        }

        for [name, azimuth, elevation, distance] in flag_occurrences::<4>(&self.view_orbit) {
            let flag = "--view-orbit";
            let orbit = PoseTransform::Orbit {
                azimuth: parse_flag_f64(flag, azimuth)?,
                elevation: parse_flag_f64(flag, elevation)?,
                distance: parse_fit_or_fixed(flag, distance)?,
            };

            views.view(flag, name)?.set_orbit(flag, orbit)?;
        }

        for [name, projection] in flag_occurrences::<2>(&self.view_projection) {
            let flag = "--view-projection";
            let projection = parse_flag_value::<ProjectionKind>(flag, projection)?;

            views.view(flag, name)?.set_projection(flag, projection)?;
        }

        for [name, fov] in flag_occurrences::<2>(&self.view_fov) {
            let flag = "--view-fov";
            let fov = parse_flag_from_str::<PositiveF64>(flag, fov)?.0;

            views.view(flag, name)?.set_fov(flag, fov)?;
        }

        for [name, scale] in flag_occurrences::<2>(&self.view_scale) {
            let flag = "--view-scale";
            let scale = parse_fit_or_fixed(flag, scale)?;

            views.view(flag, name)?.set_scale(flag, scale)?;
        }

        for [name, glob] in flag_occurrences::<2>(&self.view_select) {
            views.view("--view-select", name)?.push_select(glob.clone());
        }

        Ok(())
    }

    /// The `--light` declarations, each an index with its kind.
    fn declared_lights(&self) -> Result<Vec<(u32, LightKind)>> {
        flag_occurrences::<2>(&self.light)
            .map(|[index, kind]| {
                let flag = "--light";

                Ok((
                    parse_flag_index(flag, index)?,
                    parse_flag_value::<LightKind>(flag, kind)?,
                ))
            })
            .collect()
    }

    /// Fills `lights` from every flag that takes a light index.
    fn lower_lights(&self, lights: &mut LightTable) -> Result<()> {
        for [index, frame] in flag_occurrences::<2>(&self.light_frame) {
            let flag = "--light-frame";
            let index = parse_flag_index(flag, index)?;
            let frame = parse_flag_value::<TransformFrame>(flag, frame)?;

            lights.light(flag, index)?.set_frame(flag, frame)?;
        }

        for [index, path] in flag_occurrences::<2>(&self.light_node) {
            let flag = "--light-node";
            let index = parse_flag_index(flag, index)?;

            lights.light(flag, index)?.set_node(flag, path.clone())?;
        }

        for [index, x, y, z] in flag_occurrences::<4>(&self.light_position) {
            let flag = "--light-position";
            let index = parse_flag_index(flag, index)?;
            let position = parse_flag_vector(flag, [x, y, z])?;

            lights.light(flag, index)?.set_position(flag, position)?;
        }

        for [index, x, y, z, w] in flag_occurrences::<5>(&self.light_quaternion) {
            let flag = "--light-quaternion";
            let index = parse_flag_index(flag, index)?;
            let rotation = Rotation::Quaternion {
                value: parse_flag_quaternion(flag, [x, y, z, w])?,
            };

            lights.light(flag, index)?.set_rotation(flag, rotation)?;
        }

        for [index, x, y, z] in flag_occurrences::<4>(&self.light_euler) {
            let flag = "--light-euler";
            let index = parse_flag_index(flag, index)?;
            let rotation = Rotation::Euler {
                value: parse_flag_vector(flag, [x, y, z])?,
                unit: TyAngleUnit::Degrees,
            };

            lights.light(flag, index)?.set_rotation(flag, rotation)?;
        }

        for [index, x, y, z] in flag_occurrences::<4>(&self.light_look_at) {
            let flag = "--light-look-at";
            let index = parse_flag_index(flag, index)?;
            let rotation = Rotation::LookAt {
                target: Some(parse_flag_vector(flag, [x, y, z])?),
            };

            lights.light(flag, index)?.set_rotation(flag, rotation)?;
        }

        for [index, azimuth, elevation] in flag_occurrences::<3>(&self.light_angles) {
            let flag = "--light-angles";
            let index = parse_flag_index(flag, index)?;
            let rotation = Rotation::Angles {
                azimuth: parse_flag_f64(flag, azimuth)?,
                elevation: parse_flag_f64(flag, elevation)?,
            };

            lights.light(flag, index)?.set_rotation(flag, rotation)?;
        }

        for [index, azimuth, elevation, distance] in flag_occurrences::<4>(&self.light_orbit) {
            let flag = "--light-orbit";
            let index = parse_flag_index(flag, index)?;
            let orbit = PositionTransform::Orbit {
                azimuth: parse_flag_f64(flag, azimuth)?,
                elevation: parse_flag_f64(flag, elevation)?,
                distance: parse_flag_from_str::<PositiveF64>(flag, distance)?.0,
            };

            lights.light(flag, index)?.set_orbit(flag, orbit)?;
        }

        for [index, shadow] in flag_occurrences::<2>(&self.light_shadow) {
            let flag = "--light-shadow";
            let index = parse_flag_index(flag, index)?;
            let shadow = parse_flag_value::<RenderShadow>(flag, shadow)?;

            lights.light(flag, index)?.set_shadow(flag, shadow)?;
        }

        for [index, color] in flag_occurrences::<2>(&self.light_color) {
            let flag = "--light-color";
            let index = parse_flag_index(flag, index)?;
            let color = parse_flag_from_str::<SrgbColor>(flag, color)?.to_linear();

            lights.light(flag, index)?.set_color(flag, color)?;
        }

        for [index, strength] in flag_occurrences::<2>(&self.light_strength) {
            let flag = "--light-strength";
            let index = parse_flag_index(flag, index)?;
            let strength = parse_flag_from_str::<NonNegativeF64>(flag, strength)?.0;

            lights.light(flag, index)?.set_strength(flag, strength)?;
        }

        for [index, range] in flag_occurrences::<2>(&self.light_range) {
            let flag = "--light-range";
            let index = parse_flag_index(flag, index)?;
            let range = parse_flag_from_str::<PositiveF64>(flag, range)?.0;

            lights.light(flag, index)?.set_range(flag, range)?;
        }

        for [index, sky] in flag_occurrences::<2>(&self.light_sky) {
            let flag = "--light-sky";
            let index = parse_flag_index(flag, index)?;
            let sky = parse_flag_from_str::<SrgbColor>(flag, sky)?.to_linear();

            lights.light(flag, index)?.set_sky(flag, sky)?;
        }

        for [index, ground] in flag_occurrences::<2>(&self.light_ground) {
            let flag = "--light-ground";
            let index = parse_flag_index(flag, index)?;
            let ground = parse_flag_from_str::<SrgbColor>(flag, ground)?.to_linear();

            lights.light(flag, index)?.set_ground(flag, ground)?;
        }

        Ok(())
    }
}

/// The file each view of `views` writes: `output` alone for one view, else
/// `output`'s stem, a hyphen, and the view's name under `output`'s extension
/// beside it.
fn view_outputs(
    output: &Path,
    views: &IdVec<BRenderView, ViewRecord>,
) -> IdVec<BRenderView, PathBuf> {
    if views.len() == 1 {
        return IdVec::from(vec![output.to_path_buf()]);
    }

    let stem = output
        .file_stem()
        .expect("the output path carries a file name")
        .to_string_lossy();

    views
        .iter()
        .map(|view| {
            let mut file_name = format!("{stem}-{}", view.name);

            if let Some(extension) = output.extension() {
                file_name = format!("{file_name}.{}", extension.to_string_lossy());
            }

            output.with_file_name(file_name)
        })
        .collect()
}

/// The resolved pose of the view `name` as the flags that reproduce it, on
/// one line.
fn camera_line(name: &str, view: &RenderView) -> String {
    let position = view.pose.position;
    let rotation = view.pose.rotation;

    format!(
        "--view-frame {name} world --view-position {name} {} {} {} --view-quaternion {name} {} {} \
         {} {}\n",
        position.x, position.y, position.z, rotation.x, rotation.y, rotation.z, rotation.w
    )
}

/// The objects `selection` resolves to in `main`, in document order. A
/// document holding none is a usage error. `resolve` already rejects a
/// selector matching nothing.
fn select_render_objects<T: VoxExt>(
    main: &VoxMain<T>,
    selection: &ObjectSelection,
) -> Result<Vec<U32Id<BVoxObject>>> {
    let object_ids = selection.resolve(main)?;

    if object_ids.is_empty() {
        return Err(Error::usage("the document has no objects to render"));
    }

    Ok(object_ids)
}

/// Parses a token of `flag` through its `FromStr`.
fn parse_flag_from_str<T: FromStr<Err: Display>>(flag: &str, text: &str) -> Result<T> {
    text.parse()
        .map_err(|reason| Error::usage(format!("{flag}: {reason}")))
}

/// Parses a finite number token of `flag`.
fn parse_flag_f64(flag: &str, text: &str) -> Result<f64> {
    let number = text
        .parse::<f64>()
        .map_err(|_| Error::usage(format!("{flag} takes a number, not `{text}`")))?;

    if !number.is_finite() {
        return Err(Error::usage(format!(
            "{flag} takes a finite number, not `{text}`"
        )));
    }

    Ok(number)
}

/// Parses the three number tokens of `flag` as a vector.
fn parse_flag_vector(flag: &str, [x, y, z]: [&String; 3]) -> Result<TyVector3F64> {
    Ok(TyVector3F64::new(
        parse_flag_f64(flag, x)?,
        parse_flag_f64(flag, y)?,
        parse_flag_f64(flag, z)?,
    ))
}

/// Parses the four number tokens of `flag` as a unit quaternion.
fn parse_flag_quaternion(flag: &str, [x, y, z, w]: [&String; 4]) -> Result<TyQuaternionF64> {
    let quaternion = TyQuaternionF64::from_xyzw(
        parse_flag_f64(flag, x)?,
        parse_flag_f64(flag, y)?,
        parse_flag_f64(flag, z)?,
        parse_flag_f64(flag, w)?,
    );

    if !quaternion.is_normalized() {
        return Err(Error::usage(format!(
            "{flag} takes a unit quaternion, and `{x} {y} {z} {w}` has a length of {}",
            quaternion.length()
        )));
    }

    Ok(quaternion)
}

/// Parses a length token of `flag`: `fit` or a positive number.
fn parse_fit_or_fixed(flag: &str, text: &str) -> Result<FitOrFixed> {
    if text == "fit" {
        return Ok(FitOrFixed::Fit);
    }

    let length = text.parse::<PositiveF64>().map_err(|_| {
        Error::usage(format!(
            "{flag} takes `fit` or a positive number, not `{text}`"
        ))
    })?;

    Ok(FitOrFixed::Fixed(length.0))
}

#[cfg(test)]
mod tests {
    use crate::{
        ProfileSet, Result,
        commands::{
            ObjectRender, RenderProfile, built_in_render_profiles,
            object::object_render::object_render::{camera_line, view_outputs},
        },
    };
    use branded_id::IdVec;
    use clap::Parser;
    use std::{
        collections::BTreeMap,
        path::{Path, PathBuf},
    };
    use ty_math::{TyAngleUnit, TyLinSrgbF64, TyPoseF64, TyQuaternionF64, TySrgbU8, TyVector3F64};
    use voxsmith::operations::object::{
        FitOrFixed, LightRecord, PoseTransform, PositionTransform, RenderOcclusion,
        RenderProjection, RenderRecord, RenderShadow, RenderView, Rotation, RotationTransform,
        ViewProjection, ViewRecord,
    };

    /// The command parsed from `args` after the input.
    fn parse(args: &[&str]) -> ObjectRender {
        let mut argv = vec!["render", "model.voxj"];
        argv.extend_from_slice(args);
        ObjectRender::try_parse_from(argv).unwrap()
    }

    /// The record `args` lower into over `profiles`, or the error they raise.
    fn try_record_over(
        profiles: &ProfileSet<RenderProfile>,
        args: &[&str],
    ) -> Result<RenderRecord> {
        parse(args).record(profiles)
    }

    /// The record `args` lower into over the built-ins, or the error they
    /// raise.
    fn try_record(args: &[&str]) -> Result<RenderRecord> {
        try_record_over(&ProfileSet::layered(built_in_render_profiles(), []), args)
    }

    /// The built-ins under one `.vxlconfig` layer holding the profiles
    /// `entries` defines as json.
    fn built_ins_under(entries: &[(&str, &str)]) -> ProfileSet<RenderProfile> {
        let layer: BTreeMap<String, RenderProfile> = entries
            .iter()
            .map(|(name, json)| ((*name).to_owned(), serde_json::from_str(json).unwrap()))
            .collect();

        ProfileSet::layered(
            built_in_render_profiles(),
            [(PathBuf::from("/repo/.vxlconfig"), layer)],
        )
    }

    /// The record `args` lower into.
    fn record(args: &[&str]) -> RenderRecord {
        try_record(args).unwrap()
    }

    /// The error `args` raise.
    fn error_of(args: &[&str]) -> String {
        try_record(args).unwrap_err().to_string()
    }

    /// The one view of `record`.
    fn view(record: &RenderRecord) -> &ViewRecord {
        let [view] = record.views.as_slice() else {
            panic!("one view, got {:?}", record.views);
        };
        view
    }

    /// The one light of `record`.
    fn light(record: &RenderRecord) -> &LightRecord {
        let [light] = record.lights.as_slice() else {
            panic!("one light, got {:?}", record.lights);
        };
        light
    }

    /// Whether `a` and `b` are the same color to a rounding.
    fn same_color(a: TyLinSrgbF64, b: TyLinSrgbF64) -> bool {
        (a.red - b.red).abs() < 1e-6
            && (a.green - b.green).abs() < 1e-6
            && (a.blue - b.blue).abs() < 1e-6
    }

    #[test]
    fn the_bare_record_renders_hero_under_studio_at_the_defaults() {
        let record = record(&[]);

        assert_eq!((record.width, record.height), (1024, 1024));
        assert_eq!(record.background, None);
        assert_eq!(record.occlusion, RenderOcclusion::Corner);
        assert_eq!(record.voxel_size, 1.0);

        let hero = view(&record);
        assert_eq!(hero.name, "hero");
        assert_eq!(
            hero.transform,
            PoseTransform::Orbit {
                azimuth: 45.0,
                elevation: 30.0,
                distance: FitOrFixed::Fit,
            }
        );
        assert_eq!(hero.projection, ViewProjection::Perspective { fov: 35.0 });
        assert!(hero.select.is_empty());

        let [key, fill] = record.lights.as_slice() else {
            panic!("studio holds two lights");
        };
        assert!(matches!(
            key,
            LightRecord::Directional {
                transform: RotationTransform::Camera {
                    rotation: Rotation::Angles {
                        azimuth: -30.0,
                        elevation: 30.0
                    }
                },
                shadow: RenderShadow::PerCorner,
                ..
            }
        ));
        assert!(matches!(fill, LightRecord::Hemisphere { strength, .. } if *strength == 1.0));
    }

    #[test]
    fn the_image_flags_lower_into_the_record() {
        let record = record(&[
            "--width",
            "8",
            "--height",
            "4",
            "--background",
            "#010203",
            "--occlusion",
            "none",
            "--voxel-size",
            "0.5",
        ]);

        assert_eq!((record.width, record.height), (8, 4));
        assert_eq!(record.background, Some(TySrgbU8::new(1, 2, 3)));
        assert_eq!(record.occlusion, RenderOcclusion::None);
        assert_eq!(record.voxel_size, 0.5);

        assert!(ObjectRender::try_parse_from(["render", "m.voxj", "--width", "0"]).is_err());
        assert!(ObjectRender::try_parse_from(["render", "m.voxj", "--background", "red"]).is_err());
    }

    #[test]
    fn a_profile_stack_and_the_halves_set_the_views_and_the_rig() {
        let stacked = record(&["--profile", "front", "--profile", "flat"]);
        assert_eq!(view(&stacked).name, "front");
        assert!(matches!(
            light(&stacked),
            LightRecord::Directional {
                shadow: RenderShadow::None,
                ..
            }
        ));

        let halves = record(&["--views-from", "turnaround", "--lights-from", "studio"]);
        let names: Vec<_> = halves.views.iter().map(|view| view.name.as_str()).collect();
        assert_eq!(names, ["back", "front", "hero", "left", "right"]);
        assert_eq!(halves.lights.len(), 2);

        // A profile that sets a rig alone still takes the default view.
        let rig_alone = record(&["--profile", "flat"]);
        assert_eq!(view(&rig_alone).name, "hero");

        assert!(error_of(&["--profile", "hero", "--profile", "hero"]).contains("twice"));
        assert!(error_of(&["--profile", "nowhere"]).contains("`nowhere`"));
    }

    #[test]
    fn a_posed_view_takes_a_frame_a_position_and_one_rotation() {
        let posed = |rotation: &[&str]| {
            let mut args = vec![
                "--view-frame",
                "cam",
                "world",
                "--view-position",
                "cam",
                "1",
                "-2",
                "3",
            ];
            args.extend_from_slice(rotation);
            record(&args)
        };

        let expected = |rotation: Rotation| PoseTransform::World {
            position: TyVector3F64::new(1.0, -2.0, 3.0),
            rotation,
        };

        assert_eq!(
            view(&posed(&["--view-quaternion", "cam", "0", "0", "0", "1"])).transform,
            expected(Rotation::Quaternion {
                value: TyQuaternionF64::IDENTITY
            })
        );
        assert_eq!(
            view(&posed(&["--view-euler", "cam", "0", "-90", "0"])).transform,
            expected(Rotation::Euler {
                value: TyVector3F64::new(0.0, -90.0, 0.0),
                unit: TyAngleUnit::Degrees,
            })
        );
        assert_eq!(
            view(&posed(&["--view-look-at", "cam", "0", "0", "0"])).transform,
            expected(Rotation::LookAt {
                target: Some(TyVector3F64::ZERO)
            })
        );
        assert_eq!(
            view(&posed(&["--view-angles", "cam", "45", "-10"])).transform,
            expected(Rotation::Angles {
                azimuth: 45.0,
                elevation: -10.0,
            })
        );

        let subject = record(&[
            "--view-frame",
            "cam",
            "subject",
            "--view-position",
            "cam",
            "0",
            "0",
            "5",
            "--view-look-at",
            "cam",
            "0",
            "0",
            "0",
        ]);
        assert!(matches!(
            view(&subject).transform,
            PoseTransform::Subject { .. }
        ));

        let lacking = error_of(&["--view-frame", "cam", "world"]);
        assert!(
            lacking.contains("view `cam`'s transform lacks --view-position and a rotation flag"),
            "{lacking}"
        );

        let camera = error_of(&[
            "--view-frame",
            "cam",
            "camera",
            "--view-position",
            "cam",
            "0",
            "0",
            "0",
            "--view-angles",
            "cam",
            "0",
            "0",
        ]);
        assert!(
            camera.contains("a view's frame is world, subject, or node"),
            "{camera}"
        );

        let twice = error_of(&[
            "--view-frame",
            "cam",
            "world",
            "--view-position",
            "cam",
            "0",
            "0",
            "0",
            "--view-angles",
            "cam",
            "0",
            "0",
            "--view-euler",
            "cam",
            "0",
            "0",
            "0",
        ]);
        assert!(
            twice.contains("sets view `cam`'s rotation, which is set already"),
            "{twice}"
        );

        let unnormalized = error_of(&["--view-quaternion", "cam", "1", "1", "0", "0"]);
        assert!(unnormalized.contains("unit quaternion"), "{unnormalized}");
    }

    #[test]
    fn an_orbit_sets_the_whole_transform() {
        let fit = record(&["--view-orbit", "cam", "10", "20", "fit"]);
        assert_eq!(
            view(&fit).transform,
            PoseTransform::Orbit {
                azimuth: 10.0,
                elevation: 20.0,
                distance: FitOrFixed::Fit,
            }
        );

        let fixed = record(&["--view-orbit", "cam", "-10", "20", "5"]);
        assert!(matches!(
            view(&fixed).transform,
            PoseTransform::Orbit {
                distance: FitOrFixed::Fixed(distance),
                ..
            } if distance == 5.0
        ));

        let clash = error_of(&[
            "--view-orbit",
            "cam",
            "0",
            "0",
            "fit",
            "--view-frame",
            "cam",
            "world",
        ]);
        assert!(
            clash.contains("sets view `cam`'s transform, which"),
            "{clash}"
        );

        let near = error_of(&["--view-orbit", "cam", "0", "0", "near"]);
        assert!(near.contains("`fit` or a positive number"), "{near}");
    }

    #[test]
    fn a_node_frame_takes_a_path_and_another_frame_refuses_one() {
        let riding = record(&[
            "--view-frame",
            "cam",
            "node",
            "--view-node",
            "cam",
            "player/head",
            "--view-position",
            "cam",
            "0",
            "1",
            "5",
            "--view-look-at",
            "cam",
            "0",
            "1",
            "0",
        ]);
        assert_eq!(
            view(&riding).transform,
            PoseTransform::Node {
                path: "player/head".to_owned(),
                position: TyVector3F64::new(0.0, 1.0, 5.0),
                rotation: Rotation::LookAt {
                    target: Some(TyVector3F64::new(0.0, 1.0, 0.0)),
                },
            }
        );

        let pathless = error_of(&[
            "--view-frame",
            "cam",
            "node",
            "--view-position",
            "cam",
            "0",
            "0",
            "0",
            "--view-angles",
            "cam",
            "0",
            "0",
        ]);
        assert!(
            pathless.contains("view `cam`'s frame is node, and its transform lacks --view-node"),
            "{pathless}"
        );

        let misframed = error_of(&[
            "--view-frame",
            "cam",
            "world",
            "--view-node",
            "cam",
            "player",
            "--view-position",
            "cam",
            "0",
            "0",
            "0",
            "--view-angles",
            "cam",
            "0",
            "0",
        ]);
        assert!(
            misframed.contains(
                "view `cam` reads the node path `player`, which applies under the node frame, \
                 and its frame is world"
            ),
            "{misframed}"
        );

        // A path poses the view, so it clashes with an orbit and, alone, lacks
        // the rest of the pose.
        let orbited = error_of(&[
            "--view-orbit",
            "cam",
            "0",
            "0",
            "fit",
            "--view-node",
            "cam",
            "player",
        ]);
        assert!(
            orbited.contains("sets view `cam`'s transform, which"),
            "{orbited}"
        );

        let alone = error_of(&["--profile", "hero", "--view-node", "hero", "player"]);
        assert!(
            alone.contains(
                "view `hero`'s transform lacks --view-frame and --view-position and a rotation \
                 flag"
            ),
            "{alone}"
        );

        let sun = record(&[
            "--light",
            "0",
            "directional",
            "--light-frame",
            "0",
            "node",
            "--light-node",
            "0",
            "lamp",
            "--light-angles",
            "0",
            "0",
            "90",
        ]);
        assert!(matches!(
            light(&sun),
            LightRecord::Directional {
                transform: RotationTransform::Node { path, .. },
                ..
            } if path == "lamp"
        ));

        let lamp = record(&[
            "--light",
            "0",
            "point",
            "--light-frame",
            "0",
            "node",
            "--light-node",
            "0",
            "lamp",
            "--light-position",
            "0",
            "0",
            "1",
            "0",
        ]);
        assert!(matches!(
            light(&lamp),
            LightRecord::Point {
                transform: PositionTransform::Node { path, position },
                ..
            } if path == "lamp" && *position == TyVector3F64::Y
        ));

        let error = error_of(&[
            "--light",
            "0",
            "point",
            "--light-frame",
            "0",
            "node",
            "--light-position",
            "0",
            "0",
            "0",
            "0",
        ]);
        assert!(
            error.contains("light 0's frame is node, and its transform lacks --light-node"),
            "{error}"
        );

        let error = error_of(&[
            "--light",
            "0",
            "directional",
            "--light-frame",
            "0",
            "camera",
            "--light-node",
            "0",
            "lamp",
            "--light-angles",
            "0",
            "0",
            "0",
        ]);
        assert!(
            error.contains(
                "light 0 reads the node path `lamp`, which applies under the node frame, and \
                 its frame is camera"
            ),
            "{error}"
        );

        let error = error_of(&["--light", "0", "hemisphere", "--light-node", "0", "lamp"]);
        assert!(
            error.contains("--light-node applies to a directional or point light"),
            "{error}"
        );
    }

    #[test]
    fn the_projection_reads_one_length() {
        let wide = record(&["--profile", "hero", "--view-fov", "hero", "50"]);
        assert_eq!(
            view(&wide).projection,
            ViewProjection::Perspective { fov: 50.0 }
        );

        let plan = record(&[
            "--profile",
            "top",
            "--view-projection",
            "top",
            "orthographic",
            "--view-scale",
            "top",
            "12",
        ]);
        assert_eq!(
            view(&plan).projection,
            ViewProjection::Orthographic {
                scale: FitOrFixed::Fixed(12.0)
            }
        );

        let fit = record(&[
            "--profile",
            "top",
            "--view-projection",
            "top",
            "orthographic",
        ]);
        assert_eq!(
            view(&fit).projection,
            ViewProjection::Orthographic {
                scale: FitOrFixed::Fit
            }
        );

        let error = error_of(&["--profile", "top", "--view-scale", "top", "12"]);
        assert!(error.contains("applies under orthographic"), "{error}");

        let error = error_of(&[
            "--profile",
            "top",
            "--view-projection",
            "top",
            "orthographic",
            "--view-fov",
            "top",
            "50",
        ]);
        assert!(error.contains("applies under perspective"), "{error}");

        let error = error_of(&["--profile", "hero", "--view-fov", "hero", "180"]);
        assert!(error.contains("less than 180"), "{error}");
    }

    #[test]
    fn a_select_unions_its_globs_and_a_flag_stands_over_the_profile_s() {
        let profiles = built_ins_under(&[(
            "house",
            r#"{ "views": { "cam": { "transform": { "kind": "orbit", "azimuth": 0, "elevation": 0 }, "select": ["house/**"] } } }"#,
        )]);

        let from_profile = try_record_over(&profiles, &["--profile", "house"]).unwrap();
        assert_eq!(view(&from_profile).select, ["house/**"]);

        let from_flags = try_record_over(
            &profiles,
            &[
                "--profile",
                "house",
                "--view-select",
                "cam",
                "barn/**",
                "--view-select",
                "cam",
                "silo",
            ],
        )
        .unwrap();
        assert_eq!(view(&from_flags).select, ["barn/**", "silo"]);
    }

    #[test]
    fn a_view_needs_a_transform_and_a_file_safe_name() {
        let error = error_of(&["--view-fov", "cam", "50"]);
        assert!(error.contains("view `cam` has no transform"), "{error}");

        let error = error_of(&["--view-orbit", "", "0", "0", "fit"]);
        assert!(error.contains("empty name"), "{error}");

        let error = error_of(&["--view-orbit", "a/b", "0", "0", "fit"]);
        assert!(error.contains("path separator"), "{error}");

        let profiles = built_ins_under(&[(
            "bad",
            r#"{ "views": { "a/b": { "transform": { "kind": "orbit", "azimuth": 0, "elevation": 0 } } } }"#,
        )]);
        let error = try_record_over(&profiles, &["--profile", "bad"])
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("the profile stack names the view `a/b`"),
            "{error}"
        );
    }

    #[test]
    fn the_light_flags_declare_a_rig_of_each_kind() {
        let sun = record(&[
            "--light",
            "0",
            "directional",
            "--light-frame",
            "0",
            "camera",
            "--light-angles",
            "0",
            "-30",
            "30",
            "--light-shadow",
            "0",
            "per-face",
            "--light-color",
            "0",
            "#FF0000",
            "--light-strength",
            "0",
            "2",
        ]);
        let LightRecord::Directional {
            transform,
            shadow,
            color,
            strength,
        } = light(&sun)
        else {
            panic!("a directional light");
        };
        assert_eq!(
            *transform,
            RotationTransform::Camera {
                rotation: Rotation::Angles {
                    azimuth: -30.0,
                    elevation: 30.0,
                }
            }
        );
        assert_eq!(*shadow, RenderShadow::PerFace);
        assert!(same_color(*color, TyLinSrgbF64::new(1.0, 0.0, 0.0)));
        assert_eq!(*strength, 2.0);

        let lamp = record(&[
            "--light",
            "0",
            "point",
            "--light-orbit",
            "0",
            "0",
            "45",
            "4",
            "--light-range",
            "0",
            "10",
        ]);
        let LightRecord::Point {
            transform,
            shadow,
            color,
            strength,
            range,
        } = light(&lamp)
        else {
            panic!("a point light");
        };
        assert_eq!(
            *transform,
            PositionTransform::Orbit {
                azimuth: 0.0,
                elevation: 45.0,
                distance: 4.0,
            }
        );
        assert_eq!(*shadow, RenderShadow::PerCorner);
        assert!(same_color(*color, TyLinSrgbF64::new(1.0, 1.0, 1.0)));
        assert_eq!(*strength, 1.0);
        assert_eq!(*range, Some(10.0));

        let posed = record(&[
            "--light",
            "0",
            "point",
            "--light-frame",
            "0",
            "subject",
            "--light-position",
            "0",
            "1",
            "2",
            "3",
        ]);
        assert!(matches!(
            light(&posed),
            LightRecord::Point {
                transform: PositionTransform::Subject { position },
                range: None,
                ..
            } if *position == TyVector3F64::new(1.0, 2.0, 3.0)
        ));

        let sky = record(&[
            "--light",
            "0",
            "hemisphere",
            "--light-sky",
            "0",
            "#000000",
            "--light-ground",
            "0",
            "#FFFFFF",
            "--light-strength",
            "0",
            "0.5",
        ]);
        let LightRecord::Hemisphere {
            sky,
            ground,
            strength,
        } = light(&sky)
        else {
            panic!("a hemisphere light");
        };
        assert!(same_color(*sky, TyLinSrgbF64::new(0.0, 0.0, 0.0)));
        assert!(same_color(*ground, TyLinSrgbF64::new(1.0, 1.0, 1.0)));
        assert_eq!(*strength, 0.5);
    }

    #[test]
    fn a_light_declaration_replaces_the_rig_and_an_index_flag_fills_it() {
        let replaced = record(&["--profile", "studio", "--light", "0", "hemisphere"]);
        assert!(matches!(light(&replaced), LightRecord::Hemisphere { .. }));

        let filled = record(&["--light-shadow", "0", "none", "--lights-from", "studio"]);
        assert!(matches!(
            &filled.lights.as_slice()[0],
            LightRecord::Directional {
                shadow: RenderShadow::None,
                ..
            }
        ));

        // With no rig the default one takes the index flags.
        let default = record(&["--light-strength", "1", "2"]);
        assert!(matches!(
            &default.lights.as_slice()[1],
            LightRecord::Hemisphere { strength, .. } if *strength == 2.0
        ));

        let error = error_of(&["--light", "1", "point"]);
        assert!(
            error.contains("declares light 1 but not light 0"),
            "{error}"
        );

        let error = error_of(&["--light", "0", "point", "--light", "0", "hemisphere"]);
        assert!(error.contains("declares light 0 twice"), "{error}");

        let error = error_of(&["--light-shadow", "5", "none"]);
        assert!(error.contains("the rig holds lights 0 to 1"), "{error}");
    }

    #[test]
    fn a_light_takes_the_elements_its_kind_reads() {
        let error = error_of(&["--light", "0", "directional", "--light-range", "0", "1"]);
        assert!(
            error.contains("--light-range applies to a point light, and light 0 is directional"),
            "{error}"
        );

        let error = error_of(&[
            "--light",
            "0",
            "hemisphere",
            "--light-color",
            "0",
            "#FFFFFF",
        ]);
        assert!(
            error.contains("applies to a directional or point light"),
            "{error}"
        );

        let error = error_of(&["--light", "0", "point", "--light-angles", "0", "0", "0"]);
        assert!(error.contains("applies to a directional light"), "{error}");

        let error = error_of(&["--light", "0", "point", "--light-sky", "0", "#FFFFFF"]);
        assert!(error.contains("applies to a hemisphere light"), "{error}");

        let error = error_of(&[
            "--light",
            "0",
            "directional",
            "--light-frame",
            "0",
            "subject",
            "--light-angles",
            "0",
            "0",
            "0",
        ]);
        assert!(
            error.contains("a directional light's frame is world, camera, or node"),
            "{error}"
        );

        let error = error_of(&["--light", "0", "directional"]);
        assert!(error.contains("light 0 has no transform"), "{error}");

        let error = error_of(&["--light", "0", "point", "--light-frame", "0", "world"]);
        assert!(error.contains("lacks --light-position"), "{error}");

        let error = error_of(&[
            "--light",
            "0",
            "point",
            "--light-orbit",
            "0",
            "0",
            "0",
            "1",
            "--light-position",
            "0",
            "0",
            "0",
            "0",
        ]);
        assert!(error.contains("sets light 0's transform, which"), "{error}");

        let error = error_of(&[
            "--light",
            "0",
            "point",
            "--light-orbit",
            "0",
            "0",
            "0",
            "1",
            "--light-strength",
            "0",
            "1",
            "--light-strength",
            "0",
            "2",
        ]);
        assert!(
            error.contains("sets light 0's strength, which is set already"),
            "{error}"
        );
    }

    #[test]
    fn several_views_write_beside_the_output_under_their_names() {
        let views = |names: &[&str]| -> IdVec<_, _> {
            names
                .iter()
                .map(|name| ViewRecord {
                    name: (*name).to_owned(),
                    transform: PoseTransform::Orbit {
                        azimuth: 0.0,
                        elevation: 0.0,
                        distance: FitOrFixed::Fit,
                    },
                    projection: ViewProjection::Perspective { fov: 35.0 },
                    select: Vec::new(),
                })
                .collect()
        };

        let one = view_outputs(Path::new("out/model.png"), &views(&["hero"]));
        assert_eq!(one.as_slice(), [PathBuf::from("out/model.png")]);

        let two = view_outputs(Path::new("out/model.png"), &views(&["front", "hero"]));
        assert_eq!(
            two.as_slice(),
            [
                PathBuf::from("out/model-front.png"),
                PathBuf::from("out/model-hero.png")
            ]
        );
    }

    #[test]
    fn the_output_defaults_to_the_terminal_and_png_writes_beside_the_input() {
        assert_eq!(parse(&[]).png_output().unwrap(), None);
        assert_eq!(parse(&["--to", "terminal"]).png_output().unwrap(), None);
        assert_eq!(
            parse(&["--to", "png"]).png_output().unwrap(),
            Some(PathBuf::from("model.png"))
        );
        assert_eq!(
            parse(&["--to", "png", "--file-stem", "shot"])
                .png_output()
                .unwrap(),
            Some(PathBuf::from("shot.png"))
        );

        let error = parse(&["--file-stem", "shot"])
            .png_output()
            .unwrap_err()
            .to_string();
        assert!(error.contains("give --to png"), "{error}");

        assert!(ObjectRender::try_parse_from(["render", "model.voxj", "--to", "jpeg"]).is_err());
    }

    #[test]
    fn the_camera_line_pastes_back_as_flags() {
        let view = RenderView {
            pose: TyPoseF64::new(TyVector3F64::new(1.0, 2.5, -3.0), TyQuaternionF64::IDENTITY),
            projection: RenderProjection::Perspective { fov: 1.0 },
        };

        assert_eq!(
            camera_line("hero", &view),
            "--view-frame hero world --view-position hero 1 2.5 -3 --view-quaternion hero 0 0 0 \
             1\n"
        );

        assert!(parse(&["--print-camera"]).print_camera);
    }
}
