use crate::{
    Dependencies, Error, MeshInput, MeshOutput, MeshRequest, MeshTaskFile, Model, ModelType,
    OutputMode, Result, TargetFormat, TextureQuality, Topology,
    commands::{
        WaitArgs,
        shared::{absolute, finish_task, parent_dir, relative, wait_for_task, with_suffix},
    },
};
use clap::{ArgAction, Parser};
use std::path::PathBuf;

/// Generates a 3D mesh from an image using the Meshy [Image to 3D](https://docs.meshy.ai/en/api/image-to-3d) API.
///
/// On a successful create, writes `<output-base>.meshy.mesh.json` and prints the
/// task id. With `--wait`, blocks until the task completes and downloads its
/// files; otherwise fetch them later with `tyt meshy poll`.
#[derive(Clone, Debug, Parser)]
#[command(name = "mesh")]
pub struct Mesh {
    /// The input image, relative to the current directory.
    #[arg(value_name = "image")]
    image: PathBuf,

    /// Base path prefixing every output file (e.g. `<output-base>.usdz`),
    /// relative to the current directory. Defaults to the image's base name.
    #[arg(value_name = "output-base")]
    output_base: Option<PathBuf>,

    /// The generation pipeline. `lowpoly` ignores `--model` and the remesh
    /// options.
    #[arg(value_name = "model-type", long = "model-type", value_enum, default_value_t = ModelType::Standard)]
    model_type: ModelType,

    /// The model to use. Ignored with `--model-type lowpoly`.
    #[arg(value_name = "model", long, value_enum)]
    model: Option<Model>,

    /// Whether to texture the model.
    #[arg(
        value_name = "texture",
        long,
        action = ArgAction::Set,
        num_args = 0..=1,
        require_equals = true,
        default_value_t = false,
        default_missing_value = "true",
    )]
    texture: bool,

    /// Generate PBR maps (metallic, roughness, normal) alongside the base color.
    /// Requires `--texture`.
    #[arg(
        value_name = "texture-gen-pbr",
        long = "texture-gen-pbr",
        action = ArgAction::Set,
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true",
    )]
    texture_gen_pbr: Option<bool>,

    /// The base color texture quality. `hd` requires Meshy 6. Requires
    /// `--texture`.
    #[arg(value_name = "texture-quality", long = "texture-quality", value_enum)]
    texture_quality: Option<TextureQuality>,

    /// A text prompt guiding texturing (max 600 characters). Requires
    /// `--texture`.
    #[arg(value_name = "texture-prompt", long = "texture-prompt")]
    texture_prompt: Option<String>,

    /// A file holding the texturing prompt, relative to the current directory
    /// (max 600 characters). Requires `--texture`.
    #[arg(value_name = "texture-prompt-file", long = "texture-prompt-file")]
    texture_prompt_file: Option<PathBuf>,

    /// An image guiding texturing, relative to the current directory. Requires
    /// `--texture`.
    #[arg(value_name = "texture-image", long = "texture-image")]
    texture_image: Option<PathBuf>,

    /// Whether to run the remesh phase. Ignored with `--model-type lowpoly`.
    #[arg(
        value_name = "remesh",
        long,
        action = ArgAction::Set,
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true",
    )]
    remesh: Option<bool>,

    /// The topology of the generated model. Requires `--remesh`.
    #[arg(value_name = "topology", long, value_enum)]
    topology: Option<Topology>,

    /// The target polygon count (100–300000). Requires `--remesh`; cannot be
    /// combined with `--decimation-mode`.
    #[arg(value_name = "target-polycount", long = "target-polycount", value_parser = clap::value_parser!(u32).range(100..=300_000))]
    target_polycount: Option<u32>,

    /// Adaptive decimation level: 1 ultra, 2 high, 3 medium, 4 low. Requires
    /// `--remesh`; cannot be combined with `--target-polycount`.
    #[arg(value_name = "decimation-mode", long = "decimation-mode", value_parser = clap::value_parser!(u8).range(1..=4))]
    decimation_mode: Option<u8>,

    /// Also save the GLB captured before the remesh phase. Requires `--remesh`.
    #[arg(
        value_name = "save-pre-remeshed-model",
        long = "save-pre-remeshed-model",
        action = ArgAction::Set,
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true",
    )]
    save_pre_remeshed_model: Option<bool>,

    /// Optimize the input image for better results. Only supported on Meshy 6;
    /// defaults to true there.
    #[arg(
        value_name = "image-enhancement",
        long = "image-enhancement",
        action = ArgAction::Set,
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true",
    )]
    image_enhancement: Option<bool>,

    /// Keep the input's highlights and shadows baked into the base color
    /// texture. Only supported on Meshy 6; defaults to false there.
    #[arg(
        value_name = "keep-lighting",
        long = "keep-lighting",
        action = ArgAction::Set,
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true",
    )]
    keep_lighting: Option<bool>,

    /// A 3D file format to generate. Repeatable; defaults to `usdz`.
    #[arg(value_name = "target-format", long = "target-format", value_enum)]
    target_format: Vec<TargetFormat>,

    /// Render the four cardinal-view thumbnails (front, right, back, left)
    /// instead of just one. Adds roughly 3 seconds of latency.
    #[arg(
        value_name = "multi-view-thumbnails",
        long = "multi-view-thumbnails",
        action = ArgAction::Set,
        num_args = 0..=1,
        require_equals = true,
        default_value_t = true,
        default_missing_value = "true",
    )]
    multi_view_thumbnails: bool,

    /// How much to print.
    #[arg(value_name = "output", long, value_enum, default_value_t = OutputMode::AllThumbnails)]
    output: OutputMode,

    #[command(flatten)]
    wait: WaitArgs,
}

impl Mesh {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Mesh {
            image,
            output_base,
            model_type,
            model,
            texture,
            texture_gen_pbr,
            texture_quality,
            texture_prompt,
            texture_prompt_file,
            texture_image,
            remesh,
            topology,
            target_polycount,
            decimation_mode,
            save_pre_remeshed_model,
            image_enhancement,
            keep_lighting,
            target_format,
            multi_view_thumbnails,
            output,
            wait,
        } = self;

        let lowpoly = model_type == ModelType::Lowpoly;

        // Lowpoly ignores the model and remesh options, so reject them outright.
        if lowpoly {
            for (set, flag) in [
                (model.is_some(), "--model"),
                (topology.is_some(), "--topology"),
                (target_polycount.is_some(), "--target-polycount"),
                (decimation_mode.is_some(), "--decimation-mode"),
                (remesh.is_some(), "--remesh"),
                (
                    save_pre_remeshed_model.is_some(),
                    "--save-pre-remeshed-model",
                ),
            ] {
                if set {
                    return Err(Error::LowpolyConflict(flag));
                }
            }
        }

        // Lowpoly is always Meshy 6, so its Meshy 6-only options still apply.
        let model = if lowpoly {
            None
        } else {
            Some(model.unwrap_or_default())
        };
        let is_meshy6 = model.map(Model::is_meshy6).unwrap_or(true);

        // Texturing options require --texture.
        if !texture {
            for (set, flag) in [
                (texture_gen_pbr.is_some(), "--texture-gen-pbr"),
                (texture_quality.is_some(), "--texture-quality"),
                (texture_prompt.is_some(), "--texture-prompt"),
                (texture_prompt_file.is_some(), "--texture-prompt-file"),
                (texture_image.is_some(), "--texture-image"),
            ] {
                if set {
                    return Err(Error::TextureOptionWithoutTexture(flag));
                }
            }
        }

        // At most one texture guidance source may be given.
        let sources = [
            texture_prompt.is_some(),
            texture_prompt_file.is_some(),
            texture_image.is_some(),
        ];
        if sources.iter().filter(|set| **set).count() > 1 {
            return Err(Error::TexturePromptConflict);
        }

        // Remesh options require --remesh (lowpoly is already rejected above).
        let remesh_on = if lowpoly {
            false
        } else {
            remesh.unwrap_or(true)
        };
        if !lowpoly && !remesh_on {
            for (set, flag) in [
                (topology.is_some(), "--topology"),
                (target_polycount.is_some(), "--target-polycount"),
                (decimation_mode.is_some(), "--decimation-mode"),
                (
                    save_pre_remeshed_model.is_some(),
                    "--save-pre-remeshed-model",
                ),
            ] {
                if set {
                    return Err(Error::RemeshOptionWithoutRemesh(flag));
                }
            }
        }

        // target_polycount and decimation_mode are mutually exclusive.
        if target_polycount.is_some() && decimation_mode.is_some() {
            return Err(Error::PolycountDecimationConflict);
        }

        // image_enhancement and keep_lighting require Meshy 6.
        if !is_meshy6 {
            if image_enhancement.is_some() {
                return Err(Error::ImageEnhancementUnavailable);
            }
            if keep_lighting.is_some() {
                return Err(Error::KeepLightingUnavailable);
            }
        }

        // Paths are anchored to the cwd but stored relative to the task file so
        // the file and its outputs travel together.
        let cwd = dependencies.current_dir()?;
        let image_abs = absolute(&cwd, image.clone());
        let output_base = output_base
            .unwrap_or_else(|| PathBuf::from(image.file_stem().unwrap_or(image.as_os_str())));
        let output_base_abs = absolute(&cwd, output_base);
        let json_path = with_suffix(&output_base_abs, ".meshy.mesh.json");
        let json_dir = parent_dir(&json_path);
        let image_rel = relative(&json_dir, &image_abs);

        let texture_image_abs = texture_image.map(|path| absolute(&cwd, path));
        let texture_image_rel = texture_image_abs
            .as_ref()
            .map(|path| relative(&json_dir, path));

        // Texture phase fields, present only when texturing.
        let (enable_pbr, hd_texture, texture_prompt) = if texture {
            let quality = texture_quality.unwrap_or_default();
            if quality.is_hd() && !is_meshy6 {
                return Err(Error::HdTextureUnavailable);
            }
            let prompt = match (texture_prompt, texture_prompt_file) {
                (Some(prompt), _) => Some(prompt),
                (None, Some(file)) => Some(dependencies.read_text(&absolute(&cwd, file))?),
                (None, None) => None,
            };
            if let Some(prompt) = &prompt {
                let length = prompt.chars().count();
                if length > 600 {
                    return Err(Error::TexturePromptTooLong(length));
                }
            }
            (
                Some(texture_gen_pbr.unwrap_or(true)),
                Some(quality.is_hd()),
                prompt,
            )
        } else {
            (None, None, None)
        };

        // Remesh phase fields, present only when remeshing.
        let (topology, target_polycount, save_pre_remeshed_model) = if remesh_on {
            let topology = topology.unwrap_or_default().as_api_str().to_owned();
            // The default polycount is dropped when a decimation mode is set.
            let polycount = if decimation_mode.is_some() {
                None
            } else {
                Some(target_polycount.unwrap_or(30000))
            };
            (
                Some(topology),
                polycount,
                Some(save_pre_remeshed_model.unwrap_or(false)),
            )
        } else {
            (None, None, None)
        };

        // Meshy 6-only fields. `--keep-lighting` is the inverse of the API's
        // `remove_lighting`.
        let (image_enhancement, remove_lighting) = if is_meshy6 {
            (
                Some(image_enhancement.unwrap_or(true)),
                Some(!keep_lighting.unwrap_or(false)),
            )
        } else {
            (None, None)
        };

        // Target formats default to usdz, deduplicated with order preserved.
        let formats = if target_format.is_empty() {
            vec![TargetFormat::Usdz]
        } else {
            target_format
        };
        let mut target_formats: Vec<String> = Vec::new();
        for format in formats {
            let format = format.as_api_str().to_owned();
            if !target_formats.contains(&format) {
                target_formats.push(format);
            }
        }

        let input = MeshInput {
            image: image_rel,
            model_type: model_type.as_api_str().to_owned(),
            ai_model: model.map(|model| model.as_api_str().to_owned()),
            should_texture: texture,
            enable_pbr,
            hd_texture,
            texture_prompt,
            texture_image: texture_image_rel,
            should_remesh: if lowpoly { None } else { Some(remesh_on) },
            topology,
            target_polycount,
            decimation_mode,
            save_pre_remeshed_model,
            image_enhancement,
            remove_lighting,
            target_formats,
            pose_mode: String::new(),
            moderation: false,
            auto_size: false,
            multi_view_thumbnails,
        };

        let api_key = dependencies
            .meshy_api_key()?
            .ok_or(Error::ApiKeyNotConfigured)?;

        let request = MeshRequest {
            input: input.clone(),
            image_path: image_abs,
            texture_image_path: texture_image_abs,
        };
        let task_id = dependencies.create_task(&api_key, &request)?;

        // Record the task before any wait so its id is never lost on a later
        // failure.
        dependencies.write_task_file(
            &json_path,
            &MeshTaskFile {
                task_id: task_id.clone(),
                input: input.clone(),
                output: MeshOutput::Pending,
            },
        )?;
        let Some((interval, timeout)) = wait.interval_timeout() else {
            // Not waiting: report the freshly created, still-pending task.
            if let Some(line) = output.report_line(&task_id, "PENDING", 0, false) {
                dependencies.write_stdout(line.as_bytes())?;
            }
            return Ok(());
        };

        let task = wait_for_task(
            &dependencies,
            || dependencies.get_task(&api_key, &task_id),
            output,
            &task_id,
            interval,
            timeout,
        )?;
        finish_task(
            &dependencies,
            task,
            &output_base_abs,
            &json_dir,
            output,
            &task_id,
            |output| {
                dependencies.write_task_file(
                    &json_path,
                    &MeshTaskFile {
                        task_id: task_id.clone(),
                        input,
                        output,
                    },
                )
            },
        )
    }
}
