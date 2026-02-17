/// Preferences for `tyt-fs` loaded from `.tytconfig`.
#[derive(Debug, Default, serde::Deserialize)]
pub struct FsPrefs {
    pub scratch_dir: Option<String>,
}
