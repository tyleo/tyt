use crate::{
    Dependencies, Error, MeshOutput, MeshOutputDone, MeshProcessed, Result,
    commands::shared::{absolute, download_outputs, parent_dir},
};
use clap::{ArgAction, Parser};
use std::path::{Path, PathBuf};

/// Seconds between poll attempts when `--wait` is set.
const POLL_INTERVAL: u64 = 10;

/// Polls a previously created Meshy task and, once it has completed, writes its result files.
///
/// Reads a `<output-base>.meshy.mesh.json` or `<output-base>.meshy.texture.json`
/// written by `tyt meshy mesh` or `tyt meshy texture`, polls the task named by
/// its `taskId` (using its `taskKind` to select the API), and on completion
/// rewrites the file in place with the final `output` and downloads the result
/// files alongside it.
#[derive(Clone, Debug, Parser)]
#[command(name = "poll")]
pub struct Poll {
    /// The `<output-base>.meshy.mesh.json` or `<output-base>.meshy.texture.json`
    /// file to poll, relative to the current directory.
    #[arg(value_name = "meshy-json-path")]
    meshy_json_path: PathBuf,

    /// Block and keep polling until the task completes, instead of checking
    /// once.
    #[arg(
        value_name = "wait",
        long,
        action = ArgAction::Set,
        num_args = 0..=1,
        require_equals = true,
        default_value_t = false,
        default_missing_value = "true",
    )]
    wait: bool,
}

impl Poll {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Poll {
            meshy_json_path,
            wait,
        } = self;

        // The tracking file is rewritten in place; outputs are named from its
        // base (the path with its `.meshy.{mesh,texture}.json` suffix stripped)
        // and written alongside it.
        let cwd = dependencies.current_dir()?;
        let json_path = absolute(&cwd, meshy_json_path);
        let output_base_abs = strip_suffix(&json_path);
        let json_dir = parent_dir(&json_path);

        let head = dependencies.read_task_file(&json_path)?;
        let api_key = dependencies
            .meshy_api_key()?
            .ok_or(Error::ApiKeyNotConfigured)?;

        let task = loop {
            let task = match head.task_kind.as_str() {
                "image-to-3d" => dependencies.get_task(&api_key, &head.task_id)?,
                "retexture" => dependencies.get_texture_task(&api_key, &head.task_id)?,
                other => {
                    return Err(Error::InvalidTaskFile(format!(
                        "unsupported taskKind \"{other}\""
                    )));
                }
            };
            match task.status.as_str() {
                "SUCCEEDED" | "FAILED" | "CANCELED" => break task,
                _ => {
                    // Still in progress: report and stop unless asked to wait.
                    if !wait {
                        let line = format!("{} ({}%)\n", task.status, task.progress);
                        dependencies.write_stdout(line.as_bytes())?;
                        return Ok(());
                    }
                    dependencies.sleep(POLL_INTERVAL)?;
                }
            }
        };

        // The task finished. Download its files on success; on failure the
        // recorded output still captures the raw response (with its task_error).
        let succeeded = task.status == "SUCCEEDED";
        let status = task.status.clone();
        let error_message = task.error_message.clone();
        let processed = if succeeded {
            download_outputs(&dependencies, &task, &output_base_abs, &json_dir)?
        } else {
            MeshProcessed::default()
        };

        dependencies.write_polled_task_file(
            &json_path,
            &head,
            &MeshOutput::Done(MeshOutputDone {
                raw_json: task.raw_json,
                processed,
            }),
        )?;

        if !succeeded {
            return Err(Error::TaskFailed(status, error_message.unwrap_or_default()));
        }

        Ok(())
    }
}

/// Strips a trailing `.meshy.mesh.json` or `.meshy.texture.json` from `path`,
/// falling back to dropping a single extension when neither suffix is present.
fn strip_suffix(path: &Path) -> PathBuf {
    if let Some(text) = path.to_str() {
        for suffix in [".meshy.mesh.json", ".meshy.texture.json"] {
            if let Some(stripped) = text.strip_suffix(suffix) {
                return PathBuf::from(stripped);
            }
        }
    }
    path.with_extension("")
}
