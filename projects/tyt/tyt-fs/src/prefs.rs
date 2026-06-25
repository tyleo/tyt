use crate::MoveToScratchPrefs;
use std::collections::HashMap;

/// Preferences for `tyt-fs` loaded from the `fs` section of `.tytconfig`.
#[derive(Debug, Default)]
#[cfg_attr(
    feature = "impl",
    derive(serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct Prefs {
    /// Preferences for `move-to-scratch`, read from `fs.move-to-scratch`.
    pub move_to_scratch: Option<MoveToScratchPrefs>,
    /// Named base paths, keyed by name, read from the `fs.rel` section. Each
    /// value is interpreted relative to the config file that defines it.
    pub rel: Option<HashMap<String, String>>,
}
