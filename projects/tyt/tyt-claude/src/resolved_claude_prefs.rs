use std::collections::BTreeMap;

/// `ClaudePrefs` resolved across all cascade layers (most-local wins).
#[derive(Clone, Debug, Default)]
pub struct ResolvedClaudePrefs {
    /// Profile name to the directory used as `CLAUDE_CONFIG_DIR`.
    pub profiles: BTreeMap<String, String>,
    /// The currently active profile name, if any.
    pub active: Option<String>,
}
