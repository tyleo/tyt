/// Preferences for `move-to-scratch`, read from the `fs.move-to-scratch`
/// section of `.tytconfig`.
#[derive(Debug, Default)]
#[cfg_attr(
    feature = "impl",
    derive(serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct MoveToScratchPrefs {
    /// The directory files are moved into, interpreted relative to the git
    /// root when not absolute.
    pub scratch_dir: Option<String>,
}
