use crate::{
    Dependencies, Error, OutputMode, Result,
    commands::{
        WaitArgs,
        shared::{absolute, finish_task, is_terminal, parent_dir, wait_for_task},
    },
};
use clap::Parser;
use std::path::{Path, PathBuf};

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

    /// How much to print.
    #[arg(value_name = "output", long, value_enum, default_value_t = OutputMode::AllThumbnails)]
    output: OutputMode,

    #[command(flatten)]
    wait: WaitArgs,
}

impl Poll {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Poll {
            meshy_json_path,
            output,
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

        let get = || match head.task_kind.as_str() {
            "image-to-3d" => dependencies.get_task(&api_key, &head.task_id),
            "retexture" => dependencies.get_texture_task(&api_key, &head.task_id),
            other => Err(Error::InvalidTaskFile(format!(
                "unsupported taskKind \"{other}\""
            ))),
        };
        // The id is known up front, so print it once; the status lines that
        // follow need not repeat it.
        if let Some(line) = output.id_line(&head.task_id) {
            dependencies.write_stdout(line.as_bytes())?;
        }

        let task = match wait.interval_timeout() {
            Some((interval, timeout)) => {
                wait_for_task(&dependencies, get, output, &head.task_id, interval, timeout)?
            }
            None => {
                // Check once: report and stop when still in progress.
                let task = get()?;
                if !is_terminal(&task) {
                    if let Some(line) =
                        output.report_line(&head.task_id, &task.status, task.progress)
                    {
                        dependencies.write_stdout(line.as_bytes())?;
                    }
                    return Ok(());
                }
                task
            }
        };

        finish_task(
            &dependencies,
            task,
            &output_base_abs,
            &json_dir,
            output,
            &head.task_id,
            |output| dependencies.write_polled_task_file(&json_path, &head, &output),
        )
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
