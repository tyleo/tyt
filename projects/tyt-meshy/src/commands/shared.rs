use crate::{
    Dependencies, Error, MeshOutput, MeshOutputDone, MeshProcessed, MeshTask, OutputMode, Result,
};
use std::path::{Path, PathBuf};
use tyt_common::relativize;

/// Whether a task has reached a terminal status (successfully or not).
pub(crate) fn is_terminal(task: &MeshTask) -> bool {
    matches!(task.status.as_str(), "SUCCEEDED" | "FAILED" | "CANCELED")
}

/// Polls a task until it reaches a terminal status or the timeout elapses.
/// `get` fetches the task from its endpoint. Each non-terminal poll reports the
/// task's current progress through `output` (silent for the `id` and `quiet`
/// modes).
pub(crate) fn wait_for_task(
    dependencies: &impl Dependencies,
    get: impl Fn() -> Result<MeshTask>,
    output: OutputMode,
    task_id: &str,
    interval: u64,
    timeout: u64,
) -> Result<MeshTask> {
    let mut waited = 0u64;
    loop {
        let task = get()?;
        if is_terminal(&task) {
            return Ok(task);
        }
        if let Some(line) = output.poll_line(task_id, &task.status, task.progress) {
            dependencies.write_stdout(line.as_bytes())?;
        }
        if waited >= timeout {
            return Err(Error::PollTimeout(timeout));
        }
        dependencies.sleep(interval)?;
        waited += interval;
    }
}

/// Downloads a terminal task's files (when it succeeded), records the final
/// output with `write`, and fails when the task did not succeed. The recorded
/// output of a failed task still captures the raw response, with its
/// `task_error`.
pub(crate) fn finish_task(
    dependencies: &impl Dependencies,
    task: MeshTask,
    output_base_abs: &Path,
    json_dir: &Path,
    output: OutputMode,
    task_id: &str,
    write: impl FnOnce(MeshOutput) -> Result<()>,
) -> Result<()> {
    let succeeded = task.status == "SUCCEEDED";
    let status = task.status.clone();
    let progress = task.progress;
    let error_message = task.error_message.clone();
    let processed = if succeeded {
        download_outputs(dependencies, &task, output_base_abs, json_dir)?
    } else {
        MeshProcessed::default()
    };
    // The downloaded thumbnails' absolute paths, front first, recovered from
    // their task-file-relative paths before `processed` is consumed below.
    let thumbnails: Vec<PathBuf> = processed
        .thumbnail_files
        .iter()
        .map(|(_, rel)| absolute(json_dir, PathBuf::from(rel)))
        .collect();
    write(MeshOutput::Done(MeshOutputDone {
        raw_json: task.raw_json,
        processed,
    }))?;
    if !succeeded {
        // A terminal non-success is the API's outcome to report, not a misuse of
        // this program, so print it and exit cleanly rather than erroring out.
        if let Some(line) =
            output.failure_line(task_id, &status, progress, error_message.as_deref())
        {
            dependencies.write_stdout(line.as_bytes())?;
        }
        return Ok(());
    }
    if let Some(line) = output.report_line(task_id, &status, progress, true) {
        dependencies.write_stdout(line.as_bytes())?;
    }
    // viuer leaves the cursor directly below an image without a trailing
    // newline, so each render emits one to end on a fresh line.
    match output.select_thumbnails(&thumbnails) {
        [] => {}
        [single] => {
            dependencies.display_image_in_terminal(single)?;
            dependencies.write_stdout(b"\n")?;
        }
        many => {
            let paths: Vec<&Path> = many.iter().map(PathBuf::as_path).collect();
            dependencies.display_images_in_grid(&paths, GRID_COLUMNS, GRID_SIDE_PERCENT)?;
            dependencies.write_stdout(b"\n")?;
        }
    }
    Ok(())
}

/// The number of columns in a multi-thumbnail grid: the four cardinal views lay
/// out two-by-two.
const GRID_COLUMNS: u32 = 2;

/// How much of the terminal's shorter side a multi-thumbnail grid spans, in
/// percent. 100 fills the shorter side exactly; this leaves a small margin for
/// the prompt while still using most of the screen.
const GRID_SIDE_PERCENT: u32 = 90;

/// Downloads a completed task's files into the task file's directory and records
/// their paths, relative to the task file.
pub(crate) fn download_outputs(
    dependencies: &impl Dependencies,
    task: &MeshTask,
    output_base_abs: &Path,
    json_dir: &Path,
) -> Result<MeshProcessed> {
    let mut processed = MeshProcessed::default();

    for (format, url) in &task.model_urls {
        let suffix = if format == "pre_remeshed_glb" {
            ".pre-remeshed.glb".to_owned()
        } else {
            format!(".{format}")
        };
        let path = write_download(dependencies, url, output_base_abs, &suffix)?;
        processed
            .model_files
            .push((format.clone(), relative(json_dir, &path)));
    }

    for (map, url) in &task.texture_urls {
        // Meshy's base color map is recorded as `albedo`.
        let name = if map == "base_color" { "albedo" } else { map };
        let path = write_download(dependencies, url, output_base_abs, &format!(".{name}.png"))?;
        processed
            .texture_files
            .push((name.to_owned(), relative(json_dir, &path)));
    }

    for (name, url) in &task.thumbnail_urls {
        let path = write_download(
            dependencies,
            url,
            output_base_abs,
            &format!(".thumbnail-{name}.png"),
        )?;
        processed
            .thumbnail_files
            .push((name.clone(), relative(json_dir, &path)));
    }

    Ok(processed)
}

/// Downloads a URL and writes it to `<output-base><suffix>`, returning the path.
fn write_download(
    dependencies: &impl Dependencies,
    url: &str,
    output_base_abs: &Path,
    suffix: &str,
) -> Result<PathBuf> {
    let path = with_suffix(output_base_abs, suffix);
    let bytes = dependencies.download(url)?;
    dependencies.write_file(&path, &bytes)?;
    Ok(path)
}

/// Resolves `path` against `base` when it is relative, leaving absolute paths
/// unchanged.
pub(crate) fn absolute(base: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

/// Returns the directory containing `path`, treating an empty parent as the
/// current directory.
pub(crate) fn parent_dir(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

/// Appends `suffix` to `base`'s path (e.g. `out/foo` + `.usdz` → `out/foo.usdz`).
pub(crate) fn with_suffix(base: &Path, suffix: &str) -> PathBuf {
    let mut name = base.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// Expresses `path` relative to `base` as a string.
pub(crate) fn relative(base: &Path, path: &Path) -> String {
    relativize(base, path).to_string_lossy().into_owned()
}
