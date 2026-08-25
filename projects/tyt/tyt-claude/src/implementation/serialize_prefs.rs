use crate::ClaudePrefs;
use std::io::Result as IOResult;
use tyt_preferences::SerializePrefs;

impl SerializePrefs for ClaudePrefs {
    fn serialize_prefs(&self, key: &str, existing: Option<&[u8]>) -> IOResult<Vec<u8>> {
        tyt_preferences::serialize_prefs_jsonc(self, key, existing)
    }
}
