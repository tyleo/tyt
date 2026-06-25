/// Workspace-relative directory grouping the `tyt` binary crate and every
/// `tyt-*` sub-crate. New crates are scaffolded under here, and wiring targets
/// the binary crate at `{TYT_PROJECT_DIR}/tyt`.
pub(crate) const TYT_PROJECT_DIR: &str = "projects/tyt";
