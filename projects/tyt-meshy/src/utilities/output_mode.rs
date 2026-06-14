use clap::ValueEnum;

/// How much a Meshy command prints to stdout.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum OutputMode {
    /// Label printed values (e.g. `task id: <id>`) and render the downloaded
    /// thumbnail in the terminal.
    #[default]
    Full,

    /// Print bare values without labels or images.
    Plain,

    /// Print nothing.
    Quiet,
}

impl OutputMode {
    /// Returns the line announcing a created task's id, or `None` when not
    /// printing.
    pub fn task_id_line(self, task_id: &str) -> Option<String> {
        match self {
            OutputMode::Full => Some(format!("task id: {task_id}\n")),
            OutputMode::Plain => Some(format!("{task_id}\n")),
            OutputMode::Quiet => None,
        }
    }

    /// Returns the line reporting an in-progress task's status, or `None` when
    /// not printing.
    pub fn status_line(self, status: &str, progress: u8) -> Option<String> {
        match self {
            OutputMode::Full => Some(format!("status: {status} ({progress}%)\n")),
            OutputMode::Plain => Some(format!("{status} ({progress}%)\n")),
            OutputMode::Quiet => None,
        }
    }

    /// Whether to render the downloaded thumbnail in the terminal.
    pub fn shows_image(self) -> bool {
        self == OutputMode::Full
    }
}
