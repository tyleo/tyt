use clap::Args;

/// The shared `--wait` option for commands that can block on a Meshy task.
#[derive(Clone, Debug, Args)]
pub(crate) struct WaitArgs {
    /// Block until the task completes and download its result files. Takes an
    /// optional poll interval and timeout, in seconds (e.g. `--wait 5 600`);
    /// they default to 10 and 300.
    #[arg(
        value_names = ["interval", "timeout"],
        long,
        num_args = 0..=2,
        value_parser = clap::value_parser!(u64).range(1..),
    )]
    wait: Option<Vec<u64>>,
}

impl WaitArgs {
    /// Resolves `--wait` into its poll interval and timeout, in seconds, or
    /// `None` when not waiting.
    pub(crate) fn interval_timeout(&self) -> Option<(u64, u64)> {
        self.wait.as_ref().map(|values| {
            (
                values.first().copied().unwrap_or(10),
                values.get(1).copied().unwrap_or(300),
            )
        })
    }
}
