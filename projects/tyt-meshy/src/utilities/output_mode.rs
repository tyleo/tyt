use clap::ValueEnum;

/// What a Meshy command prints: thumbnails, text, or nothing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum OutputMode {
    /// Render every thumbnail a finished task produced — the four cardinal
    /// views, or just the single default thumbnail when that is all there is —
    /// or `pending` while the task is unfinished.
    #[default]
    AllThumbnails,

    /// Render only the front (default) thumbnail of a finished task, or
    /// `pending` while the task is unfinished.
    FrontThumbnail,

    /// Print just the task's id, for scripting.
    Id,

    /// Print nothing.
    Quiet,

    /// Print just the task's status, for scripting.
    Status,

    /// Print the task's `status:` and `id:` as labeled text, rendering no
    /// images.
    Text,
}

impl OutputMode {
    /// The text to print for a task with the given id, status, and percent
    /// progress. `done` marks a successfully finished task: the thumbnail modes
    /// then render images rather than text, so they print nothing here; while a
    /// task is unfinished they print `pending` instead.
    pub fn report_line(self, id: &str, status: &str, progress: u8, done: bool) -> Option<String> {
        match self {
            OutputMode::AllThumbnails | OutputMode::FrontThumbnail => {
                (!done).then(|| format!("pending ({progress}%)\n"))
            }
            OutputMode::Id => Some(format!("{id}\n")),
            OutputMode::Quiet => None,
            OutputMode::Status => Some(format!("{status} ({progress}%)\n")),
            OutputMode::Text => Some(format!("status: {status} ({progress}%)\nid: {id}\n")),
        }
    }

    /// The text to print for a task that reached a terminal non-success status
    /// (`FAILED` or `CANCELED`), given its id, status, percent progress, and the
    /// API's error message, if any. A task failure is a reportable outcome
    /// rather than a program error, so every mode but `Quiet` prints it; the
    /// modes that would otherwise render an image report it as labeled text.
    pub fn failure_line(
        self,
        id: &str,
        status: &str,
        progress: u8,
        error: Option<&str>,
    ) -> Option<String> {
        match self {
            OutputMode::Quiet => None,
            OutputMode::Id => Some(format!("{id}\n")),
            OutputMode::Status => Some(format!("{status} ({progress}%)\n")),
            OutputMode::AllThumbnails | OutputMode::FrontThumbnail | OutputMode::Text => {
                let mut line = format!("status: {status} ({progress}%)\nid: {id}\n");
                if let Some(error) = error.filter(|error| !error.is_empty()) {
                    line.push_str(&format!("error: {error}\n"));
                }
                Some(line)
            }
        }
    }

    /// Selects which of a finished task's thumbnails to render, from the full
    /// ordered list (front first), returning them in render order. `AllThumbnails`
    /// keeps them all — falling back to the single default when only one exists —
    /// while `FrontThumbnail` keeps just the first.
    pub fn select_thumbnails<T>(self, thumbnails: &[T]) -> &[T] {
        match self {
            OutputMode::AllThumbnails => thumbnails,
            OutputMode::FrontThumbnail => &thumbnails[..thumbnails.len().min(1)],
            OutputMode::Id | OutputMode::Quiet | OutputMode::Status | OutputMode::Text => &[],
        }
    }
}
