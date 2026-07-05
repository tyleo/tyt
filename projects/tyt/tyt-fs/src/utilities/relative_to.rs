use clap::ValueEnum;

/// What the output path of `tyt fs rel` is expressed relative to.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum RelativeTo {
    /// Relative to the current working directory.
    Cwd,
    /// Relative to the config file that defined the base path.
    Config,
}
