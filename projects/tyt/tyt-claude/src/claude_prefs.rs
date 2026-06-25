use std::collections::BTreeMap;

/// The `claude` section of a `.tytconfig` / `.tytusrconfig` file.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "impl", derive(serde::Deserialize, serde::Serialize))]
pub struct ClaudePrefs {
    /// Profile name to the directory used as `CLAUDE_CONFIG_DIR`.
    #[cfg_attr(feature = "impl", serde(default))]
    pub profiles: BTreeMap<String, String>,
    /// The currently active profile name, if any.
    #[cfg_attr(
        feature = "impl",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub active: Option<String>,
}

/// Section key for `ClaudePrefs` inside a `.tytconfig` / `.tytusrconfig` file.
pub const CLAUDE_PREFS_KEY: &str = "claude";
