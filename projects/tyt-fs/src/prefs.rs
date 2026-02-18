/// Preferences for `tyt-fs` loaded from `.tytconfig`.
#[derive(Debug, Default)]
#[cfg_attr(feature = "impl", derive(serde::Deserialize))]
pub struct Prefs {
    pub scratch_dir: Option<String>,
}
