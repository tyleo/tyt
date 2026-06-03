use std::{collections::HashMap, path::PathBuf};

/// The merged `fs.rel` configuration resolved from `.tytconfig` and
/// `.tytusrconfig`.
#[derive(Clone, Debug)]
pub struct RelConfig {
    /// The directory the config files live in (the git root). Base paths are
    /// resolved relative to it, and it is the reference for `--relative-to
    /// config`.
    pub config_dir: PathBuf,
    /// Maps each `fs.rel` base name to its resolved absolute base path. On a
    /// duplicate key the `.tytusrconfig` value wins.
    pub bases: HashMap<String, PathBuf>,
}
