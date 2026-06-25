use clap::ValueEnum;

/// What a Meshy command prints: thumbnails, text, or nothing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum OutputMode {
    /// Render every thumbnail a finished task produced — the four cardinal
    /// views, or just the single default thumbnail when that is all there is —
    /// printing the task's `id:` and `status:` as text while it runs.
    #[default]
    AllThumbnails,

    /// Render only the front (default) thumbnail of a finished task, printing
    /// the task's `id:` and `status:` as text while it runs.
    FrontThumbnail,

    /// Print just the task's id, for scripting.
    Id,

    /// Print nothing.
    Quiet,

    /// Print just the task's status, for scripting.
    Status,

    /// Print the task's `id:` and `status:` as labeled text, rendering no
    /// images.
    Text,
}

impl OutputMode {
    /// The `id:` line printed once, up front, by the human-facing modes — both
    /// thumbnail modes and `text` — since a task's id is known before it runs,
    /// so the `status:` lines that follow need not repeat it. The scripting
    /// modes (`id`, `status`) and `quiet` print nothing here; `id` emits its
    /// bare id alongside the task's outcome instead.
    pub fn id_line(self, id: &str) -> Option<String> {
        match self {
            OutputMode::AllThumbnails | OutputMode::FrontThumbnail | OutputMode::Text => {
                Some(format!("id: {id}\n"))
            }
            OutputMode::Id | OutputMode::Quiet | OutputMode::Status => None,
        }
    }

    /// The status line for an in-progress or successfully finished task, given
    /// its id, status, and percent progress. The human-facing modes — both
    /// thumbnail modes and `text` — print the status verbatim under a `status:`
    /// label (their `id:` is emitted once by [`OutputMode::id_line`]); `status`
    /// prints the bare status for scripting, `id` the bare id, and `quiet`
    /// nothing.
    pub fn report_line(self, id: &str, status: &str, progress: u8) -> Option<String> {
        match self {
            OutputMode::AllThumbnails | OutputMode::FrontThumbnail | OutputMode::Text => {
                Some(format!("status: {status} ({progress}%)\n"))
            }
            OutputMode::Id => Some(format!("{id}\n")),
            OutputMode::Quiet => None,
            OutputMode::Status => Some(format!("{status} ({progress}%)\n")),
        }
    }

    /// The status text to print on each poll while `--wait` blocks on a task.
    /// The modes that show a human a live view report their status line; `id`
    /// and `quiet` stay silent while polling and print their id or nothing only
    /// once the task finishes.
    pub fn poll_line(self, id: &str, status: &str, progress: u8) -> Option<String> {
        match self {
            OutputMode::Id | OutputMode::Quiet => None,
            _ => self.report_line(id, status, progress),
        }
    }

    /// The text to print for a task that reached a terminal non-success status
    /// (`FAILED` or `CANCELED`), given its id, status, percent progress, and the
    /// API's error message, if any. A task failure is a reportable outcome
    /// rather than a program error, so every mode but `Quiet` prints it; the
    /// human-facing modes report it as a `status:` line (their `id:` was already
    /// emitted up front) with the error appended.
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
                let mut line = format!("status: {status} ({progress}%)\n");
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
