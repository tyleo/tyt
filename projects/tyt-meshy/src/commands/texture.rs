use crate::{
    Dependencies, Error, MeshOutput, MeshOutputDone, Model, Result, TargetFormat, TextureInput,
    TextureQuality, TextureRequest, TextureTaskFile,
    commands::shared::{absolute, download_outputs, parent_dir, poll_task, relative, with_suffix},
};
use clap::{ArgAction, Parser};
use std::path::{Path, PathBuf};

/// Retextures an existing Meshy mesh using the Meshy [Retexture](https://docs.meshy.ai/en/api/retexture) API.
///
/// Reads a `<output-base>.meshy.mesh.json` written by `tyt meshy mesh` and sends
/// its `taskId` as the retexture `input_task_id`. A texture style is required:
/// exactly one of `--texture-prompt`, `--texture-prompt-file`, or
/// `--texture-image`. Writes `<output-base>.meshy.texture.json` and, with
/// `--poll` (the default), downloads the result files.
#[derive(Clone, Debug, Parser)]
#[command(name = "texture")]
pub struct Texture {
    /// The source `<output-base>.meshy.mesh.json` file, relative to the current
    /// directory. Its `taskId` is sent as `input_task_id`.
    #[arg(value_name = "meshy-mesh-json-path")]
    meshy_mesh_json_path: PathBuf,

    /// A text prompt describing the desired texture style (max 600 characters).
    #[arg(value_name = "texture-prompt", long = "texture-prompt")]
    texture_prompt: Option<String>,

    /// A file holding the texturing prompt, relative to the current directory
    /// (max 600 characters).
    #[arg(value_name = "texture-prompt-file", long = "texture-prompt-file")]
    texture_prompt_file: Option<PathBuf>,

    /// An image guiding texturing, relative to the current directory.
    #[arg(value_name = "texture-image", long = "texture-image")]
    texture_image: Option<PathBuf>,

    /// The model to use for retexturing.
    #[arg(value_name = "model", long, value_enum, default_value_t = Model::Meshy6)]
    model: Model,

    /// Generate PBR maps (metallic, roughness, normal) alongside the base color.
    #[arg(
        value_name = "texture-gen-pbr",
        long = "texture-gen-pbr",
        action = ArgAction::Set,
        num_args = 0..=1,
        require_equals = true,
        default_value_t = true,
        default_missing_value = "true",
    )]
    texture_gen_pbr: bool,

    /// The base color texture quality. `hd` requires Meshy 6.
    #[arg(value_name = "texture-quality", long = "texture-quality", value_enum, default_value_t = TextureQuality::Normal)]
    texture_quality: TextureQuality,

    /// Reuse the source model's original UVs instead of generating new ones.
    #[arg(
        value_name = "original-uv",
        long = "original-uv",
        action = ArgAction::Set,
        num_args = 0..=1,
        require_equals = true,
        default_value_t = false,
        default_missing_value = "true",
    )]
    original_uv: bool,

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

    /// Poll the task until it completes and write the result files.
    #[arg(
        value_name = "poll",
        long,
        action = ArgAction::Set,
        num_args = 0..=1,
        require_equals = true,
        default_value_t = true,
        default_missing_value = "true",
    )]
    poll: bool,

    /// Seconds between poll attempts. Requires `--poll`; defaults to 10.
    #[arg(value_name = "poll-interval", long = "poll-interval", value_parser = clap::value_parser!(u64).range(1..))]
    poll_interval: Option<u64>,

    /// Seconds to wait before giving up on polling. Requires `--poll`; defaults
    /// to 300.
    #[arg(value_name = "poll-timeout", long = "poll-timeout")]
    poll_timeout: Option<u64>,
}

impl Texture {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Texture {
            meshy_mesh_json_path,
            texture_prompt,
            texture_prompt_file,
            texture_image,
            model,
            texture_gen_pbr,
            texture_quality,
            original_uv,
            keep_lighting,
            target_format,
            poll,
            poll_interval,
            poll_timeout,
        } = self;

        let is_meshy6 = model.is_meshy6();

        // Exactly one texture style is required.
        let styles = [
            texture_prompt.is_some(),
            texture_prompt_file.is_some(),
            texture_image.is_some(),
        ];
        match styles.iter().filter(|set| **set).count() {
            0 => return Err(Error::TextureStyleRequired),
            1 => {}
            _ => return Err(Error::TexturePromptConflict),
        }

        // hd quality and keep-lighting require Meshy 6.
        if texture_quality.is_hd() && !is_meshy6 {
            return Err(Error::HdTextureUnavailable);
        }
        if !is_meshy6 && keep_lighting.is_some() {
            return Err(Error::KeepLightingUnavailable);
        }

        // Polling options require --poll.
        if !poll {
            if poll_interval.is_some() {
                return Err(Error::PollOptionWithoutPoll("--poll-interval"));
            }
            if poll_timeout.is_some() {
                return Err(Error::PollOptionWithoutPoll("--poll-timeout"));
            }
        }
        let poll_interval = poll_interval.unwrap_or(10);
        let poll_timeout = poll_timeout.unwrap_or(300);

        // The output base is the source path with its `.meshy.mesh.json` suffix
        // stripped; outputs are written alongside it.
        let cwd = dependencies.current_dir()?;
        let input_json_path = absolute(&cwd, meshy_mesh_json_path);
        let output_base_abs = strip_mesh_suffix(&input_json_path);
        let json_path = with_suffix(&output_base_abs, ".meshy.texture.json");
        let json_dir = parent_dir(&json_path);

        let input_task_id = dependencies.read_input_task_id(&input_json_path)?;

        // The texture style is either a text prompt or a local style image.
        let text_style_prompt = match (texture_prompt, texture_prompt_file) {
            (Some(prompt), _) => Some(prompt),
            (None, Some(file)) => Some(dependencies.read_text(&absolute(&cwd, file))?),
            (None, None) => None,
        };
        if let Some(prompt) = &text_style_prompt {
            let length = prompt.chars().count();
            if length > 600 {
                return Err(Error::TexturePromptTooLong(length));
            }
        }

        let style_image_abs = texture_image.map(|path| absolute(&cwd, path));
        let image_style_url = style_image_abs
            .as_ref()
            .map(|path| relative(&json_dir, path));

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

        let input = TextureInput {
            input_task_id,
            text_style_prompt,
            image_style_url,
            ai_model: model.as_api_str().to_owned(),
            enable_pbr: texture_gen_pbr,
            hd_texture: texture_quality.is_hd(),
            enable_original_uv: original_uv,
            // `--keep-lighting` is the inverse of the API's `remove_lighting`.
            remove_lighting: is_meshy6.then(|| !keep_lighting.unwrap_or(false)),
            target_formats,
            moderation: false,
        };

        let api_key = dependencies
            .meshy_api_key()?
            .ok_or(Error::ApiKeyNotConfigured)?;

        let request = TextureRequest {
            input: input.clone(),
            image_style_path: style_image_abs,
        };
        let task_id = dependencies.create_texture_task(&api_key, &request)?;

        // Record the task before polling so its id is never lost on a later
        // failure.
        dependencies.write_texture_task_file(
            &json_path,
            &TextureTaskFile {
                task_id: task_id.clone(),
                input: input.clone(),
                output: MeshOutput::Pending,
            },
        )?;
        dependencies.write_stdout(format!("{task_id}\n").as_bytes())?;

        if !poll {
            return Ok(());
        }

        let task = poll_task(
            &dependencies,
            || dependencies.get_texture_task(&api_key, &task_id),
            poll_interval,
            poll_timeout,
        )?;
        let processed = download_outputs(&dependencies, &task, &output_base_abs, &json_dir)?;

        dependencies.write_texture_task_file(
            &json_path,
            &TextureTaskFile {
                task_id,
                input,
                output: MeshOutput::Done(MeshOutputDone {
                    raw_json: task.raw_json,
                    processed,
                }),
            },
        )?;

        Ok(())
    }
}

/// Strips a trailing `.meshy.mesh.json` from `path`, falling back to dropping a
/// single extension when the suffix is absent.
fn strip_mesh_suffix(path: &Path) -> PathBuf {
    match path
        .to_str()
        .and_then(|s| s.strip_suffix(".meshy.mesh.json"))
    {
        Some(stripped) => PathBuf::from(stripped),
        None => path.with_extension(""),
    }
}
